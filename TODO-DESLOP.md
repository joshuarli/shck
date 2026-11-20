Prefix: DSLP
Next: 7

Deslop campaign backlog ([plan](https://github.com/laputa-systems/stuff/blob/main/docs/campaigns/deslop.md), tier D, by component). One agent works this file with the `deslop` skill: take the lowest-numbered open ticket whose area does not start with `Needs owner ·`. After DSLP-6 (Bench), work the open P1 and P2 findings; file P3 and P4 findings without fixing them. Then fill in `## Report` below.

## Report

Ground baseline at `2cacbda63ff32f45e0c6bfd99a25648faee9915f` (macOS aarch64, nightly-2026-09-15, `TMPDIR=$PWD/.work/tmp`): initial `make test` 214.39 s; warm `make test` 14.88 s, 5,944 passed and 18 ignored. Initial `make check` 164.18 s and warm retry 0.45 s both exited 2: Clippy passed, then Nix was unavailable for `cargo shear`; `check-scripts` did not run through that target. Independent `make check-scripts` passed.

CI now runs `make test` for pushes and pull requests to main on `ubuntu-latest`, with the release workflow's pinned checkout and Rust cache actions. Linux CI has not run here. `gha-test-workflow` accepts this shape. Benchmark fixture upstream revisions are in `crates/shuck-benchmark/resources/manifest.json`; performance workload qualification remains DSLP-6.

Open fleet finding: `rust-toolchain` expects nightly-2026-11-09 while the workspace pins nightly-2026-09-15; retain the workspace pin. Seeded fleet findings remain unverified here: rust-collections 118, audit-upgrades 22, x86-only 4, mdtool 4, gha-ubuntu 4. Large-corpus and Full benchmark tiers were not run.

## Tickets


### DSLP-2 · P2 · Diagnose: ultrareview into this backlog

- **Area:** deslop · diagnose
- **Where:** whole repository
- **Problem:** No review has ranked this repository's risks for the deslop pass.
- **Recommendation:** Run the deslop Diagnose step over the whole repository. Accept: the diff touches only docs, comments, and this file, and every finding names its Fix with.
- **Fix with:** deslop
- **Effort:** M

### DSLP-3 · P2 · Types: strict profile and typing findings

- **Area:** deslop · types
- **Where:** clippy and rustc lint configuration; Rust sources, per component
- **Problem:** The strict-profile candidates in the deslop skill's tools reference have not been measured here. The parser already warns on `clippy::unwrap_used` outside tests.
- **Recommendation:** Run the deslop Types step separately for each AGENTS.md Verification component. Contracts and affected consumers are the component's public Rust API and downstream workspace crates, plus CLI and formats where applicable; see Verification. Accept each slice with its `cargo test -p <crate>` and `make test`, clean at the adopted profile, no increase in escape hatches, and witnesses recorded.
- **Fix with:** deslop
- **Effort:** L

### DSLP-4 · P2 · Tests: coverage, boundary gaps, mutation

- **Area:** deslop · tests
- **Where:** test suites and CI, per component
- **Problem:** Coverage and mutation results are unmeasured; no deslop mutation lane exists.
- **Recommendation:** Run the deslop Tests step separately for each AGENTS.md Verification component. Contracts and consumers are the component's public Rust API and downstream workspace crates, plus CLI and formats where applicable; see Verification. Accept each slice with its `cargo test -p <crate>` and `make test`, no undispositioned survivor in critical code, the test cost budget, and a green lane where tools are mirrored.
- **Fix with:** deslop
- **Effort:** L

### DSLP-5 · P2 · Dead code: tools and debloat

- **Area:** deslop · dead code
- **Where:** whole repository, per component
- **Problem:** No dead-code scan has run since the fork's own removals.
- **Recommendation:** Run the deslop Dead code step separately for each AGENTS.md Verification component. Contracts and consumers are the component's public Rust API and downstream workspace crates, plus CLI and formats where applicable; see Verification. Accept each slice with its `cargo test -p <crate>` and `make test`, tools clean or allowlisted with reasons, and walk-backs filed as Needs owner tickets.
- **Fix with:** deslop
- **Effort:** L

### DSLP-6 · P2 · Bench: tiered benchmark entrypoint

- **Area:** deslop · bench
- **Where:** crates/shuck-benchmark, Makefile, docs/benchmarks.md
- **Problem:** The rustybench harness exists, but the bench-pyramid entrypoint and its outcome list have not been established (campaign decision 14).
- **Recommendation:** Run the bench-pyramid skill as the deslop Bench step on the existing harness. Never run the Full tier. Accept: `make bench`, `bench-check` and `bench-list` work with tier and workload selectors and a preflight; gaps are filed as tickets ranked by the risk of a missed regression.
- **Fix with:** bench-pyramid
- **Effort:** M
