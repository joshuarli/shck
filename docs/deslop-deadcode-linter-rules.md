# Linter rules and contracts dead-code audit

Scope: linter outside `facts/`, at base `d53111b180c46c75b64a0e2e9d902e13d84ec843`, pinned nightly, macOS. The registered rules and codes, diagnostics, fixes and fixtures remain unchanged. The linter's `benchmarking` feature and all targets were included in the compiler checks; other platforms were not compiled.

## Removals

- The literal `if false` return in `glob_in_string_comparison`: it could never execute. The C081 rule remains registered in `registry.rs`, dispatched in `checker.rs`, and covered by its rule tests and fixture. Searches across workspace crates, fuzz, docs, scripts, manifests and Makefile found no separate entry, build or package reference to the branch. The rule has no platform or feature gate.
- The empty `WELL_KNOWN_REQUEST_CONTRACTS` table and its `WellKnownRequestContract`, `RequestActivation`, lookup, matcher and unreachable request handling. The table had no entries; workspace and fuzz searches found no constructor, registration, build or package reference. Declarative generated contracts and custom request contracts remain active. There was no platform or feature gate on the removed path.

- `first_shell_command_segment` in ambient signals had no callers. The `zmodload_lines` collection and `loads_zsh_module` query have only two unit-test callers, so they now compile only for tests; the unused `trim_start` local was removed. Workspace, fuzz, docs, scripts and manifest searches found no other consumers or registrations. These are not feature or platform gated.
- The unannotated, uncalled `generate_declarative_contract_data_renders_runtime_descriptors` in `tests/build_script.rs` was not registered as a test; workspace and fuzz searches found no caller. The build generator remains used by `build.rs`.
- Test helpers `request_contract_for`, `has_consumed_prefix`, `has_consumed_name` and `plugin_request` had no callers in the workspace, fuzz, docs, scripts or manifests. They had no feature or platform gates.

Internal visibility was narrowed for the parse-diagnostic violation types, common word helper and unquoted-expansion safe-value module. Their containing modules are private or crate-private, and their existing consumers remain in the linter. No exported API changed.

Consumer tests for each removal: `TMPDIR="$PWD/.work/tmp" cpu-guard --cores 4 -- cargo test -p shuck-linter` and `TMPDIR="$PWD/.work/tmp" cpu-guard --cores 4 -- make test`, using local workspace crates. Fuzz was searched but not run.

## Warning allowlist

- `DeclarativeContractDescriptor::label` is emitted by the build generator from contract metadata; removing it requires changing the generator outside this slice.
- `DeclarativeActivationDescriptor::{ZshPlugin,ZshTheme}` and `DeclarativeShellDescriptor::{Zsh,ZshOrUnknown,ZshRuntime}` are emitted by the build generator for supported declarative contract inputs, even though current inputs do not instantiate them. Retain the generator contract.

`cargo-machete` and `cargo-debloat` are unavailable. Dependencies and binary size were not changed in this slice.

## Validation

On the candidate, the linter test and workspace test commands above exited 0; the linter suite reported 3,631 unit tests passed. `TMPDIR="$PWD/.work/tmp" cargo clippy --workspace --all-targets -- -D warnings` exited 0. `TMPDIR="$PWD/.work/tmp" RUSTFLAGS='-W unused -W unreachable_pub' cargo check -p shuck-linter --all-targets --all-features` exited 0 with no warnings in this slice. The same command with `--force-warn dead_code` appended to `RUSTFLAGS` exited 0 and reported only the three allowlisted findings above in this slice. The checks also report findings in `facts/`, other crates, generated files and included build-script code outside this scope. Workspace tests include ignored opt-in oracle tests; no external oracle or fuzz test was run.
