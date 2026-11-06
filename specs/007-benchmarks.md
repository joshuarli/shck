# 007: Benchmarks

## Status

Proposed

## Summary

A benchmarking setup for shuck that measures parse, semantic, and lint throughput on real-world shell scripts and compares end-to-end CLI performance against shellcheck. The design follows ruff's two-tier approach: **rustybench micro-benchmarks** in a dedicated `shuck-benchmark` crate for measuring individual components (lexer, parser, semantic analysis, linter), and **hyperfine macro-benchmarks** via shell scripts for CLI-vs-shellcheck wall-time comparison. Both tiers operate on the same 5 vendored benchmark fixtures used by the Go frontend (`shuck`), copied directly from `shuck/testdata/benchmarks/`.

## Motivation

The CHECKLIST.md item 12.3 calls for "benchmark cases for parse throughput and memory use." Today we have correctness tests (large corpus conformance against shellcheck) but no performance measurement. Without benchmarks we can't:

- **Detect regressions** — A parser refactor could halve throughput and we wouldn't notice until users report it.
- **Quantify our advantage** — Shuck's value proposition includes speed. We need numbers to back this up, comparing against shellcheck on the same scripts.
- **Guide optimization** — Profiling without repeatable benchmarks is guesswork. Rustybench's sample statistics and baseline comparison give us actionable data.

Ruff's benchmark infrastructure (dedicated `ruff_benchmark` crate, micro-benchmarks for individual components, hyperfine for macro-benchmarks) is battle-tested and maps cleanly to our problem. We adapt it for shell scripts rather than Python files.

## Design

### Benchmark Corpus

We use the same 5 vendored benchmark fixtures from the Go frontend (`shuck/testdata/benchmarks/`), copied into `crates/shuck-benchmark/resources/`. These files are committed to the repo so benchmarks run without network access or corpus download steps. Using the identical fixture set ensures benchmark numbers are directly comparable between the Go and Rust implementations.

Each fixture is pinned to a specific upstream commit and includes its license file, following the Go frontend's `manifest.json` convention:

| File | Source | Commit | Lines | Bytes | License |
|------|--------|--------|-------|-------|---------|
| `homebrew-install.sh` | Homebrew/install `install.sh` | `6d5e267` | 1,175 | 33,212 | BSD-2-Clause |
| `nvm.sh` | nvm-sh/nvm `nvm.sh` | `977563e` | 4,661 | 150,227 | MIT |
| `pyenv-python-build.sh` | pyenv/pyenv `plugins/python-build/bin/python-build` | `8397a19` | 2,740 | 81,725 | MIT |
| `ruby-build.sh` | rbenv/ruby-build `bin/ruby-build` | `d099da0` | 1,643 | 47,738 | MIT |
| `fzf-install.sh` | junegunn/fzf `install` | `55d5b15` | 452 | 12,760 | MIT |

Total: 10,671 lines, 325,662 bytes.

The Go frontend also runs an "all" benchmark case that passes all 5 files in a single invocation. We replicate this in both the micro- and hyperfine benchmarks.

### Crate: `shuck-benchmark`

A new workspace member crate, following ruff's `ruff_benchmark` structure:

```
crates/shuck-benchmark/
├── Cargo.toml
├── benches/
│   ├── lexer.rs          # rustybench: lex throughput per file
│   ├── parser.rs         # rustybench: parse throughput per file
│   ├── semantic.rs       # rustybench: semantic-model throughput per file
│   ├── unused_assignment.rs # rustybench: exact variable-dataflow throughput per file
│   └── linter.rs         # rustybench: full lint pipeline per file
├── src/
│   └── lib.rs            # TestCase, TestFile, load helpers
└── resources/
    ├── README.md          # Source, commit, license for each fixture
    ├── manifest.json      # Fixture metadata (matches Go frontend format)
    ├── files/
    │   ├── homebrew-install.sh
    │   ├── nvm.sh
    │   ├── pyenv-python-build.sh
    │   ├── ruby-build.sh
    │   └── fzf-install.sh
    └── licenses/
        ├── homebrew-install-LICENSE.txt
        ├── nvm-LICENSE.md
        ├── pyenv-LICENSE
        ├── ruby-build-LICENSE
        └── fzf-LICENSE
```

The `resources/` directory structure mirrors `shuck/testdata/benchmarks/` exactly, so files can be copied directly without renaming.

#### Cargo.toml

```toml
[package]
name = "shuck-benchmark"
version = "0.1.0"
edition = "2024"
publish = false

[dependencies]
shuck-parser = { path = "../shuck-parser" }

[dev-dependencies]
rustybench = "0.4"

[[bench]]
name = "lexer"
harness = false

[[bench]]
name = "parser"
harness = false

[[bench]]
name = "semantic"
harness = false

[[bench]]
name = "unused_assignment"
harness = false

[[bench]]
name = "linter"
harness = false
```

`unused_assignment.rs` is the dedicated rustybench harness for exact variable-dataflow work. It keeps the isolated `unused_assignment` and `uninitialized_reference` groups and also provides a combined `variable_dataflow_combined` group that measures the real shared-cache access pattern on one `SemanticAnalysis`.

Before/after exact-dataflow comparisons use explicit rustybench baseline files:

```bash
rustybench baseline --root . --baseline target/exact-dataflow-before.json \
  -- cargo bench -p shuck-benchmark --bench unused_assignment

rustybench baseline --root . --baseline target/exact-dataflow-after.json \
  --quiet -- cargo bench -p shuck-benchmark --bench unused_assignment

rustybench diff target/exact-dataflow-before.json target/exact-dataflow-after.json
```

#### src/lib.rs — Test Case Infrastructure

```rust
use std::path::{Path, PathBuf};

pub struct TestFile {
    pub name: &'static str,
    pub source: &'static str,
}

pub struct TestCase {
    pub name: &'static str,
    pub files: &'static [TestFile],
}

/// All vendored benchmark files — same set as the Go frontend's
/// testdata/benchmarks/manifest.json, ordered smallest to largest.
pub static TEST_FILES: LazyLock<Vec<TestFile>> = LazyLock::new(|| {
    vec![
        TestFile {
            name: "fzf-install",
            source: fixture_source(include_bytes!("../resources/files/fzf-install.sh")),
        },
        // ...
    ]
});

/// Single-file cases plus the aggregate `all` case.
pub fn benchmark_cases() -> &'static [TestCase];

/// Case names passed to `#[rustybench::bench(args = ...)]`.
pub fn benchmark_case_names() -> Vec<&'static str>;

pub fn resources_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("resources")
}
```

#### benches/parser.rs — Parser Micro-Benchmark

```rust
use rustybench::counter::BytesCount;
use rustybench::{Bencher, black_box};
use shuck_benchmark::{benchmark_case, benchmark_case_names, parse_fixture};

#[rustybench::bench(args = benchmark_case_names())]
fn parse(bencher: Bencher, case_name: &str) {
    let case = benchmark_case(case_name);

    bencher
        .counter(BytesCount::from(case.total_bytes()))
        .bench_local(|| {
            let body_len: usize = case
                .files
                .iter()
                .map(|file| parse_fixture(file.source).file.body.len())
                .sum();
            black_box(body_len)
        });
}

fn main() {
    rustybench::main();
}
```

The lexer, semantic, and linter benches follow the same structure, benchmarking `Lexer::new(source).collect::<Vec<_>>()`, `SemanticModel::build(...)`, and the full `AnalysisRequest::from_parse_result(...).lint()` pipeline respectively.

#### Memory Allocator

Following ruff, pin the allocator in benchmark binaries to reduce noise:

```rust
#[cfg(not(target_os = "windows"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
```

Add `tikv-jemallocator` as an optional dev-dependency, gated behind a `jemalloc` feature that defaults to on for non-Windows targets.

### Macro-Benchmarks: Hyperfine Scripts

Shell scripts in `scripts/benchmarks/` for CLI-level comparison against shellcheck. These use the same vendored fixture files — no corpus download required.

#### scripts/benchmarks/setup.sh

Builds shuck and verifies shellcheck is available:

```bash
#!/bin/sh
set -eu
repo_root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)

echo "Building shuck in release mode..."
cargo build --release -p shuck --manifest-path="$repo_root/Cargo.toml"

echo "Verifying shellcheck is installed..."
if ! command -v shellcheck >/dev/null 2>&1; then
    echo "ERROR: shellcheck not found. Install it first."
    exit 1
fi

echo "Setup complete."
echo "  shuck:      $(cargo run --release -p shuck -- --version 2>/dev/null || echo 'built')"
echo "  shellcheck: $(shellcheck --version | head -2 | tail -1)"
```

#### scripts/benchmarks/run.sh

Runs shuck vs shellcheck on each fixture file individually and on all files together, mirroring the Go frontend's `BenchmarkCLI` structure:

```bash
#!/bin/sh
set -eu
repo_root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)
fixtures_dir="$repo_root/crates/shuck-benchmark/resources/files"
shuck="$repo_root/target/release/shuck"

# Per-file benchmarks
for file in "$fixtures_dir"/*.sh; do
    name=$(basename "$file" .sh)
    echo "==> Benchmarking: $name"
    hyperfine \
        --warmup 3 \
        --runs 10 \
        --export-json "$repo_root/.cache/bench-${name}.json" \
        -n "shuck/$name" "$shuck check --no-cache $file" \
        -n "shellcheck/$name" "shellcheck --severity=style $file"
done

# All-files benchmark
echo "==> Benchmarking: all"
all_files=$(echo "$fixtures_dir"/*.sh)
hyperfine \
    --warmup 3 \
    --runs 10 \
    --export-json "$repo_root/.cache/bench-all.json" \
    --export-markdown "$repo_root/.cache/bench-all.md" \
    -n "shuck/all" "$shuck check --no-cache $all_files" \
    -n "shellcheck/all" "shellcheck --severity=style $all_files"
```

Note: The macro benchmark compares shuck against the default shellcheck CLI invocation. The `--no-cache` flag ensures shuck measures cold parse performance.

#### scripts/benchmarks/run_single.sh

Benchmarks a single file for focused comparison:

```bash
#!/bin/sh
set -eu
repo_root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)
shuck="$repo_root/target/release/shuck"
file="${1:?Usage: run_single.sh <path-to-script>}"

hyperfine \
    --warmup 5 \
    --runs 20 \
    -n "shuck" "$shuck check --no-cache $file" \
    -n "shellcheck" "shellcheck --severity=style $file"
```

### Makefile Targets

```makefile
bench:
	cargo bench -p shuck-benchmark

bench-save:
	rustybench baseline --root . --baseline target/bench-baseline.json -- cargo bench -p shuck-benchmark

bench-compare:
	rustybench baseline --root . --baseline target/bench-candidate.json --quiet -- cargo bench -p shuck-benchmark
	rustybench diff target/bench-baseline.json target/bench-candidate.json

bench-parser:
	cargo bench -p shuck-benchmark --bench parser

bench-lexer:
	cargo bench -p shuck-benchmark --bench lexer

bench-macro:
	./scripts/benchmarks/setup.sh
	./scripts/benchmarks/run.sh
```

### Profiling Support

A Cargo profile for benchmark profiling (symbols preserved, no LTO):

```toml
# workspace Cargo.toml
[profile.profiling]
inherits = "release"
strip = false
debug = "full"
lto = false
```

This keeps symbols available to `samply` on benchmark binaries without losing symbol resolution.

### What We Measure

| Tier | Tool | What | Metric |
|------|------|------|--------|
| Micro | rustybench | Lex each fixture | throughput (bytes/s), median time |
| Micro | rustybench | Parse each fixture | throughput (bytes/s), median time |
| Micro | rustybench | Build semantics for each fixture | throughput (bytes/s), median time |
| Micro | rustybench | Lint each fixture | throughput (bytes/s), median time |
| Macro | Hyperfine | `shuck check` per fixture | wall time (mean ± σ), vs shellcheck |
| Macro | Hyperfine | `shuck check` all fixtures | wall time (mean ± σ), vs shellcheck |

### What We Don't Measure (Yet)

- **Memory usage** — Deferred until we have a clear memory regression concern. Can be added via `dhat` or jemalloc stats without changing the benchmark structure.
- **Incremental/cached performance** — The cache layer (shuck-cache) makes re-runs trivially fast. Benchmarking it is less useful than benchmarking cold parse performance.
- **CI integration** — No Codspeed or automated regression detection initially. Can be added once baselines are established and benchmark variance is understood.

## Alternatives Considered

### Alternative A: Micro-Benchmarks Only (No Hyperfine)

Run all benchmarks through rustybench, including CLI-level measurements.

**Rejected because:** rustybench measures library-level function calls, not CLI startup, file discovery, argument parsing, and I/O. Hyperfine captures the full user-facing experience and directly compares against shellcheck's CLI — the comparison users care about. The two tools measure different things and complement each other.

### Alternative B: Hyperfine Only (No Micro-Benchmarks)

Use only CLI-level benchmarks for simplicity.

**Rejected because:** CLI benchmarks conflate I/O, argument parsing, file discovery, and actual analysis. When a regression appears in hyperfine, you need micro-benchmark data to know whether the lexer, parser, or linter is responsible. Ruff's experience shows that micro-benchmarks catch regressions that macro-benchmarks miss due to noise.

### Alternative C: Divan Instead of Rustybench

Use the `divan` crate (simpler API, less boilerplate) for micro-benchmarks.

**Rejected because:** Rustybench provides the sample runner, JSON reports, baseline collection, baseline diff, and budget checks this repository uses, and its `#[rustybench::bench]` attributes keep case registration declarative. Divan does not provide the same baseline workflow.

### Alternative D: Use the Large Corpus (129 Repos) for Benchmarks

Use the `scripts/corpus-download.sh` output instead of the Go frontend's 5 fixtures.

**Rejected because:** The large corpus requires a ~10 minute download step, produces thousands of files of variable quality, and isn't deterministic across runs (repos change). The Go frontend's 5-fixture set is curated, pinned to exact commits, covers a good size range (12 KB to 150 KB), and — critically — lets us compare Rust vs Go benchmark numbers directly since both frontends measure the same files.

## Verification

Once implemented, verify with:

- **Micro-benchmarks run:** `cargo bench -p shuck-benchmark --bench parser` completes and reports per-case timings.
- **All vendored files load:** Each `TestFile` in `TEST_FILES` has non-empty source (the `include_bytes!` calls compile).
- **Throughput reported:** Rustybench output includes a throughput column for each benchmark.
- **Baseline comparison works:** Run `make bench-save`, make a change, then `make bench-compare` shows a per-benchmark diff.
- **Macro-benchmarks run:** `./scripts/benchmarks/run.sh` produces `bench-*.json` files with timing data for shuck and shellcheck.
- **Per-file and all-files:** The macro-benchmark output includes both individual fixture results and the combined "all" benchmark.
- **Single-file comparison:** `./scripts/benchmarks/run_single.sh crates/shuck-benchmark/resources/files/nvm.sh` produces hyperfine output comparing shuck vs shellcheck.
- **Make targets:** `make bench` and `make bench-macro` succeed.
