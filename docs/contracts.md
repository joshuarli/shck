# Contracts and source directives

Shuck prefers source. When a sourced helper, framework module, or plugin entry point can be loaded, Shuck parses it, follows its source closure, and derives facts from real code. Contracts describe only behavior that source cannot recover: names provided by a runtime, external consumers of a script's variables, or a framework convention that is not visible in reachable source.

A contract compiles into `shuck_semantic::FileContract` and is applied before source closure. Lint rules consume the resulting semantic facts, not contract configuration.

## User contracts

Configure user contracts in `[lint.contracts]`:

```toml
[lint.contracts]
well-known = true
disabled = ["bash-it"]

[[lint.contracts.custom]]
id = "example/runtime"
when = "always"
files = ["scripts/*.sh"]
reads = ["EXAMPLE_HOME"]
provides.variables = ["EXAMPLE_MODE"]
consumes.prefixes = ["EXAMPLE_"]
```

- `well-known` enables the built-in registry and defaults to `true`.
- `disabled` removes built-ins by id, group, or `*`.
- `custom[].when` currently accepts only the string `"always"`; typed activation tables are rejected.
- `files` accepts globs relative to the project root; `!pattern` excludes. A list of only negations includes by default.
- Custom ids are checked for uniqueness among custom contracts, not against built-in ids.
- Custom contracts cannot declare ambient-only variables; `provides.variables` always means initialized at file entry.

## Effects

| Field | Meaning |
| --- | --- |
| `reads` | Names read at file entry. This creates a real read, so undefined or uninitialized analysis can report it. |
| `consumes.names` | Non-local assignment-like bindings with these names are consumed outside the lexical source and are not reported as unused. |
| `consumes.prefixes` | Same as `consumes.names`, matched by prefix. |
| `consumes.all` | Every non-local assignment-like binding in the file is externally consumed. |
| `provides.variables` | Variables that exist and are initialized at file entry. |
| `provides.ambient-variables` | Built-in YAML only: variables available before file entry without asserting an initializer. |
| `provides.functions` | Functions that exist at file entry. |
| `functions[].reads` | Names read when the provided function is called. |
| `functions[].sets` | Caller-visible variables set by the provided function. |

`reads` and `consumes` are not interchangeable. `reads` creates a synthetic reference; `consumes` only exempts assignments from unused-assignment reporting. When several contracts apply, provided bindings and functions merge and certainty degrades from definite to possible if any contributor is only possible.

## Built-in declarative contracts

Built-in contracts are authored as YAML under `contracts/` and compiled into `shuck-linter` at build time. Runtime linting never parses YAML.

```text
contracts/
  bash/completion.yaml
  bash-it/theme.yaml
```

Each file is a `version: 1` document:

```yaml
version: 1
contracts:
  - id: bash/example
    groups:
      - bash
    label: Example contract
    when:
      type: always
    files:
      - "**/example.sh"
    effects:
      consumes:
        names:
          - EXAMPLE_STATE
```

- `id` is the stable selector and must not contain whitespace.
- `groups` are stable selector aliases. They are required, non-empty, and must differ from `id`.
- `label` is display-only; it never participates in selection or cache identity.
- `when.type` is `always` in shipped contracts. The schema also parses `zsh_plugin` and `zsh_theme`, but no current producer activates them.
- `files` restricts activation by path glob.
- `effects` must contain at least one effect.
- `match` is allowed only for `always`. Predicates scan source text or the observed command stream: substring and name-mention scans, assignment-shape scans, function-definition shape, `source` commands, and the completion-initializer signal. Predicate names and semantics live in `crates/shuck-linter/src/ambient_contracts/` and are validated by `crates/shuck-linter/build.rs`.

Build validation rejects a wrong version, duplicate ids across files, empty or malformed ids and groups, groups equal to their id, invalid activation combinations, `match` on non-`always` activations, empty effects, and invalid file globs. Duplicate effect entries are deduplicated rather than rejected. Contract identity comes from the document `id`, not from the file path.

The build script writes generated descriptors to `OUT_DIR`; `ambient_contracts/contracts.rs` includes them. Descriptors hold static data plus lazy compiled matchers and materialized contracts. `crates/shuck-linter/contracts/` is the crate-local path used by the build and links to the root tree.

The `contracts/` tree currently contains a bash-completion ambient contract and a bash-it theme palette contract. The completion contract provides ambient completion variables and activates on the completion-initializer signal; the palette contract provides theme color variables.

## Activation flow

1. Config resolution merges `[lint.contracts]` and builds `ResolvedAmbientContracts`.
2. Enabled built-ins are those whose activation applies and whose selectors are not disabled; custom contracts are compiled alongside them.
3. `build_linter_semantic_artifacts` creates an `AmbientContractCollector` with the source, optional path, shell, and resolved contracts.
4. The collector checks activation, shell, file globs, and source match predicates, then merges matched contracts.
5. `build_semantic_model` applies the file-entry contract and any explicit contract before source closure. Helper summaries apply the same collector through `FileEntryContractCollectorFactory`.
6. Semantic application emits reads, provided bindings and functions, and external-consumption marks. Applying contracts re-resolves unresolved references and invalidates the call graph.

Stdin and path-less inputs have no file-entry contract because matching requires a path.

The effective contract state is part of the check cache key. Built-in YAML is compiled into the binary, so its mtimes are not runtime cache inputs; rebuilding the binary changes the cache root.

## Computed source directives

When a `source` or `.` path is computed, Shuck normally reports an untracked source (C003). A directive on or above the `source` line names the intended target:

```sh
# shuck: source=lib/util.sh
source "$DIR/util.sh"

# shuck: source=lib/util.sh lint=true
source "$SHARED_DIR/util.sh"

# shuck: source=/dev/null
source "$maybe_present"
```

Syntax and parsing:

- The prefix is case-insensitive `# shuck:`. The first `source=` token sets the target and the first `lint=true|false` token sets the lint policy; other `lint=` values are ignored.
- A `lint=` token without `source=` is not a source directive.
- `source=/dev/null` means "nothing to follow".
- A same-line directive attaches to that line's source command. An own-line directive attaches to the next source command; contiguous comment lines can carry it forward.
- An own-line `source=/dev/null` persists as the default for later source commands until another source directive overrides it.
- `# shellcheck source=<path>` is recognized with `lint=false`. `# shellcheck source-path=ROOT` adds a search root for later source commands; multiple comments accumulate. `SCRIPTDIR` means the annotating file's directory.

Resolution is first-match-wins:

1. the annotating file's own directory
2. `# shellcheck source-path=` roots
3. `[lint] source-paths` roots, relative to the project root

Absolute directive paths short-circuit the search. The literal-shortcut for `$(dirname "$0")/tail` is recognized conservatively. Importing symbols can follow `..` and absolute paths; only `lint=true` targets are confined to the referrer's project root when collected as extra inputs.

A resolved native `# shuck: source=` directive marks the reference as explicitly provided, which silences both the dynamic-path and untracked-source diagnostics. The `# shellcheck source=` spelling resolves the target but does not set that flag, so C003 can still report. An unresolved native directive also still reports C003, so a typo stays visible.

| Behavior | `source=<path>` | `source=<path> lint=true` |
| --- | --- | --- |
| Import target symbols | yes | yes |
| Lint target diagnostics | no | yes, remapped to the target |
| Follow nested `lint=true` | no | yes, cycle-safe |
| Add target as an input | no | yes |
| Auto-fix target | no | no |

Config:

```toml
[lint]
source-paths = ["lib", "scripts"]
lint-sources = true
```

`lint-sources = false` downgrades `lint=true` to symbol import only. `source-paths` roots accept a relative path, a subpath joined onto each root, or `SCRIPTDIR`. The ShellCheck compatibility flags `--external-sources` and `--source-path` remain separate and drive the same closure machinery.

## Not implemented

- `zsh_plugin` and `zsh_theme` activations, plugin requests, and request-site contracts have schema and plumbing but no active producer.
- Custom TOML contracts support only `when = "always"` and cannot declare ambient-only variables.
- Helper dependency paths from nested source closure are not currently included in cache fingerprints or watch targets.
