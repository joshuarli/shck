# Linter facts test audit

Scope: `crates/shuck-linter/src/facts/` at `ae98cd7a0e970d0cddcf637a91e77b46c51cc7ef`, macOS, pinned nightly, default features. Rules consume the structural summaries built by `LinterFacts::build` and its shell-aware variants in `facts/model.rs`. Semantic source references are built outside this module and the source-path query consumes them. No production behavior changed.

## Tiers and costs

| Tier | Selection | Count and observed cost |
| --- | --- | --- |
| Edit | `cargo test -p shuck-linter --lib <test-name>` | One selected test, 0.01 s execution for each heredoc control. |
| Change | `cargo test -p shuck-linter --lib` | Baseline: 3,631 passed, 1.16 s execution. Candidate: 3,631 passed, 1.47 s. The observed increase is 0.31 s, under the requested 2 s limit. This includes all library tests, not just facts. |
| Crate | `cargo test -p shuck-linter` | 3,631 library tests in 2.07 s, 19 integration tests in 0.08 s, and 4 doctests in 0.29 s. |
| Scenario | Rule fixture parity test in the library suite | One test, included in Change. It exercises rule dispatch against fixtures. |
| Workspace | `make test` | Passed, including downstream consumers. Initial build and execution are not a paired timing. |
| Full | Large corpus and benchmark Full tier | Not run. |

There are 460 `#[test]` declarations under `facts/`; parameterized tests and tests outside that tree affect runner counts. The initial library build took 1m 55s. Execution timings are cargo's reported durations on a busy host, not isolated performance measurements. No flake rate is established. Coverage and automated mutation were skipped because their tools are unavailable on the crate mirror. No mutation survivor dispositions are claimed.

## Boundaries and guards

| Rule-facing boundary | Guarding behavior test |
| --- | --- |
| Command inventory, wrappers and nesting | `facts/tests/flow.rs`: `builds_command_facts_for_wrapped_and_nested_commands` |
| Word facts, host and context | `facts/tests/surface.rs`: `builds_word_facts_with_contexts_hosts_and_anchor_spans` |
| Word span classification | `facts/word_spans/arrays.rs`: `unquoted_all_elements_array_expansion_spans_only_return_unquoted_at_style_parts` |
| Conditional tree and operators | `facts/tests/conditions.rs`: `builds_conditional_facts_with_root_normalization_and_nested_inventory` |
| Simple test syntax and operands | `facts/tests/conditions.rs`: `builds_simple_test_facts_with_shapes_and_closing_bracket_validation` |
| Redirect targets and cached analysis | `facts/tests/commands.rs`: `builds_redirect_facts_with_cached_target_analysis` |
| Heredoc traversal and spans | `facts/tests/conditions.rs`: `tab_stripped_heredoc_substitutions_after_earlier_heredocs_keep_command_spans_intact`; `facts/tests/surface.rs`: the two strengthened heredoc tests below |
| Assignment spans | `facts/tests/assignments.rs`: `collects_comma_array_assignment_spans_from_compound_values` |
| Loop headers, pipelines and lists | `facts/tests/conditions.rs`: `builds_loop_header_pipeline_and_list_facts` |
| Function headers, bindings and calls | `facts/tests/functions.rs`: `function_header_fact_tracks_binding_scope_and_call_arity` |
| Source path anchoring query | `rules/correctness/unanchored_source_path.rs`: `reports_literal_relative_source_paths`, `reports_quoted_home_source_paths` |

These are representative witnesses, not a claim that every field or branch has independent coverage. The source-path boundary already has consumer-level tests; no new boundary test was needed for the categories above.

## False passes

- `surface_facts_ignore_single_quoted_payload_text_in_expanding_heredocs` previously accepted an empty fragment inventory. A real quoted command outside the heredocs now supplies two expected fragments.
- `ignores_unicode_smart_quotes_in_heredoc_payloads` previously accepted an empty span inventory. A command after the heredoc now supplies two expected smart quote spans.
- `background_semicolon_facts_ignore_case_item_terminators` previously accepted an empty span inventory. A background command after the case now supplies the expected semicolon and line.

Some absence-only regression cases remain paired with separate positive tests for their fact family, including comma array assignments in `facts/tests/assignments.rs`. Those negative cases guard specific false-positive inputs. No tests were removed.

## Fault proof

Temporarily changed `WordFacts::single_quoted_fragments` to return `&[]`. `TMPDIR="$PWD/.work/tmp" cargo test -p shuck-linter --lib surface_facts_ignore_single_quoted_payload_text_in_expanding_heredocs` exited 101. The assertion observed `[]` instead of `["'%s'", "'ordinary'"]`. The fault was reverted. No new test was added; this fault checks the strengthened test's positive control.
