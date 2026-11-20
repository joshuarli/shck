# Parser dead-code audit

Scope: `crates/shuck-parser`, at base f694102801d369a95abd52a63b79997e8d42152d, on aarch64 macOS with the pinned nightly. The public entry points are `Parser`, `Lexer`, parse results, profiles, errors and shebang parsing, exported by `src/lib.rs` and `src/parser/mod.rs`. No public API or parse result is intentionally changed.

## Removals

All findings below were suppressed with `allow(dead_code)` at the base. For every named item, `rg` searched `crates/`, `fuzz/`, `docs/`, root Cargo.toml and Makefile for its identifier; results were its definition and the internal references stated below. All removed functions had restricted visibility. None is an entry point or registered through a macro, build script, manifest, feature, example, test, script or package configuration. The parser has no build script, target-specific dependency or platform-specific feature. The same evidence applies to default and `benchmarking` features and all platforms; compilation and execution were on macOS only. Consumer tests for every removal were `cargo test -p shuck-parser` and `make test`, against the local workspace. Fuzz uses the parser by path, was searched, and was not executed.

- `collect_brace_syntax_from_pattern`: only recursively called itself; no live caller reaches it. Live brace scanning remains in `scan_brace_syntax_text`.
- `zsh_glob_qualifier_source_text` and `text_position`: the latter was called only by the former, which had no callers.
- `pattern_has_glob_syntax_with_features`, `pattern_has_glob_word`, `literal_text_has_zsh_glob_syntax`, and `literal_text_has_numeric_range_suffix`: a closed call chain without a live entry. The separate pattern parser remains live. The `classic_qualifiers` and `extended_glob` fields of `ZshGlobParseFeatures` were read only by this chain; its `ksh_groups` and `bare_groups` fields remain used.
- `zsh_options_at_offset`, `zsh_word_parsing_enabled_at`, `current_zsh_options`, `record_zsh_case_group_part`, `record_zsh_always_span`, `record_zsh_brace_if_span`, and `ignore_braces_enabled_at`: no callers. The independently called option and glob helpers remain.
- `maybe_parse_loose_var_ref_target`, `looks_like_plain_parameter_access`, `is_plain_parameter_access_name`, and `is_plain_special_parameter_name`: a closed call chain without a live entry. `parse_loose_var_ref` remains used by parameter parsing. `find_matching_parameter_end` had no callers.
- `parse_single_stmt_command`, `parse_anonymous_paren_function`, `has_recorded_comment_between`, and `rebase_nested_parse_error`: no callers. Their surrounding command and error paths remain.
- `ForHeaderSurface::Paren` and its match arm: `parse_for` constructed only `In` on both paths; no other constructor exists. `BraceBodyContext::IfClause` had no constructor; `Ordinary` and `Function` remain in use. Neither variant is exposed outside the parser.

`looks_like_disabled_repeat_loop` is retained under `cfg(test)` because a unit test calls it; it is not an unused test helper. Its production suppression is no longer needed.

## Audit and validation

The parser's sole feature, `benchmarking`, is enabled by `crates/shuck-benchmark/Cargo.toml` and supplies counters used by its library and example. The declared `shuck-ast`, `memchr` and `smallvec` dependencies occur in parser source, and `deser` and `deser-json` are used by `tests/oils_parse.rs`. No dependency or feature was removed. `cargo-machete` and `cargo-udeps` were unavailable; the dependency check was manual. The forced dead-code lint was also used to inspect suppressed findings. The parser manifest's explicit readme warning is retained: it is package metadata and outside the source unused/unreachable-public lint findings.


On the candidate, `TMPDIR="$PWD/.work/tmp" cargo test -p shuck-parser` exited 0 (449 unit tests, one integration test, one doctest). `TMPDIR="$PWD/.work/tmp" cpu-guard --cores 4 -- make test` exited 0 across the workspace, and `TMPDIR="$PWD/.work/tmp" cargo clippy --workspace --all-targets -- -D warnings` exited 0. `TMPDIR="$PWD/.work/tmp" RUSTFLAGS='-W unused -W unreachable_pub' cargo check -p shuck-parser --all-targets --all-features` exited 0 with no parser source warnings; Cargo reports the explicit readme metadata warning noted above.

## Needs owner

`Error::parse_at`, `text_looks_like_nontrivial_arithmetic_expression`, `Parser::with_fuel`, `Parser::with_limits_and_profile`, and `ParseResult::expect_err` have no identifier matches in other workspace crates or fuzz. They are public and may have external consumers; retain them pending an owner decision. Identifier search alone cannot establish external usage.
