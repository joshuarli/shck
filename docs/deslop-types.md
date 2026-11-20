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

## Small-crate components

The config, discovery, cache, runner, benchmark, and fuzz packages adopt `redundant_clone` for all Cargo targets. The workspace packages also enable truncation, sign-loss, and single-variant wildcard lints; config, cache, runner, and benchmark enable the enum wildcard lint; all except config enable the pass-by-value lint. The four non-benchmark workspace packages warn on unsafe code. The benchmark examples require `unsafe` to implement and forward `GlobalAlloc`: their 21 unsafe occurrences are the three allocator implementations and calls. They record allocations only after successful allocation or reallocation and forward the original pointer and layout on deallocation. Fuzz is a separate manifest and enables all of these clean candidates, including unsafe-code warnings. Its all-target Clippy census produced no reports for the selected lints.

The config dialect match now names every linter dialect, with `shell_dialects_map_or_reject_explicitly` witnessing supported and unsupported mappings. Discovery's other wildcard deliberately groups the remaining external `ignore::Error` variants into a generic message, including future variants. Discovery's fixture size converts its fixed 4096-byte limit with `try_from`; its existing oversized-file test exercises that boundary. Four discovery test clones and the runner's two version-selection clones were redundant and removed. The eight config pass-by-value reports are private override merges: accepting the owned override consistently with the other merge methods avoids introducing a special borrowing convention for the currently copyable fields.

The cache converts signed Unix timestamp parts with `try_from`, preserving rejection of negative parts. It converts epoch milliseconds to `u64` once at creation, rejecting overflow instead of wrapping, and computes the fixed 30-day age from seconds without narrowing. `timestamp_conversion_rejects_overflow_and_negative_parts` covers the maximum, overflow, and signed inputs. The serialized timestamp remains `u64`. The remaining cache `usize` casts encode lengths as `u64`, which is lossless on supported targets. The benchmark signed-to-unsigned reports all clamp live byte deltas to zero before converting; `unsigned_abs` now expresses that nonnegative value without a sign-loss cast. Negative deltas are possible when an allocation predates a measured frame. Remaining size casts represent allocation sizes or fixture counts; allocator `Layout` limits individual allocation sizes to `isize::MAX`, and counts are bounded by addressable memory. The runner's byte-to-index casts widen bytes. The benchmark allocator's signed live totals can still overflow if cumulative measured traffic exceeds `i64` capacity; changing the measurement representation is outside the census leads and would alter benchmark output at that extreme.

Escape-hatch counts are lexical Rust occurrences across each package, including examples, benches, and tests for allows, unsafe, and numeric casts; unwrap/expect counts exclude test files and inline test modules but include benchmark executables. Before is the admitted base and after is this slice.

| Package | allow before/after | unsafe before/after | production unwrap/expect before/after | numeric casts before/after |
| --- | ---: | ---: | ---: | ---: |
| shuck-config | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 |
| shuck-discover | 1 / 1 | 0 / 0 | 0 / 0 | 1 / 0 |
| shuck-cache | 0 / 0 | 0 / 0 | 0 / 0 | 6 / 2 |
| shuck-run | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 |
| shuck-benchmark | 0 / 0 | 21 / 21 | 5 / 5 | 36 / 26 |
| fuzz | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
||||||| parent of 55bf4ac9 (fix(types): check semantic index boundaries)

## Semantic component follow-up

The semantic crate enables `clippy::redundant_clone`, `clippy::cast_possible_truncation`, and `clippy::cast_sign_loss` for all targets. The seven reported clones in builder traversal, CFG construction, source closure, model construction, call facts, and two test fixtures were removed. The arithmetic assignment work item and source-closure resolver configuration remain passed by value: both are short-lived internal inputs handed off to the operation that consumes their role, and neither owns a large allocation.

All truncation leads were reviewed. Semantic binding, reference, scope, block and interned-name IDs now use a checked index conversion when allocated; reconstruction of IDs from collection indexes uses the same conversion. Recorded command allocation already checks its count. Worklist positions now use `usize` throughout, avoiding both narrowing and collision with the `u32::MAX` sentinel. Source offsets use `TextSize::from_usize`, including editor and builder lookups. The traversal guard test uses a wider signed brace count. The remaining numeric casts widen IDs, lengths and bit counts, convert pointers for a relative offset, use the fieldless `ProvidedBindingKind` discriminant as a sort key, or construct test bounds. `oversized_semantic_index_is_rejected` witnesses the ID limit; the AST `oversized_text_size_is_rejected` test witnesses the shared offset limit. Oversized values now panic instead of silently wrapping; valid values preserve their representation.

The wildcard leads were reviewed in all semantic components and tests. Binding-origin decisions for value flow and editor spans, reference-kind dispatch, source-reference diagnostic classification, source-directive selection, function-binding certainty, and named function scope classification now enumerate the owned variants. Remaining fallbacks select a particular syntax shape, dynamic-source form, or fixture target and intentionally reject or use common behavior for the rest; the AST syntax enums are owned by `shuck-ast`. Test scope lookups intentionally select named functions with a particular name. A future distinct semantic category in the decisions now made exhaustive requires an explicit choice.

The declaration reuse assertion was removed by carrying the selected ID through the branch. The remaining production `expect` leads describe construction invariants: a simple command contributes its name to normalization; contiguous word groups are nonempty; the recorded list collector supplies a first segment and an operator for later segments; and recorded command syntax has a kind. The zsh prefix assertion is in a currently disabled branch and its predicate establishes the prefix. Changing those construction contracts or the public command-kind query would require a broader redesign; the assertions retain failure on broken internal invariants rather than adding silent defaults.

Escape-hatch counts below are lexical occurrences in tracked semantic Rust files; numeric casts include tests and all integer target types. Production unwrap/expect counts exclude test files and text following the first inline `#[cfg(test)]` in each source file; this lexical count includes some test helpers outside such modules and is not the Clippy production-only census.

| allow | unsafe | production-text unwrap/expect | numeric casts |
| --- | --- | --- | --- |
| 34 / 34 | 0 / 0 | 40 / 39 | 53 / 26 |
||||||| parent of 48cd07e8 (refactor(types): tighten parser classification decisions)

## Parser component

The parser adopts `clippy::redundant_clone` and `clippy::cast_possible_truncation` at crate level, including tests. The two truncation leads in `source_tree.rs` converted `Position::offset()` back to `u32`; `Position` stores that offset as `u32`, so `TextSize::from_usize` is lossless here. The checked conversion removes both casts without changing the reachable result. `strip_heredoc_tabs` borrows its input, eliminating an unnecessary clone, and the heredoc literal helper also borrows its argument. The `FirstChainOperand` argument in `limits.rs` remains owned because it is a one-use marker: consuming it prevents reuse of a saved peak. The remaining pass-by-value leads are `build_target_subscript` and `prefixed_parameter_raw_body` in `words/assignment.rs`: their callers already hand off temporary owned values, and borrowing would not remove a clone or strengthen an invariant.

Escape-hatch counts use lexical occurrences in parser `src` Rust files, including tests, for `allow`, `unsafe`, and numeric `as` casts; unwrap/expect counts exclude test files and text following inline `#[cfg(test)]` modules. The counts before/after are: allows 29/29, unsafe 0/0, production unwrap/expect 0/0, numeric casts 14/12.

The original wildcard leads were reviewed against their producers and consumers. Locations below are those in the original census, before this slice's edits:

- `arithmetic.rs:204,456` now names every token for assignment and primary-operand decisions. The precedence-layer probes at 246–317 intentionally recognize only their own operators; every other token, including operators handled by other layers, returns `None`. The AST lvalue checks at 503, 513, and 520 intentionally reject every shape other than variables and indexed variables.
- `commands/conditionals.rs:197` names the remaining conditional operators so new operators need an operand decision. The token probe at 476 recognizes only comparison spellings and redirects. `commands/functions.rs:89,147` selects only tokens valid in those header and terminator positions. In `commands/lists.rs`, 75 selectively applies effects to simple commands and and/or lists; 169 now names each binary operator, 324 names each parser command, and 432 recognizes only operand-like tokens.
- `heredocs.rs:128` treats literal body text specially and updates line state from the source for every other body part. `lexer/mod.rs:365` recognizes the three simple word token kinds, treating all other token kinds as composite; 455 now names all payloads that need no rebase. The accessors at 506, 517, 525, 544, 552, and 560 deliberately extract just the requested payload kind. `lexer/tests/mod.rs:15` extracts fixture text only for words, comments, and errors.
- The 33 parser test wildcard leads, including the eight single-variant leads in `tests/mod.rs`, reject fixture shapes other than the one under test; broad rejection makes each test fail for any unexpected current or future shape. These are in `tests/heredocs.rs:302,316,863,986`, `tests/mod.rs:237,268,281,310,339,362,376,411`, `tests/redirects.rs:375`, and `tests/words/{assignments.rs:552,941,943; decode.rs:599; expansions.rs:342,538,959,976,1387,1426,2212,2223,2289,2320; integration.rs:67,190; mod.rs:7,50,78,87}`.
- `word/heredocs.rs:60,63` falls back to literal syntax for non-parameter/non-expansion shapes; `word/parameters.rs:100` returns unchanged shapes not being converted; `word/render.rs:101` treats non-special parts uniformly. `word_tokens.rs:36,183,297` now names all word segment kinds for source selection; 471 and 499 deliberately select only conditional braces and static word tokens. `words/assignment.rs:208,416` selects literal parts and falls back to failure or source syntax for other parts. `words/mod.rs:104,691` treats nonliteral parts as pattern words or searches only expansion forms that can contain the relevant comma. `words/pattern.rs:52,254` accepts only source-backed literals for safe reparsing and literal/quoted parts for static text.

The existing arithmetic precedence and nested-assignment tests, conditional pattern and regex tests, prefix-redirect test, quoted-word tests, and stripped-heredoc tests witness the retained parse behavior; `arithmetic_assignment_operators_keep_their_distinct_kinds` covers every assignment token. The compiler itself witnesses the closed matches: temporarily adding a variant to each parser-owned arithmetic token, word-segment kind, parser command, or token payload makes `cargo check -p shuck-parser --lib` report non-exhaustive matches at the changed decisions. The shared `redundant_clone` negative witness is in the profile decision above.
