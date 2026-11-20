# Benchmarks and profiling

Benchmarks live in `crates/shuck-benchmark` and `scripts/benchmarks/`. They measure parser, indexer, semantic, linter, formatter, and end-to-end CLI performance against ShellCheck and shfmt.

## Fixtures

`crates/shuck-benchmark/resources/manifest.json` pins every fixture by upstream repository, path, commit, license, and byte size. `resources/files/` holds the vendored sources and `resources/licenses/` the license texts. `TEST_FILES` in `crates/shuck-benchmark/src/lib.rs` loads the fixtures used by the micro-benchmarks; a manifest invariant test requires every listed byte size to match the file on disk.

`BENCHMARK_CASES` derives a case per fixture plus an aggregate `all` case. Bench names and case names come from one registry, so per-file and aggregate runs stay in sync.

## Micro-benchmarks

Benches use rustybench with `harness = false` under `crates/shuck-benchmark/benches/`: lexer, lexer hot path, parser, indexer, arithmetic, semantic, editor, unused assignment, linter, formatter, check command, case patterns, and the feature-gated large-corpus hotspots. Every bench reports bytes so throughput is comparable across fixture sizes.

Bench binaries install jemalloc through `configure_benchmark_allocator!`; the `dhat-heap` feature switches allocation tracking for memory runs.

```sh
make bench
make bench-save            # write target/bench-baseline.json
make bench-compare         # compare target/bench-candidate.json to the baseline
make bench-parser          # one component
make bench-formatter
make bench-large-corpus-hotspots
```

`make bench-memory-save` and `make bench-memory-compare` run the parser and linter memory harnesses under `scripts/benchmarks/run_parser_memory.py` and `run_linter_memory.py`.

## Macro-benchmarks

The macro scripts run the built CLI against ShellCheck and shfmt on the vendored corpus and on cloned repositories. All macro targets enter the Nix development environment before running.

- `run.sh` compares `shuck check --no-cache --select ALL` with `shellcheck --enable=all --severity=style`.
- `run_formatter.sh` compares `shuck format --check --no-cache --dialect bash` with `shfmt -l -ln=bash`.
- `run_repo_corpus.sh` runs the whole-repository corpus defined by `.cache/large-corpus/manifest.yaml`.

```sh
make bench-macro
make bench-macro-single BENCH_FILE=path/to/file.sh
make bench-macro-format
make bench-repo-corpus
```

`bench-macro-site-local` regenerates the checked-in site snapshot from a local run. The scripts write JSON and Markdown under `.cache/`.

## Profiling

Profiling helpers use samply under `scripts/profiling/`. `profile_bench.sh` builds a benchmark binary, records a profile, and can open it with `SAMPLY_VIEW=1`; `profile_cli.sh` and `profile_large_corpus.sh` do the same for the CLI and corpus fixtures.

```sh
make profile-parser PROFILE_CASE=nvm
make profile-formatter-view PROFILE_CASE=nvm
make profile-cli PROFILE_FILE=script.sh
make profile-large-corpus
```

The scripts build with `--profile profiling`, which should preserve symbols without LTO. The workspace currently does not define that profile in `Cargo.toml`, so these targets fail until it is restored.

## Tiered entrypoint

`make bench` defaults to the Change ceiling; it never selects Full implicitly.
`make bench-list` shows every workload and whether the current selection includes it.
`make bench-check` validates the same selection and checks prerequisites and host readiness.
All three accept `TIER=Edit|Change|Scenario|Full`, `WORKLOAD=name`, `AREA=name`,
and space-separated `SKIP="name ..."`. A workload selector names a workload, not an
area. Unknown selectors and empty selections fail. `make bench DRY_RUN=1` prints
commands without running preflight or creating output. Full requires an explicit
`TIER=Full` and is reserved for a manual request.

```sh
make bench-list TIER=Scenario AREA=parser
make bench-check TIER=Change WORKLOAD=parser
make bench TIER=Edit
make bench TIER=Change WORKLOAD=parser
make bench TIER=Scenario WORKLOAD=corpus-parser
```

The tier is a ceiling. Edit runs `driver-tests`, the selection and busy-mode
contract tests (seconds estimated). Change includes one short `fzf-install` case
for each fixture-based rustybench binary (arithmetic uses `double-paren-command`,
and case patterns uses `literal/equal`): `parser`, `lexer`, `lexer_hot_path`, `arithmetic`,
`case_patterns`, `indexer`, `semantic`, `editor`, `unused_assignment`, `linter`,
`formatter`, and `check_command` (minutes per workload estimated, plus release
compilation). Scenario names those same binaries with `corpus-` prepended and runs
all their cases (tens of minutes estimated). Full adds `large-corpus-hotspots`,
which needs the separately installed large corpus; its cost is unmeasured. Every
non-Edit workload invokes `cargo bench -p shuck-benchmark --bench <binary>`;
Change adds its named case filter and Scenario uses all cases. Ready-host runs
use five samples of one iteration. Cargo rebuilds stale artifacts. The preflight
requires the pinned Rust toolchain, `perf-gate-ready`, the manifest's vendored
fixture sizes and at least 1 GiB free. It checks host readiness before execution;
on a busy host it runs rustybench's `--test` once per selected case without timing,
and reports timing deferred. `bench-check` returns success for a busy host because
readiness is reported separately from missing prerequisites. A ready timed run is
checked again with `perf-gate-ready -1`; a failed postcheck invalidates its timing.
Each invocation writes logs and a summary under `target/bench-runs/<date>/`.
Successful completion is not a performance verdict: there is no established
budget or interleaved accepted baseline in this driver.

The outcomes served by these workloads are parsing and indexing source, semantic
analysis and editor queries, linting, formatting, and checking files through the
CLI library path. The fixture manifest records upstream revisions and sizes. The
benchmarks primarily measure in-process work on those files, with throughput
counters; `check_command` prepares paths outside its timed region. The macro and
memory commands above cover other paths but are not part of these tiers. Samply
profiling is separate from timed runs and currently needs the missing profiling
profile restored as described above. Neither fixture size checks nor completion
alone establishes output correctness, workload-tree memory, or a regression gate.
