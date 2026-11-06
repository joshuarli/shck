# Development

## Clean-room policy

Shuck is a clean-room reimplementation of ShellCheck. The independent authoring process is a hard constraint, not a preference.

Prohibited inputs:

- ShellCheck source code, wiki pages, documentation examples, or diagnostic wording.
- Raw ShellCheck output copied into committed files.
- Web searches for ShellCheck source, wiki content, or diagnostic text.

Approved inputs:

- Shell language manuals and specifications (POSIX, Bash reference manual) and semantic notes.
- Files authored inside this repository.
- The ShellCheck binary as a black-box oracle: run it, observe numeric exit codes and codes, but do not copy its output text into committed files.
- The companion `shell-checks` repository at `../shell-checks`, which contains independently authored rule specs, examples, and compatibility mappings.

Write summaries, rationales, comments, and diagnostics from scratch. Compatibility codes may be referenced as bare numbers or `SC1234` identifiers. Do not copy ShellCheck text or third-party snippets verbatim.

## Toolchain

The workspace pins `nightly-2026-09-15` with `rust-src` and `clippy` in `rust-toolchain.toml`. Normal development uses that toolchain. Fuzz targets invoke `cargo +nightly` and need a nightly fuzz toolchain (`make fuzz-setup`).

## Build, test, and check

```sh
make build                     # cargo build
make test                      # cargo test
cargo run -p shuck-cli -- check script.sh
cargo test -p shuck-linter -- test_name
make check                     # clippy --workspace --all-features, cargo shear, script checks
```

`make check` runs clippy with all targets and features, then `cargo shear`, then `make check-scripts` (shuck checking `scripts/` and the crate-list check). `make hawk` checks for dead public APIs with a separate toolchain.

The parser crate warns on `clippy::unwrap_used` outside tests (`crates/shuck-parser/src/parser/mod.rs`); avoid new `unwrap()` calls there.

## Large corpus

Compatibility runs compare Shuck against the nix-provided ShellCheck. Use `make test-large-corpus`, which enters the Nix development environment before running `crates/shuck-cli/tests/large_corpus.rs`.

```sh
make setup-large-corpus
make test-large-corpus
make test-large-corpus SHUCK_LARGE_CORPUS_RULES=C001,C006
make test-large-corpus SHUCK_LARGE_CORPUS_SAMPLE_PERCENT=10
make test-large-corpus SHUCK_LARGE_CORPUS_TIMING=1
```

Main environment variables:

| Variable | Effect |
| --- | --- |
| `SHUCK_TEST_LARGE_CORPUS=1` | Enables the ignored test; the Make target sets it. |
| `SHUCK_LARGE_CORPUS_ROOT` | Overrides corpus discovery; defaults to `.cache/large-corpus`, then `../shell-checks`. |
| `SHUCK_LARGE_CORPUS_RULES` | Comma-separated rule selectors to compare. |
| `SHUCK_LARGE_CORPUS_SAMPLE_PERCENT` | Deterministic sampling percentage in `[1,100]`. |
| `SHUCK_LARGE_CORPUS_TIMEOUT_SECS` | ShellCheck timeout per fixture. |
| `SHUCK_LARGE_CORPUS_SHUCK_TIMEOUT_SECS` | Shuck timeout per fixture. |
| `SHUCK_LARGE_CORPUS_MAPPED_ONLY=1` | Restrict ShellCheck diagnostics to mapped codes. |
| `SHUCK_LARGE_CORPUS_KEEP_GOING=1` | Collect all failures instead of stopping at the first. |
| `SHUCK_LARGE_CORPUS_TIMING=1` | Shuck-only timing pass with the slowest fixtures; no compatibility log. |
| `TEST_SHARD_INDEX`, `TEST_TOTAL_SHARDS` | Split the run across shards. |

Run ShellCheck through Nix when debugging so versions match:

```sh
nix --extra-experimental-features 'nix-command flakes' develop --command shellcheck --version
```

Reviewed divergences for a rule are recorded under `crates/shuck-cli/tests/testdata/corpus-metadata/`. `docs/S001_BUGS.md` tracks the current S001 review.

## Linter rules

New rules belong under `crates/shuck-linter/src/rules/<category>/` and must read precomputed facts rather than walking the AST. `crates/shuck-linter/AGENTS.md` owns the layering rules; the short version:

- Add new structural data to `crates/shuck-linter/src/facts/` and expose it through the fact views.
- Do not call traversal or query helpers from rule files.
- Do not import `crate::rules::common::*` inside rule files.
- Keep repeated discovery in facts; rule files should be filters plus policy and wording.

Rule metadata lives in `docs/rules/<CODE>.yaml`. That directory is build input and parity-test input, not generated output.

## Conventions

- Rust edition 2024 on the pinned nightly.
- Rule codes: category letter plus number (`C001`, `S001`, `X001`, `P001`, `K001`). ShellCheck `SCNNNN` codes are accepted in suppressions; `SH-NNN` is a legacy suppression alias only.
- Config files: `.shuck.toml` or `shuck.toml` at the project root.
- The cache uses the shared OS cache root; `shuck clean` removes project entries. Legacy `.shuck_cache/` directories are ignored and cleaned.

## Release

Releases are cut by dispatching the `release` workflow from the default branch. It builds `aarch64-apple-darwin`, `x86_64-unknown-linux-musl`, and `aarch64-unknown-linux-musl`, verifies the Linux binaries are statically linked, creates a `release-<sha>` tag, and publishes a GitHub prerelease containing `shuck-<target>.zip` archives and `SHA256SUMS`.

```sh
make release TARGET=aarch64-apple-darwin
make verify-release TARGET=aarch64-apple-darwin
```

The release build uses the `dist` Cargo profile (`lto = "fat"`, one codegen unit, stripped).

## Commit messages

The repository uses Conventional Commits: `type(scope): subject`, with an imperative, lowercase subject of at most 72 characters. Append `!` or a `BREAKING CHANGE:` footer for breaking changes. PRs are squash-merged, so the PR title is the commit that lands on `main`.

Types: `feat`, `fix`, `perf`, `docs`, `refactor`, `test`, `chore`, `ci`, `build`, `revert`.

Write the subject for a downstream reader; the commit is the permanent record.
