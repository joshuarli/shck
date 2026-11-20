# CLI test audit

Scope: `crates/shuck-cli`, base `ae98cd7a0e970d0cddcf637a91e77b46c51cc7ef`, macOS arm64, pinned nightly, default features. The CLI integration tests invoke the binary. No production behavior or snapshots changed.

## Tiers and cost

| Tier | Selection | Baseline | Candidate |
| --- | --- | --- | --- |
| Edit | `cargo test -p shuck-cli --test integration_test <name>` | Individual witnesses | Fault checks below |
| Change | `cargo test -p shuck-cli` | 431 passed, 15 ignored; test binaries 0.06, 0, 1.61, 0.51, 0.10, 0, 0.50, 0.20, 0 s | 437 passed, 15 ignored; 0.08, 0, 1.69, 0.52, 0.03, 0, 0.31, 0.13, 0 s |
| Scenario | Binary integration, managed shells, compatibility and cache tests | Included in Change | Included in Change |
| Workspace | `make test` | Not measured here | Passed; individual binaries reported their execution times |
| Full | Large corpus and black-box oracle | 1 large-corpus and 14 oracle cases ignored | Not run |

Cargo's reported test-binary execution totals were 2.98 s before and 2.76 s after, a 0.22 s decrease in this session; compilation took 2m29s initially and 24.51s after fault builds. These are busy-host observations, not isolated performance measurements. No flake rate is established. Coverage and automated mutation were skipped because their tools are not available on the crate mirror; no survivor dispositions are claimed.

## Boundaries

Names refer to `tests/integration_test.rs` unless a different file is named. Unit checks are supplementary where stated.

| Boundary | Guard |
| --- | --- |
| `check` | `check_good_file_succeeds`, `check_reports_lint_with_full_snippet_by_default` |
| `format` | `format_good_file_succeeds_and_preserves_contents`, `format_check_and_diff_report_noncanonical_input` |
| `clean` | `clean_only_removes_selected_project_entries_from_shared_cache` |
| `run`, `install`, `shell` | `run_cli.rs`: `run_command_string_uses_managed_shell_and_sets_env`, `install_list_shows_newest_version_first`, `shell_subcommand_uses_system_shell` |
| Help, version | `help_shows_commands`, `run_cli.rs::top_level_help_includes_runtime_commands`, `shellcheck_compat.rs::plain_shuck_version_reports_release_version` |
| Exit 0, 1, 2 and child status | `check_good_file_succeeds`, `check_reports_lint_with_full_snippet_by_default`, `config_file_and_isolated_conflict`; `run_cli.rs::run_preserves_nonzero_child_status` (new) checks exit 37 |
| Full, concise | `check_reports_lint_with_full_snippet_by_default`, `check_concise_output_preserves_legacy_one_line_format` |
| JSON, JSON lines, JUnit, GitLab, RDJSON, SARIF, GitHub | `check_json_output_is_valid_json`, `check_json_lines_output_is_valid_json_lines`, `check_junit_output_is_valid_xml`, `check_gitlab_output_is_valid_json`, `check_rdjson_output_is_valid_json`, `check_sarif_output_is_valid_json`, `check_github_output_uses_actions_annotation_format` |
| Grouped | `check_grouped_output_groups_diagnostics_by_file` (new) |
| stdin | `check_dash_reads_from_stdin`, `check_stdin_filename_is_optional_and_controls_per_file_settings`, `format_stdin_round_trips_valid_input`, `run_cli.rs::run_stdin_with_shell_executes_managed_interpreter` |
| Config discovery, explicit file and inline override, isolation | `format_prefers_nested_project_config_for_explicit_files`, `format_honors_explicit_global_config_file`, `format_inline_global_config_override_beats_global_config_file`, `format_isolated_ignores_discovered_project_config`, `check_stdin_loads_project_configuration` |
| Check cache hit, invalidation of source, settings and transitive sources | `commands/check/cache.rs::reuses_cached_results`, `invalidates_cache_when_file_changes`, `invalidates_cache_when_rule_options_change`, `source_closure_changes_invalidate_cached_diagnostics`; binary `check_cache_invalidates_transitive_source_changes` (new) compares changed output with an uncached run |
| `format --minify` | `format_minified_compound_heredoc_reparses` (new), plus formatter's `minify_flushes_heredocs_before_compound_closers` |
| Fix, unsafe fixes, add-ignore, watch and exit-zero | `check_fix_rewrites_safe_s074_and_bypasses_cache`, `check_unsafe_fixes_applies_s059_fix`, `check_add_ignore_writes_inline_shuck_ignore`, `check_watch_reruns_when_files_change`, `check_add_ignore_respects_exit_zero_for_warning_leftovers` |
| Selection and exclusions | `check_cli_select_replaces_config_selection`, `check_additive_rule_flags_change_diagnostics` (new: extend-select, ignore, extend-per-file-ignores), `check_per_file_ignores_skip_matching_files`, `check_unfixable_prevents_fixing_matching_rules`, `check_extend_exclude_adds_to_exclude_patterns`, `check_gitignore_and_force_exclude_flags_control_explicit_files`, `format_gitignore_and_force_exclude_flags_control_explicit_files` |
| Color and output format overrides | `check_color_always_forces_ansi_output`, `check_color_never_overrides_force_color_env`, `check_cli_output_format_overrides_env_var` |
| Format options | `format_honors_project_config_and_cli_overrides_it`, `format_flags_change_stdin_output` (new: binary-next-line, switch-case-indent, space-redirects, keep-padding, never-split, simplify); parser-level `cli::format::tests::parses_format_options`, `tri_state_overrides` |
| Conflicting flags | `config_file_and_isolated_conflict`, `check_stdin_rejects_mixed_paths_and_non_streaming_modes`; parser-level `cli::check::tests::rejects_add_ignore_with_fix_flags`, `rejects_watch_with_add_ignore`, `cli::run::tests::rejects_conflicting_command_and_script` |
| Managed runtime flags | `run_cli.rs::run_dry_run_uses_project_config_and_registry`, `run_command_string_uses_managed_shell_and_sets_env`, `run_bashkit_script_preserves_script_args`; parser-level `cli::run::tests::parses_run_flags_and_passthrough_args` |

## False passes and fault proof

`check_sarif_output_is_valid_json` checked only the envelope; an empty result array would pass. It now requires one C001 result. `cli::check::tests::zsh_plugin_flags_removed` was empty; it now checks both removed flag spellings are rejected. The existing cache-output comparisons alone do not prove a hit; the internal cache test checks hit counts, and the new binary test verifies observable invalidation.

Each fault was reverted after its focused `TMPDIR="$PWD/.work/tmp" cargo test -p shuck-cli --test integration_test <name>` exited 101 on its behavioral assertion. The child-status fault used `--test run_cli` instead:

| Fault | Failing test |
| --- | --- |
| Clear collected dependency paths before writing the check cache entry | `check_cache_invalidates_transitive_source_changes` |
| Discard the minify option before formatting | `format_minified_compound_heredoc_reparses` |
| Dispatch grouped output to the concise renderer | `check_grouped_output_groups_diagnostics_by_file` |
| Emit an empty SARIF results array | `check_sarif_output_is_valid_json` |
| Return success without executing the selected shell | `run_cli.rs::run_preserves_nonzero_child_status` |
| Drop the switch-case-indent override | `format_flags_change_stdin_output` |
| Drop `--extend-select` values during argument parsing | `check_additive_rule_flags_change_diagnostics` |
