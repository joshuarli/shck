#!/usr/bin/env python3
import argparse
import datetime
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parent.parent
TIERS = ('Edit', 'Change', 'Scenario', 'Full')
BENCHES = {
    'parser': 'parser', 'lexer': 'parser', 'lexer_hot_path': 'parser',
    'arithmetic': 'parser', 'case_patterns': 'linter', 'indexer': 'semantic',
    'semantic': 'semantic', 'editor': 'editor', 'unused_assignment': 'linter',
    'linter': 'linter', 'formatter': 'formatter', 'check_command': 'cli',
}
WORKLOADS = [('driver-tests', 'Edit', 'harness')] + [
    (name, 'Change', area) for name, area in BENCHES.items()
] + [('corpus-' + name, 'Scenario', area) for name, area in BENCHES.items()] + [
    ('large-corpus-hotspots', 'Full', 'corpus')
]


def select(tier, workload, area, skip):
    if tier not in TIERS:
        raise ValueError(f'unknown tier {tier!r}; choose from {", ".join(TIERS)}')
    names = {w[0] for w in WORKLOADS}
    areas = {w[2] for w in WORKLOADS}
    for name in ([workload] if workload else []) + skip:
        if name not in names:
            raise ValueError(f'unknown workload {name!r}; see make bench-list')
    if area and area not in areas:
        raise ValueError(f'unknown area {area!r}; see make bench-list')
    selected = [w for w in WORKLOADS if TIERS.index(w[1]) <= TIERS.index(tier)
                and (not workload or w[0] == workload) and (not area or w[2] == area)
                and w[0] not in skip]
    if not selected:
        raise ValueError('selection matches no workloads within the tier ceiling')
    return selected


def command(workload, busy):
    name, tier, _ = workload
    if tier == 'Edit':
        return [sys.executable, '-m', 'unittest', 'discover', '-s', 'benches', '-p', 'test_driver.py']
    if tier == 'Full':
        return ['cargo', 'bench', '-p', 'shuck-benchmark', '--features', 'large-corpus-hotspots', '--bench', 'large_corpus_hotspots'] + (['--', '--test'] if busy else [])
    bench = name.removeprefix('corpus-')
    args = ['cargo', 'bench', '-p', 'shuck-benchmark', '--bench', bench, '--']
    if tier == 'Change':
        args += [{'arithmetic': 'double-paren-command', 'case_patterns': 'literal/equal'}.get(bench, 'fzf-install')]
    return args + (['--test'] if busy else ['--sample-count', '5', '--sample-size', '1'])


def preflight(selected):
    names = ', '.join(w[0] for w in selected)
    for tool in ('cargo', 'rustc', 'perf-gate-ready'):
        if not shutil.which(tool):
            raise ValueError(f'{tool} missing; install it or restore PATH (needed by {names})')
    manifest = ROOT / 'crates/shuck-benchmark/resources/manifest.json'
    for fixture in json.loads(manifest.read_text())['fixtures']:
        path = manifest.parent / fixture['local_filename']
        if not path.is_file() or path.stat().st_size != fixture['byte_size']:
            raise ValueError(f'fixture missing or wrong size: {path}; restore from git (needed by {names})')
    if any(w[1] == 'Full' for w in selected):
        hint = os.environ.get('SHUCK_LARGE_CORPUS_ROOT')
        candidates = [Path(hint)] if hint else [ROOT / '.cache/large-corpus', ROOT.parent / 'shell-checks']
        if not any((path / 'scripts').is_dir() or (path / 'corpus/scripts').is_dir() for path in candidates):
            raise ValueError(f'large corpus missing; run make setup-large-corpus or set SHUCK_LARGE_CORPUS_ROOT (needed by {names})')
    if shutil.disk_usage(ROOT).free < 1024**3:
        raise ValueError(f'less than 1 GiB free; free disk space (needed by {names})')
    result = subprocess.run(['perf-gate-ready'], text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    print(result.stdout.strip(), flush=True)
    if result.returncode not in (0, 75):
        raise ValueError(f'perf-gate-ready failed with exit {result.returncode}; inspect host gate')
    busy = result.returncode == 75
    print('HOST: busy; timing deferred; functional runs only' if busy else 'HOST: ready', flush=True)
    return busy


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('action', choices=('run', 'check', 'list'))
    parser.add_argument('--tier', default='Change')
    parser.add_argument('--workload', default='')
    parser.add_argument('--area', default='')
    parser.add_argument('--skip', action='append', default=[])
    parser.add_argument('--dry-run', action='store_true')
    args = parser.parse_args()
    try:
        selected = select(args.tier, args.workload, args.area, args.skip)
        if args.action == 'list':
            for name, tier, area in WORKLOADS:
                print(f'{name:30} {tier:8} {area:10} ' + ('selected' if (name, tier, area) in selected else 'excluded'))
            return 0
        if args.dry_run:
            for w in selected:
                print(f'{w[0]}: {shlex.join(command(w, False))} (busy host: {shlex.join(command(w, True))})')
            return 0
        busy = preflight(selected)
        if args.action == 'check':
            return 0
        out = ROOT / 'target' / 'bench-runs' / datetime.datetime.now().strftime('%Y%m%d-%H%M%S-%f')
        out.mkdir(parents=True)
        failed = False
        summary = []
        for w in WORKLOADS:
            if w not in selected:
                summary.append(f'{w[0]}: excluded by selection')
                continue
            cmd = command(w, busy)
            print(f'RUN {w[0]}: {shlex.join(cmd)}', flush=True)
            start = time.monotonic()
            with (out / (w[0] + '.log')).open('w') as log:
                result = subprocess.run(cmd, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
            elapsed = time.monotonic() - start
            failed |= result.returncode != 0
            line = f'{w[0]}: exit {result.returncode}, {elapsed:.1f}s; ' + ('timing deferred: host busy' if busy and w[1] != 'Edit' else 'completion only; no performance budget')
            summary.append(line)
            print(line, flush=True)
        if not busy and any(w[1] != 'Edit' for w in selected):
            post = subprocess.run(['perf-gate-ready', '-1'], text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
            summary.append('POSTFLIGHT: ' + ('ready' if post.returncode == 0 else 'timing invalid: host busy or gate failed') + '\n' + post.stdout.strip())
        summary.append('No performance verdict: no established budget or paired baseline.')
        (out / 'summary.txt').write_text('\n'.join(summary) + '\n')
        print(f'Output: {out}\n' + summary[-1], flush=True)
        return int(failed)
    except (ValueError, OSError, json.JSONDecodeError) as error:
        print(f'bench: {error}', file=sys.stderr)
        return 2


if __name__ == '__main__':
    sys.exit(main())
