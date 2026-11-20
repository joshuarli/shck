Prefix: SHCK
Next: 11

# Open work
### SHCK-1 · P3 · config files silently ignore unknown keys that --config rejects

- **Area:** diagnose · config
- **Where:** crates/shuck-config/src/lib.rs:103 ShuckConfig, :127 FormatConfig, :243 LintConfig (all without deny_unknown_fields) versus :1800 parse_config_override which validates strictly
- **Problem:** A mistyped key in shuck.toml is accepted and ignored, so the intended settings never apply, while the same key through --config is an error. Repro: a script that reports C006 with defaults; with '[lint] select = ["C001"]' C006 is suppressed; with the typo '[lint] selects = ["C001"]' the default rule set runs again (C006 reported) and check exits 0. A '[bogus]' table is likewise accepted. The nested section structs already use deny_unknown_fields, so the behavior is inconsistent within one file format.
- **Recommendation:** Add deny_unknown_fields to the root and section structs, or emit a warning that names the ignored key; add a test that a mistyped [lint] key fails or warns.
- **Fix with:** refactor-rust
- **Effort:** S

### SHCK-2 · P3 · zsh leftovers advertise shells and config keys the code rejects or ignores

- **Area:** diagnose · config and help text
- **Where:** crates/shuck-cli/src/cli/run.rs:15, install.rs:21, shell.rs:12 (help lists zsh); crates/shuck-config/src/lib.rs:1318-1370 ([lint.zsh]) and :1414 ([run.shells].zsh); crates/shuck-cli/src/cli/check.rs:611 empty zsh_plugin_flags_removed test
- **Problem:** After zsh support was removed the public surfaces still offer it. 'shuck run --shell zsh' errors 'invalid managed shell zsh; expected one of: bash, gbash, bashkit, dash, mksh, busybox' while the help text for run, install, and shell lists zsh. configuration_metadata(), which generates the published settings reference, still documents [lint.zsh.plugins] (silently ignored by check) and [run.shells].zsh (rejected by shuck run but accepted by shuck check). The zsh_plugin_flags_removed test has an empty body and asserts nothing.
- **Recommendation:** Remove zsh from the three help strings and from configuration_metadata, or restore it end to end; validate [run] the same way in check and run; replace the empty test with one that asserts --shell zsh is rejected.
- **Fix with:** refactor-rust
- **Effort:** S

### SHCK-3 · P3 · Inputs over 4 GiB panic instead of failing with an error

- **Area:** types
- **Where:** crates/shuck-ast/src/span.rs (TextSize::from_usize, Position::advance); CLI input reading
- **Problem:** Source positions are packed in u32. The first Types slice made the conversion checked, so an input whose offsets, lines or columns exceed u32::MAX now panics where it previously wrapped and reported wrong spans. Not reproduced: it needs an input over 4 GiB.
- **Recommendation:** Reject inputs longer than u32::MAX bytes at the CLI and library entry points with an ordinary error before parsing, so the checked conversion is never reached by user input.
- **Fix with:** deslop
- **Effort:** S

### SHCK-4 · P3 · Bench driver has no regression verdict

- **Area:** bench
- **Where:** benches/driver.py (run); docs/benchmarks.md
- **Problem:** `make bench` runs tiers and workloads but has no accepted baseline, interleaved comparison or budget, so neither an immediate nor a cumulative slowdown fails it. The existing `bench-save`/`bench-compare` targets are separate and untiered.
- **Recommendation:** Have `make bench-check` compare a candidate against a saved rustybench baseline for the selected tier, interleaved, with a per-workload budget, and fail on an exceeded budget.
- **Fix with:** bench-pyramid
- **Effort:** M

### SHCK-5 · P3 · Parser benchmark and corpus script do not check their results

- **Area:** bench
- **Where:** crates/shuck-benchmark/benches/parser.rs:9; scripts/benchmarks/run_repo_corpus.sh:246
- **Problem:** The parser benchmark consumes a command count without asserting the expected value, and the repository-corpus script discards output and tolerates failures. A change that does less or wrong work can look faster.
- **Recommendation:** Assert each workload's expected result (command count, diagnostic count or output digest) once per run before timing, and make the corpus script fail on a failed or changed run.
- **Fix with:** bench-pyramid
- **Effort:** S

### SHCK-6 · P3 · Tiered benchmarks omit process startup, whole-repo runs and memory

- **Area:** bench
- **Where:** docs/benchmarks.md (outcome list); benches/driver.py
- **Problem:** The tiered workloads exercise in-process library paths. Process startup, whole-repository `shuck check` and peak memory stay in separate macro and memory scripts outside the tiers, so a regression there is not seen by `make bench`.
- **Recommendation:** Add a Scenario workload that times the release `shuck check` binary over a fixture tree and records peak RSS, using the existing memory scripts.
- **Fix with:** bench-pyramid
- **Effort:** M

### SHCK-7 · P3 · Benchmark runs do not identify fixture content or revision

- **Area:** bench
- **Where:** benches/driver.py:59 (preflight) and :107 (summary)
- **Problem:** Preflight checks fixture sizes but not contents, and run summaries record neither the tested revision nor a dirty tree. A same-size fixture change, or an unidentified candidate, can hide or fake a regression.
- **Recommendation:** Verify fixture digests from crates/shuck-benchmark/resources/manifest.json in preflight, and record the commit and dirty state in each summary.
- **Fix with:** bench-pyramid
- **Effort:** S

### SHCK-8 · P3 · Profiling commands need an undefined cargo profile

- **Area:** bench
- **Where:** docs/benchmarks.md:56; Cargo.toml
- **Problem:** The documented profiling commands use `--profile profiling`, which the workspace does not define, so they fail and regressions are harder to attribute.
- **Recommendation:** Define a `profiling` profile (release with debug symbols) in Cargo.toml, or fix the documented commands.
- **Fix with:** bench-pyramid
- **Effort:** S

### SHCK-9 · P3 · X047 fixture never fires its rule

- **Area:** tests
- **Where:** crates/shuck-linter rule X047 fixture; parity silent-fixture allowlist
- **Problem:** The X047 rule-specific fixture produces no diagnostic, so the rule has no positive witness and the parity test allowlists it as silent. A regression that stops X047 from firing would pass.
- **Recommendation:** Add a fixture case that triggers X047 and remove it from the silent-fixture allowlist, or record why the rule cannot fire.
- **Fix with:** deslop
- **Effort:** S

### SHCK-10 · P4 · C098 spec rationale contradicts its safe fix

- **Area:** diagnose · rule metadata
- **Where:** docs/rules/C098.yaml:9-10 (rationale and fix_description); crates/shuck-linter/src/rules/correctness/set_flags_without_dashes.rs:45
- **Problem:** The rule rationale tells the reader to 'Add a dash before the flags: set -euo pipefail', but the safe fix inserts '--' instead: 'set euox pipefail' becomes 'set -- euox pipefail', which keeps the words as positional parameters and never enables the options. The fixed script therefore still does not have the behavior the rationale names.
- **Recommendation:** Reword the rationale and description to state that the fix only makes the bare operands explicit positional parameters, and keep the flags interpretation in the message only.
- **Fix with:** refactor-rust
- **Effort:** S

