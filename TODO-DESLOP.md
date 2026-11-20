Prefix: DSLP
Next: 27

Deslop campaign backlog ([plan](https://github.com/laputa-systems/stuff/blob/main/docs/campaigns/deslop.md), tier D, by component). One agent works this file with the `deslop` skill: take the lowest-numbered open ticket whose area does not start with `Needs owner ·`. After DSLP-6 (Bench), work the open P1 and P2 findings; file P3 and P4 findings without fixing them. Then fill in `## Report` below.

## Report

Ground baseline at `2cacbda63ff32f45e0c6bfd99a25648faee9915f` (macOS aarch64, nightly-2026-09-15, `TMPDIR=$PWD/.work/tmp`): initial `make test` 214.39 s; warm `make test` 14.88 s, 5,944 passed and 18 ignored. Initial `make check` 164.18 s and warm retry 0.45 s both exited 2: Clippy passed, then Nix was unavailable for `cargo shear`; `check-scripts` did not run through that target. Independent `make check-scripts` passed.

CI now runs `make test` for pushes and pull requests to main on `ubuntu-latest`, with the release workflow's pinned checkout and Rust cache actions. Linux CI has not run here. `gha-test-workflow` accepts this shape. Benchmark fixture upstream revisions are in `crates/shuck-benchmark/resources/manifest.json`; performance workload qualification remains DSLP-6.

Open fleet finding: `rust-toolchain` expects nightly-2026-11-09 while the workspace pins nightly-2026-09-15; retain the workspace pin. Seeded fleet findings remain unverified here: rust-collections 118, audit-upgrades 22, x86-only 4, mdtool 4, gha-ubuntu 4. Large-corpus and Full benchmark tiers were not run.

Diagnose at `cc6d37ce` (macOS aarch64, `TMPDIR=$PWD/.work/tmp`) covered the 12 workspace crates through the ultrareview lenses, weighted to what a user hits: parser robustness, formatter semantics and idempotence, `--fix`, suppressions and config, cache staleness, `shuck run`, and exit codes. Builds: `cargo build -p shuck-cli` 2m36s and `--release` 3m18s. Evidence: 1600 randomized and mutated scripts through `check` with no panic and no hang; 32 construct fixtures through `format` in default plus nine option sets with no idempotence failure and no `bash -n` regression (none has a heredoc inside a compound, which DSLP-7 covers); 44 of the 45 safe-fix rule examples removed their diagnostic under `check --fix` (C103 withholds its fix for `-exec` arguments, as its tests intend); suppression, exit-code, and tar-extraction probes behaved as documented. Filed 7 findings: P1 DSLP-7, P1 DSLP-8, P2 DSLP-9, P2 DSLP-10, P3 DSLP-11, P3 DSLP-12, P4 DSLP-13. Not covered: watch mode end to end, `shuck install` network downloads, the full ShellCheck-compat flag surface, large-corpus and Full benchmark tiers, and the Nix-dependent `cargo shear`.

DSLP-7 closed: minified heredocs now end before compound separators, with regression coverage and a pre-write parse check.

DSLP-9 closed: parse-diagnostic enrichment precomputes per-line state; existing fixture diagnostics match the baseline.

DSLP-3 closed: every component has a Types slice (#6 to #10, #13, #15, #18, #20); dispositions are in docs/deslop-types.md.

DSLP-10 closed: parser depth-limit errors survive recovery and are reported without cascading closing-keyword diagnostics.

DSLP-15 closed: full output shares a line index across diagnostics for each source.

## Tickets

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

### DSLP-16 · P3 · Bench driver has no regression verdict

- **Area:** deslop · bench
- **Where:** benches/driver.py (run); docs/benchmarks.md
- **Problem:** `make bench` runs tiers and workloads but has no accepted baseline, interleaved comparison or budget, so neither an immediate nor a cumulative slowdown fails it. The existing `bench-save`/`bench-compare` targets are separate and untiered.
- **Recommendation:** Have `make bench-check` compare a candidate against a saved rustybench baseline for the selected tier, interleaved, with a per-workload budget, and fail on an exceeded budget.
- **Fix with:** bench-pyramid
- **Effort:** M

### DSLP-17 · P3 · Parser benchmark and corpus script do not check their results

- **Area:** deslop · bench
- **Where:** crates/shuck-benchmark/benches/parser.rs:9; scripts/benchmarks/run_repo_corpus.sh:246
- **Problem:** The parser benchmark consumes a command count without asserting the expected value, and the repository-corpus script discards output and tolerates failures. A change that does less or wrong work can look faster.
- **Recommendation:** Assert each workload's expected result (command count, diagnostic count or output digest) once per run before timing, and make the corpus script fail on a failed or changed run.
- **Fix with:** bench-pyramid
- **Effort:** S

### DSLP-18 · P3 · Tiered benchmarks omit process startup, whole-repo runs and memory

- **Area:** deslop · bench
- **Where:** docs/benchmarks.md (outcome list); benches/driver.py
- **Problem:** The tiered workloads exercise in-process library paths. Process startup, whole-repository `shuck check` and peak memory stay in separate macro and memory scripts outside the tiers, so a regression there is not seen by `make bench`.
- **Recommendation:** Add a Scenario workload that times the release `shuck check` binary over a fixture tree and records peak RSS, using the existing memory scripts.
- **Fix with:** bench-pyramid
- **Effort:** M

### DSLP-19 · P3 · Benchmark runs do not identify fixture content or revision

- **Area:** deslop · bench
- **Where:** benches/driver.py:59 (preflight) and :107 (summary)
- **Problem:** Preflight checks fixture sizes but not contents, and run summaries record neither the tested revision nor a dirty tree. A same-size fixture change, or an unidentified candidate, can hide or fake a regression.
- **Recommendation:** Verify fixture digests from crates/shuck-benchmark/resources/manifest.json in preflight, and record the commit and dirty state in each summary.
- **Fix with:** bench-pyramid
- **Effort:** S

### DSLP-20 · P3 · Profiling commands need an undefined cargo profile

- **Area:** deslop · bench
- **Where:** docs/benchmarks.md:56; Cargo.toml
- **Problem:** The documented profiling commands use `--profile profiling`, which the workspace does not define, so they fail and regressions are harder to attribute.
- **Recommendation:** Define a `profiling` profile (release with debug symbols) in Cargo.toml, or fix the documented commands.
- **Fix with:** bench-pyramid
- **Effort:** S

### DSLP-21 · P3 · Decide whether unused public indexer and discover APIs stay

- **Area:** Needs owner · deslop · dead code
- **Where:** crates/shuck-indexer/src/selection_range_index.rs:40 (selection_chain); crates/shuck-indexer/src/lib.rs:273 (is_continuation); crates/shuck-discover/src/lib.rs:222 (discover_files)
- **Problem:** These public functions have only in-crate test consumers anywhere in the workspace, including fuzz/. Removing them changes the public API of published crates, so external consumers are unknown.
- **Recommendation:** Owner decides: remove them (and their tests) if the crates have no external consumers, or keep them as supported API.
- **Fix with:** deslop
- **Effort:** S

### DSLP-22 · P3 · Decide whether parser public APIs with no workspace consumer stay

- **Area:** Needs owner · deslop · dead code
- **Where:** crates/shuck-parser: Error::parse_at, text_looks_like_nontrivial_arithmetic_expression, Parser::with_fuel, Parser::with_limits_and_profile, ParseResult::expect_err
- **Problem:** These public items have no consumer in another workspace crate or fuzz/; some are used only by parser tests or internally. Removing or narrowing them changes the published parser API, and external consumers are unknown.
- **Recommendation:** Owner decides: narrow them to pub(crate) (keeping test use) if the parser has no external consumers, or keep them as supported API.
- **Fix with:** deslop
- **Effort:** S

### DSLP-23 · P3 · Decide whether GlobPatternBehavior::sh_glob stays public

- **Area:** Needs owner · deslop · dead code
- **Where:** crates/shuck-semantic/src/glob.rs:150 (GlobPatternBehavior::sh_glob, re-exported from lib.rs)
- **Problem:** The public constructor has only an in-crate test consumer in the workspace, including fuzz/. Removing it changes the published semantic API; external consumers are unknown.
- **Recommendation:** Owner decides: remove it or narrow it to pub(crate) if shuck-semantic has no external consumers, or keep it as supported API.
- **Fix with:** deslop
- **Effort:** S

### DSLP-24 · P3 · X047 fixture never fires its rule

- **Area:** deslop · tests
- **Where:** crates/shuck-linter rule X047 fixture; parity silent-fixture allowlist
- **Problem:** The X047 rule-specific fixture produces no diagnostic, so the rule has no positive witness and the parity test allowlists it as silent. A regression that stops X047 from firing would pass.
- **Recommendation:** Add a fixture case that triggers X047 and remove it from the silent-fixture allowlist, or record why the rule cannot fire.
- **Fix with:** deslop
- **Effort:** S

### DSLP-25 · P3 · Decide whether format_range stays public

- **Area:** Needs owner · deslop · dead code
- **Where:** crates/shuck-formatter/src/range.rs:13 (format_range, FormattedRange; re-exported from lib.rs)
- **Problem:** Range formatting has no consumer in the CLI, config, benchmark or fuzz crates; only formatter tests use it. Removing it changes the published formatter API, and an editor integration could be an external consumer.
- **Recommendation:** Owner decides: keep it as supported API for editor range formatting, or remove it with its tests if there are no external consumers.
- **Fix with:** deslop
- **Effort:** S

### DSLP-26 · P3 · Six registered rules are empty stubs that never fire

- **Area:** Needs owner · deslop · dead code
- **Where:** crates/shuck-linter rules C065, C066, C067, C068, C077, X077 (registered, empty bodies); C109 reserved
- **Problem:** C065 (else without then), C066 (closing brace after an unterminated command), C067 (empty function body), C068 (closing brace without a preceding terminator), C077 (command substitution inside arithmetic expansion) and X077 (nested default expansion in POSIX sh) are registered and selectable but their bodies are empty, so enabling them silently checks nothing. Their codes are user-facing: configs and suppression comments can name them.
- **Recommendation:** Owner decides per rule: implement it (several overlap parser recovery diagnostics), or deregister it and reject or warn on its code in config and suppressions. Document C109 as reserved.
- **Fix with:** deslop
- **Effort:** M

### DSLP-13 · P4 · C098 spec rationale contradicts its safe fix

- **Area:** deslop · diagnose · rule metadata
- **Where:** docs/rules/C098.yaml:9-10 (rationale and fix_description); crates/shuck-linter/src/rules/correctness/set_flags_without_dashes.rs:45
- **Problem:** The rule rationale tells the reader to 'Add a dash before the flags: set -euo pipefail', but the safe fix inserts '--' instead: 'set euox pipefail' becomes 'set -- euox pipefail', which keeps the words as positional parameters and never enables the options. The fixed script therefore still does not have the behavior the rationale names.
- **Recommendation:** Reword the rationale and description to state that the fix only makes the bare operands explicit positional parameters, and keep the flags interpretation in the message only.
- **Fix with:** refactor-rust
- **Effort:** S

