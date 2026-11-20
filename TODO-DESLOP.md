Prefix: DSLP
Next: 7

Deslop campaign backlog ([plan](https://github.com/laputa-systems/stuff/blob/main/docs/campaigns/deslop.md), tier D, by component). One agent works this file with the `deslop` skill: take the lowest-numbered open ticket whose area does not start with `Needs owner ·`. After DSLP-6 (Bench), work the open P1 and P2 findings; file P3 and P4 findings without fixing them. Then fill in `## Report` below.

## Report

Not started.

## Tickets

### DSLP-1 · P2 · Ground: baseline, boundaries, components, and CI test entry point

- **Area:** deslop · ground
- **Where:** AGENTS.md, Makefile, .github/workflows/
- **Problem:** Base 90c560d; Rust 337k physical lines across 12 crates plus fuzz (crates/shuck-linter 172k, shuck-semantic 45k, shuck-parser 35k, shuck-formatter 33k, shuck-cli 25k, shuck-ast 6.9k, shuck-config 4.8k, shuck-indexer 4.4k, shuck-run 4.1k, shuck-benchmark 3.8k, shuck-discover 1.5k, shuck-cache 0.8k, fuzz 0.7k); timings not yet measured. Hard fork of ewhauser/shuck (18 local commits, 154k lines already removed); ShellCheck and shfmt are black-box oracles only, under the clean-room policy in docs/development.md. CI has only release.yml; no workflow runs `make test`. Open fleet lint findings: rust-collections 118, audit-upgrades 22, x86-only 4, mdtool 4, gha-ubuntu 4, rust-toolchain 1. Perf gate applies: rustybench harness in crates/shuck-benchmark.
- **Recommendation:** Run the deslop Ground step. Over 50k lines, so Types, Tests and Dead code run per component: list the components (a crate or a subsystem of at most about 20k lines; split shuck-linter, shuck-semantic, shuck-parser, shuck-formatter and shuck-cli; group the small crates) in AGENTS.md or this ticket's close note. Accept: CI runs make test and is green, and AGENTS.md has a Verification section.
- **Fix with:** deslop
- **Effort:** M

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
- **Recommendation:** Run the deslop Types step, one PR per component. Accept: clean at the adopted profile, escape hatches not increased, witnesses recorded.
- **Fix with:** deslop
- **Effort:** L

### DSLP-4 · P2 · Tests: coverage, boundary gaps, mutation

- **Area:** deslop · tests
- **Where:** test suites and CI, per component
- **Problem:** Coverage and mutation results are unmeasured; no deslop mutation lane exists.
- **Recommendation:** Run the deslop Tests step, one PR per component. Accept: no undispositioned survivor in critical code, the test cost budget holds, and the lane is green where the tools are mirrored.
- **Fix with:** deslop
- **Effort:** L

### DSLP-5 · P2 · Dead code: tools and debloat

- **Area:** deslop · dead code
- **Where:** whole repository, per component
- **Problem:** No dead-code scan has run since the fork's own removals.
- **Recommendation:** Run the deslop Dead code step, one PR per component. Accept: tools clean or allowlisted with reasons; walk-backs filed as Needs owner tickets.
- **Fix with:** deslop
- **Effort:** L

### DSLP-6 · P2 · Bench: tiered benchmark entrypoint

- **Area:** deslop · bench
- **Where:** crates/shuck-benchmark, Makefile, docs/benchmarks.md
- **Problem:** The rustybench harness exists, but the bench-pyramid entrypoint and its outcome list have not been established (campaign decision 14).
- **Recommendation:** Run the bench-pyramid skill as the deslop Bench step on the existing harness. Never run the Full tier. Accept: `make bench`, `bench-check` and `bench-list` work with tier and workload selectors and a preflight; gaps are filed as tickets ranked by the risk of a missed regression.
- **Fix with:** bench-pyramid
- **Effort:** M
