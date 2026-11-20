# Linter rules test audit

Scope: linter outside `facts/`, base `ae98cd7a0e970d0cddcf637a91e77b46c51cc7ef`, default features on macOS. The CLI and config crates consume the linter. No production behavior, existing fixture, or snapshot changed.

## Tiers and costs

| Tier | Selection and role | Cost and count |
| --- | --- | --- |
| Edit | `cargo test -p shuck-linter --test public_api <test-name>` | One public boundary scenario per invocation; execution under 0.01 s in the fault runs. |
| Change | `cargo test -p shuck-linter` | Baseline: 3,631 unit, 20 integration and 3 doc tests; test binaries 1.22, 0.00, 0.04, 0.02, 0.01 and 0.05 s. Candidate: 3,631 unit, 22 integration and 3 doc tests; test binaries 2.40, 0.00, 0.06, 0.01, 0.40 and 0.16 s. Cargo reported 22.61 s for the candidate rebuild and 0.17 s for the warm baseline build. |
| Scenario | Rule fixture snapshots, including 317 parameterized category rows, and the registry/spec/fixture parity test | Included in Change. Parity lints fixtures individually and checks rule-specific diagnostics and fix availability. |
| Workspace | `make test` | Required downstream acceptance; passed. The initial workspace build took 3m 05s; execution is reported per test binary. |
| Full | Large corpus and benchmark Full tier | Not run. |

Costs are cargo's reported test execution durations from the same session, not controlled performance measurements; compilation is separate. The first candidate execution sums to 3.03 s versus 1.34 s baseline, an observed increase of 1.69 s (under 2 s). A subsequent warm candidate run summed to 4.39 s on the busy host; the public API binary containing the additions reported 0.01 s and 0.03 s versus 0.00 s baseline. Host variation prevents attribution of the total change to the tests. No flake was observed; no flake rate was established. Coverage and automated mutation were skipped because `cargo-llvm-cov` and `cargo-mutants` are not on the crate mirror. Survivor dispositions and a qualified mutation lane are therefore unverified; the manual faults below are not a mutation sweep.

## Boundaries and guards

| Boundary | Guard |
| --- | --- |
| Registry, aliases and metadata | `rule_codes_are_unique`, `resolves_legacy_shuck_aliases`, `rule_spec_status_matches_registry`, `spec_aliases_resolve_to_their_rule` and `rule_fixtures_produce_diagnostics_and_honor_fix_availability` |
| Select, ignore, extend-select, specificity and unknown selectors | `selection_layers_preserve_specific_rules_and_reject_unknown_codes` (new), `rejects_removed_rule_codes`; CLI `check_cli_select_replaces_config_selection` |
| Suppression disable and file scope | `applies_shuck_disable_to_the_next_command`, `applies_disable_file_directives_to_the_entire_file`, `promotes_shuck_disable_before_the_first_statement_to_file_scope`, and `parsed_result_linting_respects_shellcheck_directive` |
| Enable, unknown and malformed directives | `ignores_malformed_and_unknown_directives`: `enable` is currently unsupported and ignored; disable is scoped to a command or file, not an active range requiring re-enable. `resolves_known_codes_and_ignores_unknown_ones` covers unknown compatibility codes. |
| Fix applicability and applied source | `fix_applicability_and_relint` (new) checks an unsafe rule fix is rejected at the safe threshold, applied at the unsafe threshold, reparsed cleanly and no longer reported. `applies_safe_fix_to_background_semicolons` covers safe edits. |
| Parse diagnostics and nesting limit | `maps_missing_fi_parse_error_to_c035_at_end_of_file`, `ignores_missing_then_parse_error_for_c016`; CLI `nesting_limit_reports_one_parse_error` and `nesting_limit_is_reported_alongside_lint_diagnostics` guard DSLP-10 and absence of cascading C016. |
| Settings | `downstream_construction_and_inspection_path_stays_ergonomic` now asserts an actual selected diagnostic and severity override; `default_rules_include_non_style_rules`, `per_file_shell_matches_paths_and_uses_the_last_match`, and CLI `check_per_file_ignores_skip_matching_files` cover defaults and path settings. |

## False passes and dispositions

`downstream_construction_and_inspection_path_stays_ergonomic` previously linted a clean script and inspected diagnostics only inside a loop, so zero diagnostics passed. It now requires one unused-assignment diagnostic with the configured severity. The existing parity test guards nonempty rule-specific fixture output; its explicit silent-fixture allowlist identifies known exceptions rather than silently claiming coverage. C109 is intentionally reserved and silent, C065–C068, C077 and X077 have empty rule bodies, and X047 has a non-firing existing fixture; X044 and X051 are planned and not registered. Existing empty snapshots and fixture gaps remain visible, not counted as positive rule witnesses. No tests were removed.

## Fault proof

- Temporarily allowed unsafe fixes at the safe threshold in `Applicability::includes`. `TMPDIR="$PWD/.work/tmp" cargo test -p shuck-linter --test public_api fix_applicability_and_relint` exited 101: expected zero fixes, got one. Reverted.
- Temporarily omitted `extend_select` from selector updates. `TMPDIR="$PWD/.work/tmp" cargo test -p shuck-linter --test public_api selection_layers_preserve_specific_rules_and_reject_unknown_codes` exited 101: S074 was missing from the selected rules. Reverted. An earlier mutation that removed extended selectors only from the specificity list survived because the selected exact rules already supplied the same specificity; it was reverted and does not establish coverage of that variation.
