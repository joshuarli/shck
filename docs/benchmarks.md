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
