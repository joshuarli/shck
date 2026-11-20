# Formatter test audit

Scope: `crates/shuck-formatter`, base `f694102801d369a95abd52a63b79997e8d42152d`, default features on macOS. The formatter is consumed by the CLI; workspace tests exercise that consumer. No production behavior changed.

## Verification

| Tier | Selection | Count and observed execution cost |
| --- | --- | --- |
| Edit | `cargo test -p shuck-formatter --test format_cases minify_flushes_heredocs_before_compound_closers -- --exact` | 1 test, 0.00 s execution |
| Change | `cargo test -p shuck-formatter` | 92 unit tests, 0.05 s; 551 integration cases, 0.14 s; 7 oracle harness tests, 0.00 s; 2 oracle tests ignored; 0 doctests. Before and after reported the same durations, a 0.00 s increase, below 2 s. |
| Scenario | `selected_fixtures_match_shfmt` | 1 explicitly ignored test; requires shfmt and `SHUCK_RUN_SHFMT_ORACLE=1`; cost not measured. |
| Full | `make test` | Workspace acceptance suite: 5,960 passed, 18 ignored across 41 runners; summed reported execution times 10.54 s, build 2m 27s. Large corpus and external shfmt scenarios remain opt-in. |

The Change baseline warm build reported 0.15 s; the candidate rebuilt in 21.85 s. Execution times exclude compilation. Host contention was not controlled, so these are cargo's observed durations, not a qualified performance gate. Flake rate and memory use are not established. Coverage and mutation tools are unavailable on the crate mirror and were skipped; no mutation survivor disposition is claimed.

## Oracles

- `format_range_edits_reformat_to_whole_file_output_at_every_offset` could pass if every selection returned `None`. It now requires a selection for each source.
- The two ignored shfmt tests returned success when explicitly run without their required environment; they now fail with instructions. The large-corpus test also requires at least one comparison, preventing all-skipped input from passing.
- `minify_flushes_heredocs_before_compound_closers` asserted exact output but did not assert that it parses. It now reparses each result as Bash before checking its literal output and the AST path.

The shfmt tests remain ignored during normal Change runs; their dependencies and corpus were not exercised. Range tests allowing `None` for individual offsets remain intentional: whitespace and incomplete selections need not select a statement.

Negative control: running the built `oracle_shfmt` test binary with `SHUCK_RUN_SHFMT_ORACLE` unset and `selected_fixtures_match_shfmt --ignored --exact` exited 101 with the required-environment message.

| Public boundary | Guarding test |
| --- | --- |
| `format_source`, changed output | `formats_simple_command_with_tabs_by_default` |
| `format_file_ast` | `preserves_tab_stripped_heredoc_body_source` via `assert_formats_to_with_ast`; `format_file_ast_matches_format_source_for_formatter_fixtures` |
| `source_is_formatted`, true and false | `check_path_reports_already_formatted_sources`; `formats_simple_command_with_tabs_by_default` |
| `format_range`, selected replacement and unaffected lines | `format_range_leaves_other_lines_alone`; `format_range_formats_targeted_statement` |
| Indent style and width | `space_indented_block_command_substitution_formats_shell_around_multiline_literal` (two spaces); `space_indented_command_substitution_normalizes_with_spaces` (four); `keeps_if_close_suffix_comment_on_outer_close` (tabs) |
| Minify | `minify_drops_comments_but_preserves_shebang`; `minify_flushes_heredocs_before_compound_closers` |
| Simplify | `simplify::tests::arithmetic_var_rewrite_unwraps_simple_parameters` calls `format_source` and checks literal output |
| Idempotence | `stable_formatter_fixtures_are_idempotent` |
| Failure on unparsable input | `posix_dialect_propagates_parse_errors` |

All listed boundaries already had guards, so no new boundary test was needed. The new reparse assertion was fault-tested: temporarily replace `\nfi;` with ` fi;` in `format_output`; `TMPDIR="$PWD/.work/tmp" cargo test -p shuck-formatter --test format_cases minify_flushes_heredocs_before_compound_closers -- --exact` exited 101 at the new parse assertion, reporting `EOF fi; echo after`. The fault was reverted. This single hand-injected fault is not a mutation sweep.
