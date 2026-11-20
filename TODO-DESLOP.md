Prefix: DSLP
Next: 14

Deslop campaign backlog ([plan](https://github.com/laputa-systems/stuff/blob/main/docs/campaigns/deslop.md), tier D, by component). One agent works this file with the `deslop` skill: take the lowest-numbered open ticket whose area does not start with `Needs owner ·`. After DSLP-6 (Bench), work the open P1 and P2 findings; file P3 and P4 findings without fixing them. Then fill in `## Report` below.

## Report

Ground baseline at `2cacbda63ff32f45e0c6bfd99a25648faee9915f` (macOS aarch64, nightly-2026-09-15, `TMPDIR=$PWD/.work/tmp`): initial `make test` 214.39 s; warm `make test` 14.88 s, 5,944 passed and 18 ignored. Initial `make check` 164.18 s and warm retry 0.45 s both exited 2: Clippy passed, then Nix was unavailable for `cargo shear`; `check-scripts` did not run through that target. Independent `make check-scripts` passed.

CI now runs `make test` for pushes and pull requests to main on `ubuntu-latest`, with the release workflow's pinned checkout and Rust cache actions. Linux CI has not run here. `gha-test-workflow` accepts this shape. Benchmark fixture upstream revisions are in `crates/shuck-benchmark/resources/manifest.json`; performance workload qualification remains DSLP-6.

Open fleet finding: `rust-toolchain` expects nightly-2026-11-09 while the workspace pins nightly-2026-09-15; retain the workspace pin. Seeded fleet findings remain unverified here: rust-collections 118, audit-upgrades 22, x86-only 4, mdtool 4, gha-ubuntu 4. Large-corpus and Full benchmark tiers were not run.

Diagnose at `cc6d37ce` (macOS aarch64, `TMPDIR=$PWD/.work/tmp`) covered the 12 workspace crates through the ultrareview lenses, weighted to what a user hits: parser robustness, formatter semantics and idempotence, `--fix`, suppressions and config, cache staleness, `shuck run`, and exit codes. Builds: `cargo build -p shuck-cli` 2m36s and `--release` 3m18s. Evidence: 1600 randomized and mutated scripts through `check` with no panic and no hang; 32 construct fixtures through `format` in default plus nine option sets with no idempotence failure and no `bash -n` regression (none has a heredoc inside a compound, which DSLP-7 covers); 44 of the 45 safe-fix rule examples removed their diagnostic under `check --fix` (C103 withholds its fix for `-exec` arguments, as its tests intend); suppression, exit-code, and tar-extraction probes behaved as documented. Filed 7 findings: P1 DSLP-7, P1 DSLP-8, P2 DSLP-9, P2 DSLP-10, P3 DSLP-11, P3 DSLP-12, P4 DSLP-13. Not covered: watch mode end to end, `shuck install` network downloads, the full ShellCheck-compat flag surface, large-corpus and Full benchmark tiers, and the Nix-dependent `cargo shear`.

## Tickets


### DSLP-7 · P1 · format --minify writes invalid shell for heredocs inside compounds

- **Area:** deslop · diagnose · formatter
- **Where:** crates/shuck-formatter/src/streaming/statements.rs:212 (flush only at sequence end) and :288 (per-statement heredoc queue); crates/shuck-formatter/src/lib.rs:155 format_file_ast_resolved (rendered output is never re-parsed); crates/shuck-cli/src/commands/format.rs:302 write path
- **Problem:** shuck format --minify on a heredoc inside if, case, or a function exits 0 and overwrites the file with unparsable shell. Repro: printf '#!/bin/bash\nif true; then\ncat <<EOF\nbody\nEOF\nfi\necho after\n' > t.sh; target/release/shuck format --minify t.sh exits 0 and leaves 'if true; then cat <<EOF' / 'body' / 'EOF; fi; echo after'; bash -n t.sh exits 2 with 'here-document ... delimited by end-of-file'. A case body becomes 'cat <<EOF; case a in a) echo x' / 'body' / 'EOF; ;; esac'. Default and --simplify formatting keep these inputs valid; the only minify fixture (oracle-fixtures/minify.sh) has no heredoc.
- **Recommendation:** Flush pending heredocs before writing any compound close or statement separator under minify/compact layout, then re-parse the rendered output before writing and fail as an internal error (or fall back to non-minified output) when it does not parse; add format_cases fixtures for a heredoc inside if, case, and a function under minify.
- **Fix with:** refactor-rust
- **Effort:** M

### DSLP-8 · P1 · check cache keeps stale diagnostics when a transitively sourced helper changes

- **Area:** deslop · diagnose · cache
- **Where:** crates/shuck-cli/src/commands/check/analyze.rs:215 collect_source_paths (records direct source refs only) and :152 dependency_paths; crates/shuck-cli/src/commands/check/run.rs:113 dependency validation; crates/shuck-semantic source closure; documented gap docs/contracts.md:160
- **Problem:** A file's cached diagnostics survive a change to a helper reached only through another sourced file, so check reports stale results while --no-cache reports the truth. Repro in a directory with b.sh ('b_var=1'), a.sh ('# shuck: source=b.sh lint=true' + 'source ./b.sh'), main.sh ('# shuck: source=a.sh lint=true' + 'source ./a.sh' + 'echo "$b_var"'): target/release/shuck check --cache-dir ./cache main.sh exits 0; change b.sh to 'other=1'; the cached rerun still reports no C006 for main.sh, while shuck check --no-cache main.sh reports error[C006] 'variable `b_var` is referenced before assignment'. Editing a.sh (the direct source) does invalidate main.sh, so only the transitive edge is missing.
- **Recommendation:** Include every resolved path in the transitive source closure in the cache dependency fingerprints for the referring file (and in watch targets), invalidating on any of them; add a regression test with a three-file chain that edits the leaf.
- **Fix with:** refactor-rust
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


### DSLP-9 · P2 · parse-diagnostic enrichment is quadratic in diagnostics times lines

- **Area:** deslop · diagnose · linter performance
- **Where:** crates/shuck-linter/src/parse_diagnostics.rs:1469 if_missing_then_span (upward line scan per diagnostic) and :1691 is_more_specific_parse_diagnostic (repeats the scan per stray-keyword candidate)
- **Problem:** Analysis time grows quadratically with the number of parse diagnostics because each one scans every preceding line. Repro with the release binary: python3 -c "open('then.sh','w').write('then\n'*32000)" (160 KB) then target/release/shuck check --no-cache then.sh takes 18.04 s, and 8000 lines takes 1.11 s (4x input, 16x time). A 272 KB file of 16000 nested ifs takes 27.73 s; 68 KB takes 1.78 s and 136 KB takes 7.77 s. A macOS sample of the run attributes most samples to if_missing_then_span and line_contains_shell_word. Inputs of a few hundred KB therefore hang check for tens of seconds.
- **Recommendation:** Precompute per-line enclosing-if/then state once (the pattern compute_pending_until_flags already uses) instead of scanning upward per diagnostic, or bound the scan by the diagnostic's own statement; add a timing regression case over an error-dense fixture.
- **Fix with:** refactor-rust
- **Effort:** M


### DSLP-10 · P2 · nesting past the parser depth limit floods false C016 errors on valid scripts

- **Area:** deslop · diagnose · parser diagnostics
- **Where:** crates/shuck-parser/src/parser/limits.rs:35 and :137 (depth limit 100); crates/shuck-linter/src/parse_diagnostics.rs:1662 stray_closing_keyword_spans and :1499 if_missing_then_span; docs/architecture.md:40 claims exhausted limits produce a fatal result
- **Problem:** A valid script nested deeper than 100 control blocks produces one false C016 'this control keyword has no matching opener' per recovered line and never reports that the nesting limit was hit. Repro: python3 -c "open('if150.sh','w').write('if true; then\n'*150+':\n'+'fi\n'*150)"; bash -n if150.sh exits 0; target/release/shuck check --no-cache if150.sh exits 1 with 102 error[C016] diagnostics, the first pointing at the 'then' on line 101. 99 nested ifs produce 0 diagnostics and 101 produce 102; while and for nesting behave the same, and the same happens on a .bash file. No diagnostic mentions the depth limit.
- **Recommendation:** Surface the recorded resource-limit error (for example 'AST nesting too deep') and suppress the recovery-derived parse-rule diagnostics that follow from it; do not treat 'then'/'do' as stray closing keywords when the opener was dropped by the depth cap.
- **Fix with:** refactor-rust
- **Effort:** M


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


### DSLP-13 · P4 · C098 spec rationale contradicts its safe fix

- **Area:** deslop · diagnose · rule metadata
- **Where:** docs/rules/C098.yaml:9-10 (rationale and fix_description); crates/shuck-linter/src/rules/correctness/set_flags_without_dashes.rs:45
- **Problem:** The rule rationale tells the reader to 'Add a dash before the flags: set -euo pipefail', but the safe fix inserts '--' instead: 'set euox pipefail' becomes 'set -- euox pipefail', which keeps the words as positional parameters and never enables the options. The fixed script therefore still does not have the behavior the rationale names.
- **Recommendation:** Reword the rationale and description to state that the fix only makes the bare operands explicit positional parameters, and keep the flags interpretation in the message only.
- **Fix with:** refactor-rust
- **Effort:** S
