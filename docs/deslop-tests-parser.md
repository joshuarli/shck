# Parser test audit

Scope: `crates/shuck-parser`, at base b7f330cb9e3910bd868f5d0ed5cc540337c9ae8f, default features on macOS. The parser's callers include the formatter, semantic model, linter and CLI; workspace tests exercise those consumers.

## Tiers and cost

| Tier | Selection | Count and observed execution cost |
| --- | --- | --- |
| Edit | `cargo test -p shuck-parser process_substitution_like_regex_inside_nested` | 2 tests, <0.01 s execution; 0.14 s reported warm build |
| Change | `cargo test -p shuck-parser` | 449 unit tests, 0.60 s; 1 corpus integration test, 0.20 s; 1 doctest, 0.01 s execution and 0.45 s merged compilation. These numbers include the new error test. |
| Scenario | `oils_corpus_matches_parser_expectations` | 1 integration test, 0.20 s; runs as part of Change. |
| Full | `make test` | Workspace suite; required acceptance check. Warm comparison deferred: `perf-gate-ready` repeatedly reported a busy host. The recorded earlier baseline in AGENTS.md is 14.88 s, but is not a valid paired measurement for this slice. |

Compilation and runner setup are outside the test execution times above. No flake rate is established. The corpus integration test loads fixture expectations; no network or external shell is required.

## Oracles

The nested regex tests in `parser/tests/words/expansions.rs` checked only absence of process substitutions: an empty argument list or body could pass. They now require the command, argument count and regex text, or a nonempty pipeline, respectively. The OILS corpus test could also pass when every case was skipped; it now requires at least one executed case. The malformed fuzz regressions that discard results remain: their contract is termination without a panic, and their inputs previously crashed or hung. The `zip` in the escaped-dollar test is guarded by an explicit argument count.

| Boundary | Guarding test |
| --- | --- |
| `Parser::new`, `parse`, clean result and AST | `test_parse_simple_command` |
| `with_dialect`, dialect gating | `test_posix_dialect_rejects_double_bracket_conditionals` |
| `with_profile`, `with_limits_and_profile`, depth | `substitution_nesting_counts_toward_the_configured_depth` (profile constructor also reached by `with_dialect`) |
| `with_fuel`, nested budget sharing | `nested_parsers_share_the_callers_fuel` |
| `parse_word_string` | `test_parse_word_string_keeps_escaped_dollar_literal` |
| `parse_assignment_word_group` | `test_parse_assignment_word_handles_process_substitution_subscript` |
| `ParseResult`, clean status and diagnostics | `test_parse_simple_command` |
| `ParseStatus`, `ParseDiagnostic`, recovered outcome | `test_parse_reports_eof_only_missing_fi_as_recovered` |
| Fatal status, diagnostic location, partial AST and recovery | `test_parse_recovered_skips_invalid_command_and_continues`; `test_unexpected_top_level_token_errors_in_strict_mode` |
| Limit-exceeded status and diagnostic | `substitution_nesting_counts_toward_the_configured_depth` |
| `Error` constructors, fields and display | `error::tests::parse_errors_preserve_location_and_display` (new) |

Fault proof: inverted the location condition in `Error::fmt` (`line > 0` to `line == 0`); `TMPDIR="$PWD/.work/tmp" cargo test -p shuck-parser parse_errors_preserve_location_and_display` exited 101, with the located error rendered without its location. The fault was reverted.

`cargo-llvm-cov` and `cargo-mutants` are unavailable and not on the crate mirror. Coverage and mutation runs, including survivor dispositions, were skipped; the manual fault above is not a substitute for a mutation sweep.
