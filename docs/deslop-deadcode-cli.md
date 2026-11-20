# CLI and benchmark dead-code audit

Scope: `shuck-cli` and `shuck-benchmark`, base `3e0a4e8f71ae7b1922e69b992ade7ec0964229d7`, pinned nightly on aarch64 macOS. CLI subcommands, flags, output formats, exit codes and Makefile targets remain unchanged. The compiler checks cover all declared features and targets on the host; Windows-specific code was inspected but not compiled or run.

## Removals

For each item below, identifier and entry-point searches covered the workspace, Makefile, `scripts/`, `benches/`, `.github/workflows/`, and `fuzz/`, including manifests and registrations. Consumer tests are the package tests, workspace tests, Clippy, all-target checks and `make bench-list` recorded below. Fuzz was searched but not run.

- The plugin-resolver argument threaded through private CLI check analysis functions. Both file runner call sites and both stdin call sites passed `None`; the test calls did too. `AnalysisRequest::from_parse_result` initializes this resolver to `None`, so omitting the setter preserves the value. The CLI rejects both former zsh plugin-resolution flags. The file and stdin check entry points and the separate source-path resolver remain. No external registration or feature or platform gate uses this argument.
- The literal-false match arm in each benchmark `large_corpus_hotspots` and `large_corpus_profile` shell classifier. The guard can never match; the remaining dialect arms are unchanged. The bench is registered in the manifest, Makefile and `benches/driver.py`; the example is registered in the manifest and invoked by `scripts/profiling/profile_large_corpus.sh`. Both require `large-corpus-hotspots`; neither removed arm was platform-dependent.

The compiler also found 15 unreachable public declarations in the private `shellcheck_compat::{config,optional,render}` modules. Their visibility is now restricted to the parent module; callers and compatibility command behavior remain intact. These modules are private and none of these declarations was re-exported.

Cargo identified `dhat` and `tikv-jemallocator` as unused normal dependencies: the former is used only by the `dhat-heap` profiling example, the latter by the benchmark allocator macro expanded by bench targets. They are now development dependencies; the existing feature still gates the profiling allocator. All benchmark registrations and invocations were checked in the same consumer search. The feature and target matrix is covered by the all-target, all-feature checks on macOS; Windows was not run.

## Audit

`cargo-machete` and `cargo-udeps` are not installed; dependency usage was checked against source and targets manually. The remaining benchmark dependencies have source, example, test, or bench consumers. The Cargo warnings for the CLI's explicit README and homepage are allowlisted: they preserve package metadata (two entries). No CLI subcommand, flag, format, exit code, or Makefile target was identified as a removal candidate. The broader structural debloat sweep and external consumers were not established by this scoped audit.

## Validation

All commands used `TMPDIR="$PWD/.work/tmp"` against this worktree. `cargo test -p shuck-cli -p shuck-benchmark`, `make test`, `cargo clippy --workspace --all-targets -- -D warnings`, and `make bench-list` exited 0. The workspace test also passed under `cpu-guard --cores 4`. `RUSTFLAGS='-W unused -W unreachable_pub' cargo check -p shuck-cli -p shuck-benchmark --all-targets --all-features` and the same command with `--force-warn dead_code` appended to `RUSTFLAGS` exited 0; neither reported a scoped source warning or benchmark unused-dependency warning. Warnings in other crates are outside this scope. Clippy emitted no benchmark unused-dependency warning.
