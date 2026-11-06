# shuck

Shuck is a shell script linter, formatter, and managed shell runner written in Rust. It is a clean-room reimplementation of ShellCheck: never read ShellCheck source, wiki pages, or diagnostic wording; the ShellCheck binary is only a black-box oracle. See [development](docs/development.md#clean-room-policy).

Crates: `crates/shuck-ast`, `crates/shuck-benchmark`, `crates/shuck-cache`, `crates/shuck-cli`, `crates/shuck-config`, `crates/shuck-discover`, `crates/shuck-formatter`, `crates/shuck-indexer`, `crates/shuck-linter`, `crates/shuck-parser`, `crates/shuck-run`, `crates/shuck-semantic`. Roles are in [architecture](docs/architecture.md#crates).

## Commands

```sh
make build                 # cargo build
make test                  # cargo test
make check                 # clippy, cargo shear, script checks
cargo run -p shuck-cli -- check script.sh
cargo test -p shuck-linter -- test_name
```

The workspace pins `nightly-2026-09-15` in `rust-toolchain.toml`. Fuzz targets use `cargo +nightly`.

## Hard invariants

- Lint rules read facts from `crates/shuck-linter/src/facts/`; rule files do not walk the AST or call traversal helpers. See [linter layering](crates/shuck-linter/AGENTS.md).
- `docs/rules/*.yaml` is build and parity input. Rule changes update the matching spec.
- Rule codes use a category letter plus number; `SH-NNN` is a legacy suppression alias only.
- Config files are `.shuck.toml` or `shuck.toml` at the project root.
- The parser warns on `clippy::unwrap_used` outside tests (`crates/shuck-parser/src/parser/mod.rs`).
- Commits follow Conventional Commits; PRs are squash-merged, so the PR title is the commit.

## Where things are

| Topic | Doc |
| --- | --- |
| Crates, check pipeline, parser, indexer, semantic model | [architecture](docs/architecture.md) |
| Rules, dispatch, facts, diagnostics, suppressions, expansion analysis | [linting](docs/linting.md) |
| Ambient contracts and `# shuck: source=` directives | [contracts](docs/contracts.md) |
| Formatter design and options | [formatting](docs/formatting.md) |
| `shuck run`, `install`, and `shell` | [managed shells](docs/run.md) |
| Benchmarks, fixtures, profiling | [benchmarks](docs/benchmarks.md) |
| Clean room, toolchain, tests, large corpus, releases, commits | [development](docs/development.md) |
| Rust API compatibility boundary | [Rust API](docs/rust-api-compatibility.md) |
| Rule metadata | `docs/rules/` |
| Reviewed S001 divergences | [S001 bugs](docs/S001_BUGS.md) |
| Linter crate layering rules | [shuck-linter AGENTS](crates/shuck-linter/AGENTS.md) |
