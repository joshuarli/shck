Prefix: DSLP
Next: 16

Deslop campaign backlog ([plan](https://github.com/laputa-systems/stuff/blob/main/docs/campaigns/deslop.md), tier D, by component). One agent works this file with the `deslop` skill: take the lowest-numbered open ticket whose area does not start with `Needs owner ·`. After DSLP-6 (Bench), work the open P1 and P2 findings; file P3 and P4 findings without fixing them. Then fill in `## Report` below.

## Report

Ground baseline at `2cacbda63ff32f45e0c6bfd99a25648faee9915f` (macOS aarch64, nightly-2026-09-15, `TMPDIR=$PWD/.work/tmp`): initial `make test` 214.39 s; warm `make test` 14.88 s, 5,944 passed and 18 ignored. Initial `make check` 164.18 s and warm retry 0.45 s both exited 2: Clippy passed, then Nix was unavailable for `cargo shear`; `check-scripts` did not run through that target. Independent `make check-scripts` passed.

CI now runs `make test` for pushes and pull requests to main on `ubuntu-latest`, with the release workflow's pinned checkout and Rust cache actions. Linux CI has not run here. `gha-test-workflow` accepts this shape. Benchmark fixture upstream revisions are in `crates/shuck-benchmark/resources/manifest.json`; performance workload qualification remains DSLP-6.

Open fleet finding: `rust-toolchain` expects nightly-2026-11-09 while the workspace pins nightly-2026-09-15; retain the workspace pin. Seeded fleet findings remain unverified here: rust-collections 118, audit-upgrades 22, x86-only 4, mdtool 4, gha-ubuntu 4. Large-corpus and Full benchmark tiers were not run.

Diagnose at `cc6d37ce` (macOS aarch64, `TMPDIR=$PWD/.work/tmp`) covered the 12 workspace crates through the ultrareview lenses, weighted to what a user hits: parser robustness, formatter semantics and idempotence, `--fix`, suppressions and config, cache staleness, `shuck run`, and exit codes. Builds: `cargo build -p shuck-cli` 2m36s and `--release` 3m18s. Evidence: 1600 randomized and mutated scripts through `check` with no panic and no hang; 32 construct fixtures through `format` in default plus nine option sets with no idempotence failure and no `bash -n` regression (none has a heredoc inside a compound, which DSLP-7 covers); 44 of the 45 safe-fix rule examples removed their diagnostic under `check --fix` (C103 withholds its fix for `-exec` arguments, as its tests intend); suppression, exit-code, and tar-extraction probes behaved as documented. Filed 7 findings: P1 DSLP-7, P1 DSLP-8, P2 DSLP-9, P2 DSLP-10, P3 DSLP-11, P3 DSLP-12, P4 DSLP-13. Not covered: watch mode end to end, `shuck install` network downloads, the full ShellCheck-compat flag surface, large-corpus and Full benchmark tiers, and the Nix-dependent `cargo shear`.

DSLP-7 closed: minified heredocs now end before compound separators, with regression coverage and a pre-write parse check.

DSLP-9 closed: parse-diagnostic enrichment precomputes per-line state; existing fixture diagnostics match the baseline.

DSLP-10 closed: parser depth-limit errors survive recovery and are reported without cascading closing-keyword diagnostics.

## Tickets

### DSLP-3 · P2 · Types: strict profile and typing findings

- **Area:** deslop · types
- **Where:** clippy and rustc lint configuration; Rust sources, per component
- **Problem:** The workspace census is complete, but the selected strict profile and typing findings still need component-by-component application. The parser already warns on `clippy::unwrap_used` outside tests. The workspace candidate census and first slice (shuck-ast + shuck-indexer) are recorded in `docs/deslop-types.md`. Remaining: all linter, semantic, parser, formatter, and CLI components; shuck-config + shuck-discover + shuck-cache; shuck-run + shuck-benchmark + fuzz.
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

### DSLP-15 · P2 · default check output rebuilds a line index per diagnostic

- **Area:** deslop · linter performance
- **Where:** crates/shuck-cli/src/commands/check_output.rs:1098 (full renderer)
- **Problem:** After DSLP-9 made parse-diagnostic enrichment linear, `shuck check --no-cache` with the default full output still scales quadratically because the renderer rebuilds a line index for each diagnostic. Release medians on a loaded host: 4x more `then` lines (8,000 to 32,000) took 14.0x longer and 4x more nested ifs took 12.2x longer with default output, while `--output-format concise` grew 2.5x and 2.1x. Error-dense inputs of a few hundred KB still take seconds to render.
- **Recommendation:** Build the line index once per file and share it across that file's diagnostics in the full renderer; confirm byte-identical output on the linter fixtures and measure the 4x scaling ratio again.
- **Fix with:** refactor-rust
- **Effort:** S

### DSLP-11 · P3 · config files silently ignore unknown keys that --config rejects

- **Area:** deslop · diagnose · config
- **Where:** crates/shuck-config/src/lib.rs:103 ShuckConfig, :127 FormatConfig, :243 LintConfig (all without deny_unknown_fields) versus :1800 parse_config_override which validates strictly
- **Problem:** A mistyped key in shuck.toml is accepted and ignored, so the intended settings never apply, while the same key through --config is an error. Repro: a script that reports C006 with defaults; with '[lint] select = ["C001"]' C006 is suppressed; with the typo '[lint] selects = ["C001"]' the default rule set runs again (C006 reported) and check exits 0. A '[bogus]' table is likewise accepted. The nested section structs already use deny_unknown_fields, so the behavior is inconsistent within one file format.
- **Recommendation:** Add deny_unknown_fields to the root and section structs, or emit a warning that names the ignored key; add a test that a mistyped [lint] key fails or warns.
- **Fix with:** refactor-rust
- **Effort:** S

### DSLP-12 · P3 · zsh leftovers advertise shells and config keys the code rejects or ignores

- **Area:** deslop · diagnose · config and help text
- **Where:** crates/shuck-cli/src/cli/run.rs:15, install.rs:21, shell.rs:12 (help lists zsh); crates/shuck-config/src/lib.rs:1318-1370 ([lint.zsh]) and :1414 ([run.shells].zsh); crates/shuck-cli/src/cli/check.rs:611 empty zsh_plugin_flags_removed test
- **Problem:** After zsh support was removed the public surfaces still offer it. 'shuck run --shell zsh' errors 'invalid managed shell zsh; expected one of: bash, gbash, bashkit, dash, mksh, busybox' while the help text for run, install, and shell lists zsh. configuration_metadata(), which generates the published settings reference, still documents [lint.zsh.plugins] (silently ignored by check) and [run.shells].zsh (rejected by shuck run but accepted by shuck check). The zsh_plugin_flags_removed test has an empty body and asserts nothing.
- **Recommendation:** Remove zsh from the three help strings and from configuration_metadata, or restore it end to end; validate [run] the same way in check and run; replace the empty test with one that asserts --shell zsh is rejected.
- **Fix with:** refactor-rust
- **Effort:** S

### DSLP-14 · P3 · Inputs over 4 GiB panic instead of failing with an error

- **Area:** deslop · types
- **Where:** crates/shuck-ast/src/span.rs (TextSize::from_usize, Position::advance); CLI input reading
- **Problem:** Source positions are packed in u32. The first Types slice made the conversion checked, so an input whose offsets, lines or columns exceed u32::MAX now panics where it previously wrapped and reported wrong spans. Not reproduced: it needs an input over 4 GiB.
- **Recommendation:** Reject inputs longer than u32::MAX bytes at the CLI and library entry points with an ordinary error before parsing, so the checked conversion is never reached by user input.
- **Fix with:** deslop
- **Effort:** S

### DSLP-13 · P4 · C098 spec rationale contradicts its safe fix

- **Area:** deslop · diagnose · rule metadata
- **Where:** docs/rules/C098.yaml:9-10 (rationale and fix_description); crates/shuck-linter/src/rules/correctness/set_flags_without_dashes.rs:45
- **Problem:** The rule rationale tells the reader to 'Add a dash before the flags: set -euo pipefail', but the safe fix inserts '--' instead: 'set euox pipefail' becomes 'set -- euox pipefail', which keeps the words as positional parameters and never enables the options. The fixed script therefore still does not have the behavior the rationale names.
- **Recommendation:** Reword the rationale and description to state that the fix only makes the bare operands explicit positional parameters, and keep the flags interpretation in the message only.
- **Fix with:** refactor-rust
- **Effort:** S

