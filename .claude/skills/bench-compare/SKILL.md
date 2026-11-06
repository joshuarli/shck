---
name: bench-compare
description: >
  Compare benchmark performance between two git worktrees (or the current
  worktree vs main). Runs rustybench microbenchmarks and hyperfine macrobenchmarks,
  extracts deltas, and drills down into regressions. Use this skill whenever the
  user asks to compare benchmarks, check for performance regressions, benchmark
  their branch against main, run a perf comparison, or says things like "bench
  compare", "any regressions?", "compare perf", "how does this branch perform",
  "run benchmarks against main". Even if the user just says "benchmarks" in the
  context of a feature branch or worktree, this skill applies.
---

# Benchmark Comparison

Compare shuck's benchmark performance between two worktrees (typically the current
feature worktree and the main worktree). Produces a delta report covering both
rustybench microbenchmarks (lexer, parser, semantic, linter) and hyperfine
macrobenchmarks (wall-clock comparison against ShellCheck).

## Before you start

Identify the two worktrees to compare:

```bash
git worktree list
```

This gives you:
- **Main worktree** — the first entry, typically `/Users/.../shuck`
- **Current worktree** — the one you're running in (may be the same as main)

If both are the same directory, you'll need to stash or commit changes and write
two baseline files from the same tree. But the typical case is two
separate worktrees.

Also check what benchmark targets exist — the registered benches may
change over time:

```bash
grep -A1 '^\[\[bench\]\]' crates/shuck-benchmark/Cargo.toml | grep '^name'
```

## Step 1: Create a shared scratch directory

Baselines are explicit JSON files, so both worktrees only need a shared scratch
location. Point `CARGO_TARGET_DIR` at the scratch tree so the two worktrees reuse
one build cache.

```bash
SCRATCH=$(mktemp -d "${TMPDIR:-/tmp}/shuck-bench-compare.XXXXXX")
export CARGO_TARGET_DIR="$SCRATCH/target"
echo "Scratch: $SCRATCH"
```

Also create subdirectories for macrobenchmark exports:

```bash
mkdir -p "$SCRATCH/main-macro" "$SCRATCH/current-macro"
```

## Step 2: Run the rustybench baseline on main

Install the runner once with `cargo install rustybench`, then `cd` into the main
worktree and collect a baseline. `cargo bench -p shuck-benchmark` runs every
registered bench target.

```bash
cd /path/to/main/worktree
env CARGO_TARGET_DIR="$SCRATCH/target" \
  rustybench baseline --root . --baseline "$SCRATCH/main-bench.json" \
  -- cargo bench -p shuck-benchmark \
  > "$SCRATCH/main-bench.log" 2>&1
```

Monitor progress by tailing the log.

## Step 3: Run the current worktree and diff

`cd` into the current worktree, collect a second baseline, and diff the two files.

```bash
cd /path/to/current/worktree
env CARGO_TARGET_DIR="$SCRATCH/target" \
  rustybench baseline --root . --baseline "$SCRATCH/current-bench.json" \
  -- cargo bench -p shuck-benchmark \
  > "$SCRATCH/current-bench.log" 2>&1

rustybench diff "$SCRATCH/main-bench.json" "$SCRATCH/current-bench.json" \
  > "$SCRATCH/micro-diff.txt"
```

## Step 4: Run macrobenchmarks on both worktrees

Macrobenchmarks use hyperfine via `scripts/benchmarks/run.sh`, which requires
`hyperfine` and `shellcheck` — tools only available inside the nix dev shell.

For each worktree (main first, then current):

```bash
cd /path/to/worktree

# Clear stale bench exports to avoid mixing results
rm -f .cache/bench-*.json .cache/bench-*.md 2>/dev/null || true

# Build and verify deps
nix --extra-experimental-features 'nix-command flakes' develop --command \
  ./scripts/benchmarks/setup.sh > "$SCRATCH/{side}-macro-setup.log" 2>&1

# Run hyperfine comparisons
nix --extra-experimental-features 'nix-command flakes' develop --command \
  ./scripts/benchmarks/run.sh > "$SCRATCH/{side}-macro.log" 2>&1

# Copy exports to scratch
cp .cache/bench-*.json "$SCRATCH/{side}-macro/"
```

Replace `{side}` with `main` or `current` as appropriate.

## Step 5: Extract and report deltas

### Microbenchmark deltas

`rustybench diff` prints the comparison table for the two baseline files:

```bash
rustybench diff "$SCRATCH/main-bench.json" "$SCRATCH/current-bench.json"
```

Benchmark names follow the bench target, function, and case, for example
`parser::parse::all`. Use the diff output directly; no artifact scraping is
needed.

### Macrobenchmark deltas

Hyperfine exports JSON with a `results` array. Each result has `command` and `mean`
(in seconds). Compare the shuck entries between main and current:

```python
for p in sorted((ROOT / "main-macro").glob("bench-*.json")):
    name = p.stem.removeprefix("bench-")
    main_results = json.loads(p.read_text())["results"]
    cur_results = json.loads((ROOT / "current-macro" / p.name).read_text())["results"]
    m_shuck = next(r for r in main_results if r["command"].startswith("shuck/"))
    c_shuck = next(r for r in cur_results if r["command"].startswith("shuck/"))
    change = (c_shuck["mean"] / m_shuck["mean"] - 1) * 100
    print(f"  {name:30s}  {m_shuck['mean']*1000:.1f} → {c_shuck['mean']*1000:.1f} ms  ({change:+.2f}%)")
```

## Step 6: Interpret the results

Present a summary table to the user covering:
- Each rustybench benchmark group with % change
- Each macrobenchmark fixture with % change
- Highlight anything above +3% as a potential regression (microbenchmarks have noise, so
  small changes are usually not meaningful)

If no regressions are found, report that and stop.

## Step 7: Drill down into regressions

If a significant regression is found (>3% in microbenchmarks or >5% in macrobenchmarks),
investigate the root cause:

### 7a. Identify which component regressed

The rustybench benchmarks isolate stages: lexer, parser, semantic, linter. If only
`linter` regressed but `parser` and `semantic` are flat, the regression is in the
linting pipeline (checker, suppression, directives).

### 7b. Build a breakdown harness (if needed)

For finer-grained measurement, create a temporary
`crates/shuck-benchmark/examples/lint_breakdown.rs` that measures individual pipeline
stages:

1. Parse only
2. Indexer only
3. Directives/suppression only
4. Semantic model build only
5. Lint with empty rules
6. Lint with default rules

Run it on both worktrees with `cargo run -p shuck-benchmark --release --example lint_breakdown`
and compare the per-stage timings. This isolates which stage accounts for the
regression.

### 7c. Diff the suspected code

Once you've narrowed to a stage, diff the relevant source files between worktrees:

```bash
git diff --no-index /path/to/main/crates/... /path/to/current/crates/...
```

Look for:
- New `.clone()` calls on hot paths
- Recursive traversals that weren't there before
- Changed data structures (Vec → HashMap, etc.)
- New allocations in tight loops

### 7d. Report findings

Tell the user:
- Which benchmark regressed and by how much
- Which code change caused it (with file path and description)
- Whether you have a fix suggestion

Remove any temporary breakdown harness files after the investigation.

## Gotchas

- **Install the runner**: `rustybench baseline`, `diff`, and `check` come from the
  `rustybench` binary (`cargo install rustybench`), not from `cargo bench`.

- **Macrobenchmarks need nix**: `hyperfine` and `shellcheck` are only available
  inside `nix develop`. Always run macro setup and benchmarks through
  `nix --extra-experimental-features 'nix-command flakes' develop --command ...`.

- **Explicit baseline paths**: pass `--baseline` with a scratch path. Without it,
  rustybench derives a host-keyed file name under `benches/`.

- **Suppression-heavy fixtures**: `ruby-build.sh` and `nvm.sh` have many
  `shellcheck disable=` comments. Regressions in the directive/suppression parsing
  path show up disproportionately on these fixtures.

- **zsh glob errors**: When clearing `.cache/bench-*.json`, use
  `rm -f ... 2>/dev/null || true` because zsh complains when no files match a glob.

- **Benchmark noise**: rustybench medians on a laptop can have 2-3% noise.
  Don't chase regressions under 3% unless they're consistent across all fixtures.
  Macrobenchmarks (hyperfine) are more stable since they measure full CLI invocations.
