# Deslop completion review

Intent: review the cumulative campaign changes for regressions, particularly parser limits, diagnostics, dependency caching, minification, checked conversions, dead-code removal, and benchmark entrypoints. Reviewed diff: `90c560d2cb767045c90550489534a552cc1be2ae..3e0a4e8f71ae7b1922e69b992ade7ec0964229d7`. Verdict: two demonstrated resource-limit defects remain. Previously filed SHCK-1 through SHCK-3 and SHCK-4 through DSLP-25 are excluded as requested.

## Findings

### P2 — stdin hides the resource limit when lint diagnostics exist

`crates/shuck-cli/src/commands/check/analyze.rs:133` now appends the limit error even when lint findings exist, but `crates/shuck-cli/src/commands/check/stdin.rs:167` still displays the parse error only when the lint diagnostics are empty. Thus a partial analysis of stdin fails without telling the user why later commands were not analyzed. The exit status alone does not distinguish this from an ordinary lint failure. Both inputs below are identical; the file path reports the limit, while stdin does not. Expected: both report it alongside C006.

Reproduction, after `TMPDIR=$PWD/.work/tmp cargo build -p shuck-cli` (build exit 0):

```sh
export TMPDIR="$PWD/.work/tmp"
python3 -c 'from pathlib import Path; Path(".work/tmp/limit.sh").write_text("#!/bin/bash\necho $missing\n" + "true && " * 1001 + "true\n")'
target/debug/shuck check --no-cache --output-format concise "$TMPDIR/limit.sh"
printf 'file exit: %s\n' "$?"
target/debug/shuck check --no-cache --output-format concise --stdin-filename "$TMPDIR/limit.sh" - < "$TMPDIR/limit.sh"
printf 'stdin exit: %s\n' "$?"
```

Output (the two `shuck` invocations each exit 1):

```text
.work/tmp/limit.sh:2:6: error[C006] variable `missing` is referenced before assignment
.work/tmp/limit.sh:3:8009: parse error command list too long (1001 chained operators, max 1000)
file exit: 1
/Users/josh/repos/shck/.work/wt/review/.work/tmp/limit.sh:2:6: error[C006] variable `missing` is referenced before assignment
stdin exit: 1
```

### P3 — fuel exhaustion at the recovery loop boundary is classified as Fatal

`crates/shuck-parser/src/parser/recovery.rs:85` and `:90` break on fuel errors without setting `limit_exceeded`; the new status is selected only at `:171`. This contradicts the `ParseStatus::LimitExceeded` contract in `crates/shuck-parser/src/parser/result.rs:17` and makes consumers that distinguish resource limits treat an incomplete AST as an ordinary fatal parse. With the public `Parser::with_fuel` API, zero fuel reaches this path immediately. Expected status: `LimitExceeded`; observed: `Fatal`.

Reproduction against the parser artifact produced by the reviewed CLI build (the command exits 0):

```sh
export TMPDIR="$PWD/.work/tmp"
cat > "$TMPDIR/status.rs" <<'RS'
use shuck_parser::parser::Parser;
fn main() {
    let result = Parser::with_fuel("echo hello\n", 0).parse();
    println!("status: {:?}", result.status);
    println!("error: {}", result.strict_error());
}
RS
bash -c 'rustc --edition=2024 "$TMPDIR/status.rs" --extern "shuck_parser=$(find target/debug/build/shuck-parser -name "libshuck_parser*.rlib" -print -quit)" --extern "shuck_parser=$(find target/debug/build/shuck-parser -name "libshuck_parser*.rmeta" -print -quit)" $(find target/debug/build -type d -name out -exec printf " -L dependency=%s" "{}" \;) -o "$TMPDIR/status" && "$TMPDIR/status"'
```

```text
status: Fatal
error: parse error: parser fuel exhausted (0 operations, max 0)
```

## Supported risks, unreproduced

None identified beyond the demonstrated findings and the excluded filed issues. The status defect also affects the `skip_newlines` error branch by inspection; that branch was not separately reproduced.

## Evidence and limits

Inspected the cumulative diff and surrounding paths for source dependency collection and fingerprints, minified heredoc flushing and pre-write parsing, line-index ownership, diagnostic state, parser recovery and limits, conversions, removed parser and semantic paths, and benchmark selection. Built the reviewed CLI. A nested-case heredoc minification fixture formatted successfully and `bash -n` accepted the result; this is one case, not a proof of semantic equivalence. The bench driver unit tests passed (3 tests) and its Edit-tier listing selected the harness. `cargo test -p shuck-parser resource_limits` passed: 13 tests, 0 failures (exit 0). No base-revision build, large corpus, full benchmark, network access, or exhaustive search of all possible shell inputs was performed. `git diff --check` on the campaign diff reported an existing trailing blank line at `TODO-DESLOP.md:171`; it is not a behavior finding.

## Resolution

Both findings are fixed in the same change as this review:

- P2: `check` on stdin now reports a resource-limit parse error alongside lint diagnostics, using the same rule as file input (`stdin.rs`). The regression test is `stdin_reports_resource_limit_alongside_lint_diagnostics`.
- P3: fuel exhaustion in the recovery loop's `tick` and `skip_newlines` exits now sets `ParseStatus::LimitExceeded`. The regression test is `fuel_exhausted_between_commands_reports_limit_exceeded`.

Each test fails against the reviewed revision and passes with the fix.

## Later Dead code slices

#30 (linter rules) and #31 (linter facts) merged after the review above and were reviewed separately. #31 only deletes code and narrows visibility. #30 also deletes a `build_script.rs` test that had no `#[test]` attribute and never ran; every build exercises the generator it covered, so no coverage is lost.
