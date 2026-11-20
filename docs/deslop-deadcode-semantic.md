# Semantic dead-code audit

Scope: `shuck-semantic` at base `ae98cd7a0e970d0cddcf637a91e77b46c51cc7ef`, pinned nightly on aarch64 macOS. Its manifest has no features, target-specific dependencies, build script, binary or benchmark targets. The all-targets, all-features compiler check covers the host and all declared features; Windows-only source-closure paths were inspected but not compiled or executed. No external public contract or analysis result is intentionally changed.

## Removals

- The literal-false zsh existence checks in word and parameter traversal, their private probe and name parsing helpers, and the uncalled `visit_zsh_unknown_operation_references` helper. Entry points remain `visit_word_part` and `visit_parameter_expansion` in `builder/words.rs`; the deleted checks could never execute. Searches of the entire workspace, including linter, CLI, formatter, benchmark, fuzz, manifests, docs and scripts, found no other consumers or registrations. There are no feature, platform, build or package gates for these helpers. The semantic and workspace tests below exercise the remaining traversal.
- The literal-false zsh subscript flag-prefix branch in `builder/mod.rs`: the branch always chose zero, so scanning starts at the same byte and retains the same offsets. The containing source-text scan remains reachable through semantic building; workspace searches found no independent registration or build/package reference. It has no feature or platform gate. Semantic and workspace tests below cover consumers.
- `zsh_qualified_glob_is_semantically_inert` in `builder/inert.rs` and `pattern_operator_behavior` in `glob.rs`: each was a suppressed, uncalled private helper; searches across all workspace crates, fuzz, manifests, docs and scripts found no consumer or registration. Existing inert-pattern and glob entry points remain. Neither helper had platform or feature gates or build/package references. Semantic and workspace tests below cover their surrounding entry points.
- Test-only `model_at_path_with_plugin_resolver` and `zsh_runtime_source` in `tests.rs`: searches across the workspace and fuzz found no calls, registrations, build or package references. They were not feature- or platform-gated; all semantic tests run below.
- The unreachable `AstFacts`, `zsh_plugin_frameworks`, and `layout_for_plugin_framework` placeholders in the private `source_closure::plugin_managers` module. Whole-workspace and fuzz searches found no consumer, registration, build or package reference; the distinct parent `source_closure::AstFacts` remains in use. The source-closure entry points and their empty plugin hooks remain unchanged. These placeholders had no feature or platform gates; semantic and workspace tests below cover their consumers.

The internal recorded-program statement accessor and test-only dataflow result and accessor were narrowed to crate visibility. The linter calls the separate public `SemanticModel::statement_sequence_commands` wrapper, which remains unchanged. Workspace and fuzz searches found no consumers of the internal types. This is not an externally reachable API change.

## Audit and validation

The compiler check uses `RUSTFLAGS='-W unused -W unreachable_pub'` with all targets and features. Cargo's `manual_readme` warning for the explicit semantic package README is allowlisted: it is package metadata, and retaining it makes the published README explicit. Other manifest warnings belong to other crates. No source warning is allowlisted.

Neither `cargo-machete` nor `cargo-udeps` was installed. Manual inspection matched all semantic dependencies to source or tests: bitflags in `binding.rs`, compact_str in `source_ref.rs`, rustc-hash and smallvec in `cfg/builder.rs`, AST and indexer in `lib.rs`, parser in `lib.rs` and tests, and tempfile in `source_resolve.rs` tests. No unused feature was found. Fuzz has a separate manifest and was searched, but not executed. Remaining disabled zsh machinery and suppressed helpers were not proven safely removable within this slice; no claim of a complete structural debloat is made.

Consumer verification: `cargo test -p shuck-semantic` (exit 0; 449 unit tests and one integration test passed), `make test` (exit 0) for the workspace consumers, and `cargo clippy --workspace --all-targets -- -D warnings` (exit 0), all with `TMPDIR` set to the worktree's `.work/tmp`. The final all-targets, all-features warning check exited 0 with no semantic source warnings. The workspace uses the local semantic crate via the root workspace dependency. Fuzz was inspected only.

## Needs owner

The remaining zsh surfaces and dormant paths may represent externally usable API or live behavior and need separate analysis before any walk-back.
