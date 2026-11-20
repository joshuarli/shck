# Small-crate test audit

Scope: shuck-ast, shuck-indexer, shuck-config, shuck-discover, shuck-cache and shuck-run at base f694102801d369a95abd52a63b79997e8d42152d, default features, macOS. The CLI owns the final check-cache policy and shell execution; its tests below exercise those consumers.

## Tiers and cost

Edit selects a named boundary test with `cargo test -p <crate> <test>`; the new AST test and strengthened cache test each execute in 0.00 s. Change is `cargo test -p <crate>` for each affected crate. Scenario is the CLI's source-closure and run tests named below, included in Full. Full is the requested `make test` workspace suite; it passed with a 42.08 s build after restoring the injected fault. Compilation and setup are separate from execution times; no flake rate was established.

A warm baseline used `TMPDIR="$PWD/.work/tmp" cargo test -p shuck-ast -p shuck-indexer -p shuck-config -p shuck-discover -p shuck-cache -p shuck-run`. Cargo reported 0.29 s build and these test execution durations, in runner order:

| Crate | Baseline tests and execution | Candidate tests and execution |
| --- | --- | --- |
| ast | 110, 0.00 s | 111, 0.00 s |
| cache | 10, 0.01 s | 10, 0.00 s |
| config | 78, 0.01 s unit; 5, 0.00 s integration | 78, 0.01 s unit; 5, 0.00 s integration |
| discover | 17, 0.02 s | 17, 0.01 s |
| indexer | 48, 0.00 s | 48, 0.00 s |
| run | 49, 0.57 s | 49, 0.45 s |

The candidate selection exited 0; summed execution was 0.47 s versus 0.61 s baseline, a decrease of 0.14 s (under the 2 s added-cost limit). The candidate build reported 31.67 s after source changes.

The baseline command completed all tests successfully; its shell wrapper then exited 1 because `status` is read-only in zsh. The first, cold combined run exited 0. Timing is Cargo's reported execution time, not a controlled performance gate; the host is busy.

## Boundaries and oracles

| Crate and boundary | Guarding test and disposition |
| --- | --- |
| ast: position and span to byte range, size conversion | New `span::tests::position_span_and_text_range_preserve_byte_offsets`; retain `oversized_text_size_is_rejected` for checked overflow on 64-bit. |
| indexer: line/column, line ranges, continuation | Retain `line_index::tests::resolves_line_and_column_to_offsets`, `tracks_multiple_lines_and_ranges`, and `tests::detects_continuation_lines_without_allocating_source_copies`. |
| indexer: heredoc regions and closing markers | Retain `region_index::tests::finds_heredoc_regions_and_tracks_quoted_heredocs` and `tracks_heredoc_closing_marker_ranges`. |
| config: file parsing, precedence, validation | Retain `public_api_layers_discovered_config_and_inline_overrides`, `public_api_prefers_explicit_config_file_over_discovered_file`, and `config_file_rejects_unknown_nested_rule_option_fields`. DSLP-11 remains: ordinary unknown config-file keys are silently ignored. |
| discover: recursive filtering and shell detection | Retain `directory_walk_keeps_unknown_regular_files_filtered` and `detects_env_split_bash_shebang`. |
| discover: ignores and explicit inputs | Retain `ignored_ancestor_directories_do_not_hide_inputs` and CLI `format_gitignore_and_force_exclude_flags_control_explicit_files`. |
| cache: structured settings and invalidation | Retain `cache_key_changes_when_settings_change`, `cache_key_miss_when_only_file_size_changes`, and CLI `invalidates_cache_when_rule_options_change` and `cache_key_includes_analyzed_path_set`. |
| cache: transitive source paths and invalidation | Retain CLI `source_closure_changes_invalidate_cached_diagnostics`: it checks a cache hit, changes a transitive leaf, and compares the subsequent result to an uncached run. |
| cache: corrupt data | Fix `corrupt_or_truncated_cache_is_a_miss`: it now first persists and reads the same entry successfully. Previously an implementation that always missed passed its negative assertion. |
| run: interpreter process execution | Retain CLI `run_command_string_uses_managed_shell_and_sets_env`, `run_stdin_defaults_to_system_bash`, and `run_bashkit_script_preserves_script_args`; shuck-run resolves and installs interpreters, while the CLI launches them. |

No other false pass was identified in the reviewed boundary tests. The cache size and mtime negative checks have a separate positive persistence test. No tests were deleted. Non-Unix process behavior was not exercised on this host.

## Fault proof

- Changed `Span::to_range` to use the start offset for both endpoints. `TMPDIR="$PWD/.work/tmp" cargo test -p shuck-ast position_span_and_text_range_preserve_byte_offsets` exited 101: actual `(4, 4)`, expected `(4, 8)`. Reverted.
- Made `PackageCache::get` return `None` unconditionally. `TMPDIR="$PWD/.work/tmp" cargo test -p shuck-cache corrupt_or_truncated_cache_is_a_miss` exited 101: actual `None`, expected `Some("ok")`. Reverted.

Coverage and mutation tools are unavailable on the crate mirror, so both runs and survivor dispositions were skipped; these manual faults are not a mutation sweep.
