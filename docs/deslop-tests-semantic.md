# Semantic test audit

Scope: `crates/shuck-semantic` at base `b7f330cb9e3910bd868f5d0ed5cc540337c9ae8f`, macOS arm64, pinned nightly, default features. The CLI consumes `imported_dependency_paths` when collecting cache dependencies (`crates/shuck-cli/src/commands/check/analyze.rs` and `stdin.rs`). No production behavior changed.

## Pyramid and costs

| Tier | Selection and role | Baseline | Candidate |
| --- | --- | --- | --- |
| Edit | `cargo test -p shuck-semantic <test-name>`: individual behavior witness | Not timed | Fault witness below |
| Change | `cargo test -p shuck-semantic`: model, analyses, filesystem-backed source closure, scaling checks and a static source-traversal guard | 448 unit, 1 integration, 0 doc tests; test binaries reported 0.18 s and 0.03 s | 449 unit, 1 integration, 0 doc tests; test binaries reported 0.20 s and 0.05 s |
| Scenario | Named scaling tests in the same crate suite; source-closure fixtures use isolated temporary files | Included above | Included above |
| Workspace | `make test`, explicitly requested acceptance including downstream consumers | Passed; warm time recorded in AGENTS.md: 14.88 s | Timing pending host readiness |
| Full | Large corpus and benchmark Full tier | Not run | Not run |

The initial semantic build took 1m39s; binary execution costs above are runner-reported observations, not isolated performance measurements. Workspace warmup passed. Perf gate deferred: host busy. Repeated `perf-gate-ready` checks exited 75 (latest load 15.75 on 18 cores; limit 0.7 per core). No valid local before or after timing was taken, so the 5 s budget is unverified. The earlier 14.88 s is a project-recorded observation, not a paired local baseline. No flake was observed in the runs here; that is not an estimate of a flake rate. No memory measurement was requested or taken. There are no ignored semantic tests in the observed crate run.

`cargo-llvm-cov` and `cargo-mutants` are not installed or available on the crate mirror, as recorded by the campaign. Coverage, automated mutation, survivor dispositions and a qualified mutation lane remain unverified; this audit does not claim they passed. The hand-injected fault below is a single targeted sensitivity check, not a mutation sweep.

## Boundaries and witnesses

| Boundary | Guard |
| --- | --- |
| Model build entry point | `creates_file_and_function_scopes_and_resolves_local_shadowing` builds from parsed source and observes model contents |
| Bindings, references, scopes and resolution | `creates_file_and_function_scopes_and_resolves_local_shadowing` |
| CFG paths | `semantic_block_coverage_detects_uncovered_branch_paths` checks a branch that does not dominate its later read |
| Dataflow | `prefix_name_expansions_do_not_read_the_prefix_as_a_variable` checks both absent and present uninitialized results through the public analysis; `detects_overwritten_assignments_and_possible_uninitialized_reads` also checks internal overwritten and potentially uninitialized outcomes |
| Source closure | `layered_source_closure_imports_function_contracts_transitively`, now with an uninitialized positive control without closure |
| Imported dependency paths | `imported_dependency_paths_include_transitive_and_missing_sources`, added here; checks direct and transitive paths, deduplication and an absent path that can later be created |
| Oversized index | `oversized_semantic_index_is_rejected` checks the maximum and rejects the next value on 64-bit targets; 32-bit target not exercised |

## False passes and dispositions

- Fixed `commands_iter_filters_synthetic_ids_so_command_kind_is_safe`: an empty command iterator previously skipped its entire loop. It now requires the case command and nested `echo a` before querying every returned command kind.
- Strengthened `layered_source_closure_imports_function_contracts_transitively`: its absence-only uninitialized result could also pass if that analysis always returned no results. The same main script without closure must report `flag` as uninitialized.
- Retained internal parity and cache-reuse tests as supplementary checks: they cover different execution paths and query-order or reuse invariants, while the boundary witnesses above assert independently expected outcomes. No tests were removed or weakened.

## Fault proof

Temporarily changed `SemanticModel::imported_dependency_paths` to return `&[]`. `TMPDIR="$PWD/.work/tmp" cargo test -p shuck-semantic imported_dependency_paths_include_transitive_and_missing_sources` failed at its path equality assertion: actual `[]`, expected helper, loader and missing paths. The production change was reverted. This is the only new boundary test; the other two edits strengthen existing tests.
