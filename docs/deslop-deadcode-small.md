# Small-crate dead-code audit

Scope: the root manifest and shuck-ast, shuck-indexer, shuck-config,
shuck-discover, shuck-cache, and shuck-run. The audit used the pinned nightly
on aarch64 macOS. The six manifests define no features or target-specific
dependencies. Source contains Unix and non-Unix branches; only the host's Unix
branches were compiled and exercised. No public API, configuration or CLI
behavior was intentionally changed.

## Removals

- Root `workspace.dependencies.shuck-benchmark`: no member or fuzz manifest
  inherits it. The benchmark remains a workspace member (`members = ["crates/*"]`),
  with its own package, benches, feature and Makefile entry points; those do not
  use this dependency declaration. Removed the now-obsolete cargo-shear ignore
  for that declaration as well. `rg` checked all Cargo manifests, source,
  scripts and documentation, including linter, CLI, benchmark and fuzz.
- Root `workspace.dependencies.tracing`: no member or fuzz manifest inherits
  it, and the workspace source has no Rust crate reference to it. No entry
  point, registration, build script or package reference depends on this
  declaration. Other uses of the word tracing refer to shell fixtures or
  diagnostic concepts, not this Cargo dependency.
- Root `workspace.package.description`, `keywords`, and `categories`: no
  member or fuzz manifest inherits these fields. Member package metadata is
  explicitly declared where present; the benchmark and CLI retain their own
  values. These fields have no runtime entry point or registration. Cargo
  reported all three as unused; categories was an additional finding beyond
  the original lead. All member manifests and fuzz/Cargo.toml were checked.
- `shuck-config` direct `shuck-ast` dependency: no reference to `shuck_ast` or
  `shuck-ast` exists in its source, tests or manifest beyond the removed line;
  it has no build script, feature or target-specific registration. Config's
  public entry points are consumed by CLI and discovery; no linter, benchmark
  or fuzz source or manifest refers to shuck-config. Other crates and fuzz use
  shuck-ast directly and retain their own dependencies. Cargo.lock removes only
  the config-to-AST edge; AST remains in the dependency graph.

The root declarations are build metadata and the config edge is unused: none
has platform-specific logic or a feature gate. The consumer checks below cover
all workspace crates including linter, CLI and benchmark against these local
manifests. Fuzz has a separate workspace and was checked by source and manifest
inspection, not executed; its direct AST dependency remains intact.

## Audit and validation

- `RUSTFLAGS='-W unused -W unreachable_pub' cargo check -p shuck-ast -p
  shuck-indexer -p shuck-config -p shuck-discover -p shuck-cache -p shuck-run
  --all-targets --all-features`: no source warnings in the six crates. Dependency warnings
  from formatter, semantic and linter are outside this slice; their visibility
  findings are not evidence for removing small-crate APIs. The audit excludes
  these three dependency warning groups because edits to those crates are
  outside the assigned scope.
- Neither cargo-machete nor cargo-udeps was installed. Manual manifest audit
  matched each remaining dependency to source or test references, including
  derive macros and the separate fuzz manifest. No unused feature was found.
- `cargo test -p shuck-config`, `make test`, and
  `cargo clippy --workspace --all-targets -- -D warnings` are the focused,
  workspace consumer, and lint checks respectively; each exited 0. The config
  run passed 78 unit and 5 public API tests. The workspace suite ran under
  `cpu-guard --cores 4`. All cargo commands used the worktree's `.work/tmp` as
  TMPDIR. Cargo also reported manual-readme and homepage metadata warnings and
  a benchmark `tikv-jemallocator` unused-dependency warning; these are outside
  this slice and the manifest warnings do not make Clippy exit nonzero.

## Owner decision

`SelectionRangeIndex::selection_chain` and `Indexer::is_continuation` have
only in-crate test consumers in the searched workspace, but are public index
queries on a published crate. `shuck-discover::discover_files` likewise has
only in-crate test consumers. Removing them would change a public API, so they
remain walk-back candidates for an owner. This is not a claim that external
consumers do or do not exist.
