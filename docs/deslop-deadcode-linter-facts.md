# Linter facts dead-code audit

Scope: `crates/shuck-linter/src/facts/` at base `d53111b180c46c75b64a0e2e9d902e13d84ec843`, on aarch64 macOS with the pinned nightly. The linter crate root exports its supported fact types. None of the removed items is re-exported there. The all-targets, all-features check covers the host and all declared features, not other platforms.

## Removals

For every item below, identifier searches covered all rules, the rest of the linter, all other workspace crates including CLI and benchmark, fuzz, manifests, and docs. The only consumers were the internal references described below. None is a registered entry point or has a platform or feature gate. The consumer tests are the linter and workspace tests listed under validation.

- `zsh_parameter_targets_visible_array` returned `false` for every input. Its only caller, `collect_zsh_parameter_array_fanout`, therefore could not change the result. The separate variable-array path remains live.
- `zsh_declaration_brace_assignment_target` had an `if false` return. `zsh_literal_assignment_value_for_command_name_check` always returned its input before its disabled branch. Their live callers retain the same results.
- `CommandFactRef::suspicious_body_name_bracket_glob_spans` had an `if false` extension. The separate brace character-class query remains used by word occurrences.
- The guarded `builtin`, `regexp-replace`, `sched`, declaration, `zpty`, `zstyle`, `_p9k_param`, and `_p9k_prompt_segment` cases in `single_quoted_literal_exempt_argument` could never match. Their private helpers `zsh_builtin_dynamic_wrapper_argument`, `zsh_declaration_delayed_eval_literal_argument`, `sched_command_argument_index`, `zpty_command_argument_index`, `zpty_non_command_mode_option`, `zstyle_delayed_eval_argument_indices`, `zstyle_command_style_argument_indices`, `p10k_param_default_argument_index`, and `p10k_prompt_segment_expansion_argument` had no other callers.
- `BodyTraversal::SkipChildren` and `Break` had no constructors, and `BodyTopology::next_sibling` had no caller. `previous_sibling` is live and remains.
- `collect_zsh_option_map_suppressed_subscripts_in_expr`, `collect_zsh_option_map_suppressed_subscripts_in_lvalue`, and `arithmetic_index_uses_zsh_option_map_key_semantics` formed an uncalled recursive chain. `collect_base_prefix_spans_in_zsh_target`, `collect_base_prefix_spans_in_arithmetic_zsh_target`, `assignment_target_has_assoc_context`, and `collect_status_parameter_spans_in_zsh_target` had no callers.
- `collect_zsh_parameter_expansion_self_uses`, `command_is_pipeline_tail_operand`, `binary_child_pipe_tail_relation`, `command_is_textual_pipeline_tail_operand`, and `text_starts_with_pipeline_operator` had no live callers. The pipeline helpers called one another only inside the removed chain.
- `BindingValueFact`'s selectorless-subscript field and its two queries had no readers. Their builders `word_zsh_selectorless_subscript_value_base_names`, `word_part_nodes_are_zsh_selectorless_subscript_value`, `word_part_is_zsh_selectorless_subscript_value`, `literal_starts_with_zsh_subscript`, `parameter_is_zsh_selectorless_subscript_value`, and `var_ref_has_selectorless_subscript` served only that field.
- `CompoundLookupKind::Repeat`, `Foreach`, and `Always` and `WordTraversalOrigin::ZshParameterTarget` and `ZshModifierArgument` had no constructors. `visit_zsh_qualified_glob` had no calls, including in the test visitor. `collect_zsh_qualified_glob` was an uncalled empty method.
- `SourceFactStore::shell` and its query had no readers. Shell behavior used elsewhere remains in the builder and checker.
- `zsh_parameter_index_flag_fragments` and its query had no readers. Its type, collector, scratch buffer, and private scanning helpers `quoted_parameter_target_len`, `collect_zsh_parameter_index_flag_spans_in_word`, `next_live_parameter_expansion_start`, and `dollar_single_quoted_fragment_len` served only that result.
- `single_quoted_body`, `zsh_delayed_eval_assignment_target`, `zsh_delayed_prompt_assignment_target`, `zsh_delayed_eval_literal_body`, `zsh_literal_map_key_target`, `zsh_vcs_info_patch_loop_literal`, `single_parameter_marker_literal`, and `single_quoted_body_contains_expansion` had no live entry point.
- The private word-span helpers `literal_starts_with_zsh_subscript`, `var_ref_is_zsh_ranged_positional_at`, `parameter_uses_zsh_ranged_positional_at`, `part_has_zsh_short_positional_range`, `literal_starts_with_zsh_positional_range`, `zsh_qualified_glob_active_pattern_spans`, and `zsh_qualified_glob_has_control_syntax` had no live callers. The similarly named assignment helper served the removed selectorless-subscript builder.
- `zsh_modifier_behavior`, `apply_toggle_override`, and `zsh_operation_uses_pattern` had no callers. Test helpers `analyze_argument_words_with_dialect` and `span_for` had no calls.
- `NativeZshScalarArrayReference::new` had no callers. Its type and public query remain exported. `single_quoted_fragment_len`, `double_quoted_fragment_len`, `parameter_expansion_len`, `command_substitution_len`, and `backtick_substitution_len` formed a closed chain after the unused index flag collector was removed.

The compiler identified internal `pub` declarations that were unreachable outside the crate. Their visibility was narrowed to the compiler-suggested scope. The crate-root exports remain unchanged.

## Validation

The all-targets, all-features check with `-W unused -W unreachable_pub` exited 0 with no warnings under `src/facts/`. The same check with `--force-warn dead_code` exited 0 with two allowlisted warnings:

- `CommandFactStore::case_pattern_impossible_spans` is read by the fact test in `facts/tests/surface.rs`.
- `CommandFactQueries::case_pattern_impossible_spans` is the query used by that test.

Both are retained to preserve the existing test of case-pattern analysis. Other source warnings belong to the rest of the linter or dependencies. Cargo also reports manifest metadata warnings.

`cargo-machete` and `debloat` were unavailable. No dependencies were changed. Fuzz consumers were searched but not run. The declared `benchmarking` feature was included in the compiler checks.

`TMPDIR="$PWD/.work/tmp" cargo test -p shuck-linter` exited 0 with 3,631 unit tests and all integration and documentation tests passing. `TMPDIR="$PWD/.work/tmp" cpu-guard --cores 4 -- make test` exited 0 across the workspace.
`TMPDIR="$PWD/.work/tmp" cargo clippy --workspace --all-targets -- -D warnings` exited 0. Cargo reported a benchmark manifest dependency warning.
