# Types candidate census

Measured at `b7657b63a076b99b6708f1b9cfe9b3cd3e839b8d` with nightly-2026-09-15 (Clippy 0.1.100), default workspace features, all workspace targets, on macOS. Counts are distinct primary source spans per lint and component, deduplicated across targets. Build scripts are included with their crate; test files are included with the component named in AGENTS.md. The separate fuzz manifest is not a workspace member and remains for its own slice. `unsafe_code` was measured as a warning: `forbid` would stop at the first occurrence.

The command was `TMPDIR="$PWD/.work/tmp" cargo clippy --workspace --all-targets --message-format=json -- -W unsafe-code -W clippy::wildcard_enum_match_arm -W clippy::match_wildcard_for_single_variants -W clippy::cast_possible_truncation -W clippy::cast_sign_loss -W clippy::unwrap_used -W clippy::expect_used -W clippy::redundant_clone -W clippy::needless_pass_by_value` (exit 0). Columns 6 and 7 instead use `cargo clippy --workspace --lib --bins --examples --message-format=json -- -W clippy::unwrap_used -W clippy::expect_used` (exit 0), excluding test targets and inline test modules. Other columns include tests. Counts are diagnostic leads, not a claim that every report is a defect.

1. `unsafe_code`; 2. `wildcard_enum_match_arm`; 3. `match_wildcard_for_single_variants`; 4. `cast_possible_truncation`; 5. `cast_sign_loss`; 6. `unwrap_used` outside tests; 7. `expect_used` outside tests; 8. `redundant_clone`; 9. `needless_pass_by_value`.

| Component | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| ast + indexer | 0 | 14 | 0 | 102 | 0 | 0 | 0 | 0 | 0 |
| config + discover + cache | 0 | 2 | 0 | 3 | 2 | 0 | 0 | 4 | 8 |
| run + benchmark | 21 | 0 | 0 | 0 | 10 | 0 | 0 | 2 | 0 |
| linter root/tests | 0 | 9 | 0 | 7 | 0 | 0 | 2 | 1 | 3 |
| linter facts a-f | 0 | 18 | 1 | 7 | 0 | 0 | 2 | 0 | 4 |
| linter facts g-m | 0 | 2 | 0 | 1 | 1 | 0 | 0 | 0 | 0 |
| linter facts n-z | 0 | 20 | 1 | 0 | 0 | 0 | 2 | 0 | 0 |
| linter facts/command_options | 0 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| linter facts/tests | 0 | 7 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| linter facts/word_spans | 0 | 30 | 0 | 0 | 0 | 0 | 7 | 0 | 0 |
| linter facts/words | 0 | 16 | 4 | 0 | 0 | 0 | 1 | 0 | 1 |
| linter correctness a-f | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 2 | 0 |
| linter correctness g-m | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 |
| linter correctness n-s | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 19 | 1 |
| linter correctness t-z | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 6 | 0 |
| linter style a-m | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| linter style n-z | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| linter style/unquoted_expansion | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| linter rules other | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 |
| linter ambient_contracts | 0 | 0 | 0 | 0 | 0 | 0 | 4 | 0 | 1 |
| linter suppression | 0 | 1 | 0 | 2 | 0 | 0 | 0 | 1 | 0 |
| semantic src a-f | 0 | 5 | 1 | 9 | 0 | 0 | 1 | 1 | 0 |
| semantic src g-m | 0 | 1 | 0 | 3 | 0 | 0 | 0 | 1 | 0 |
| semantic src n-z | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic builder | 0 | 13 | 1 | 6 | 0 | 0 | 5 | 1 | 1 |
| semantic cfg | 0 | 0 | 0 | 3 | 0 | 0 | 2 | 1 | 0 |
| semantic dataflow | 0 | 2 | 0 | 7 | 0 | 0 | 0 | 0 | 0 |
| semantic plugins | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic source_closure | 0 | 7 | 1 | 0 | 0 | 0 | 0 | 1 | 1 |
| semantic tests | 0 | 12 | 0 | 2 | 0 | 0 | 0 | 2 | 0 |
| parser src + parser direct | 0 | 21 | 0 | 2 | 0 | 0 | 0 | 0 | 2 |
| parser/commands | 0 | 8 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| parser/lexer | 0 | 9 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| parser/tests | 0 | 33 | 8 | 0 | 0 | 0 | 0 | 0 | 0 |
| parser/word | 0 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| parser/words | 0 | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 2 |
| parser examples/tests | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| formatter src/facts/visit | 0 | 29 | 0 | 12 | 0 | 0 | 3 | 1 | 2 |
| formatter command/render_plan | 0 | 16 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| formatter streaming | 0 | 10 | 0 | 0 | 1 | 0 | 1 | 0 | 2 |
| formatter word | 0 | 23 | 0 | 0 | 0 | 0 | 1 | 0 | 0 |
| formatter tests | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| cli src/cli | 0 | 9 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| cli commands direct | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 8 |
| cli commands/check | 0 | 0 | 2 | 1 | 0 | 0 | 0 | 2 | 3 |
| cli shellcheck compat/runtime | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 |
| cli test: oracle_shellcheck_cli.rs | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| cli test: large_corpus.rs | 0 | 0 | 0 | 3 | 1 | 0 | 0 | 2 | 3 |
| cli test: run_cli.rs | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 |
| cli test: shellcheck_compat.rs | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| cli test: discovery_cache_robustness.rs | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| cli test: integration_test.rs | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| cli other | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 |
| Total | 21 | 330 | 19 | 170 | 16 | 0 | 39 | 52 | 44 |

## Profile decision

Adopt `clippy::redundant_clone` at warn, enforced with `-D warnings`, including tests. There are 52 distinct reports in 337,297 Rust source lines; even if all were false positives, this is below one per 5,000 lines. For example, `crates/shuck-run/src/resolve.rs:49` clones a version that is subsequently moved and never used again, potentially allocating for a range of predicates. Later slices should inspect each report before changing it. The first component has zero reports. Existing production-only `unwrap_used` warnings in the AST, indexer, and parser remain in place; they are not a newly selected workspace profile.

The negative witness below is rejected with `error: redundant clone` by `clippy-driver --edition=2024 -D clippy::redundant_clone --emit=metadata witness.rs` (exit 1). It isolates the new lint contract: once the original is dead, an unnecessary clone fails the selected check. The offset changes below additionally tighten this component’s runtime boundary.

```rust
fn main() {
    let value = String::from("shell");
    let copy = value.clone();
    drop(copy);
}
```

The other candidates are not adopted as a shared profile in this slice: `unsafe_code` (21) includes required benchmark allocator implementations; wildcard matches (330 and 19) include intentional partial matches, including tests, without evidence that the fallback is wrong; truncation (170) and sign-loss (16) include bounded source offsets and checked ranges but deserve individual review in each component; `unwrap_used` (0 production) has no new finding, and `expect_used` (39 production) includes startup/build-time invariants; `needless_pass_by_value` (44) includes public signatures and ownership choices. These counts do not establish that all reported casts or assertions are safe. Later slices should investigate relevant findings without treating this census as a blanket exemption.

## First component escape-hatch baseline

Counts are lexical occurrences in first-party `src/*.rs`, including tests for `allow`, `unsafe`, and numeric `as` casts; unwrap/expect counts cover production text before inline `#[cfg(test)]` modules.

| Crate | allow before/after | unsafe before/after | production unwrap/expect before/after | numeric casts before/after |
| --- | ---: | ---: | ---: | ---: |
| shuck-ast | 3 / 3 | 0 / 0 | 0 / 0 | 28 / 20 |
| shuck-indexer | 0 / 0 | 0 / 0 | 0 / 0 | 94 / 0 |

## Component follow-up

For later components, apply the adopted `redundant_clone` profile and review and fix that component’s census leads for casts and wildcard matches. The census above remains the original baseline.

In AST and indexer, `cast_possible_truncation` is now enabled for all targets. `TextSize::from_usize` checks the shared `usize` to `u32` boundary and panics if the value cannot fit; `Position::at` uses it for all three coordinates, and position advancement and rebasing check addition. Index construction checks source length before scanning; derived byte offsets and test offsets use the same checked conversion. This preserves existing signatures, while oversized coordinates and source lengths now panic instead of potentially wrapping in release builds. The tests `oversized_position_is_rejected`, `oversized_text_size_is_rejected`, `advancing_past_maximum_is_rejected`, and `adding_past_maximum_is_rejected` witness the boundary. The remaining numeric casts in AST widen `u32` to `usize` or define/test the maximum, rather than truncating source offsets.

All 14 original wildcard reports were reviewed. The AST parameter-operator fallback in rendering and source-backed checks, indexer redirect traversal, compound folding-start decision, and close-delimiter visitor now name all variants so additions require review. The remaining AST matches are intentionally selective: static text and status detection reject all expansion kinds; source-slice selection and quoted rendering delegate every non-special case to common logic. The close-delimiter test helper rejects all compounds outside the three fixture kinds. The wildcard lint is not enabled: these intentional partial matches remain.
