# Declarative Built-In Contracts

This directory contains Shuck's repository-authored built-in ambient contracts.
They are source files for the build, not runtime configuration files.

## Source First

Before adding a contract, ask:

> Could Shuck load real source and derive this fact instead?

If yes, prefer improving source resolution, source closure, helper
summarization, or deferred runtime modeling.

Use a declarative contract only for residual behavior such as runtime-provided
names available before file entry, or stable framework conventions that are not
recoverable from reachable source.

## Layout

Contracts live under `contracts/`, grouped by ecosystem:

```text
contracts/
  bash/
  bash-it/
```

Each YAML file uses this shape:

```yaml
version: 1
contracts:
  - id: bash/example
    groups:
      - bash
      - bash/example
    label: Example contract
    when:
      type: always
    effects:
      consumes:
        prefixes:
          - EXAMPLE_
```

## Fields

- `id`: Stable built-in selector id.
- `groups`: Stable built-in selector groups.
- `label`: Optional human-facing label.
- `when`: Activation selector (`type: always` is the common form for residual
  ambient contracts).
- `match`: Optional predicates supported by the current ambient-contract
  implementation.
- `files`: Optional path filters.
- `effects.reads`: Names read at activation time.
- `effects.consumes.names`: Exact names consumed outside the lexical source.
- `effects.consumes.prefixes`: Name prefixes consumed outside the lexical
  source.
- `effects.consumes.all`: Escape hatch for consuming all non-local
  assignments.
- `effects.provides.variables`: Definite initialized file-entry variables.
- `effects.provides.ambient-variables`: Ambient variables that exist before
  file entry but are not initialized by the file itself.
- `effects.provides.functions`: Definite function bindings.
- `effects.functions`: Function-specific caller reads and sets.

## Validation

The `shuck-linter` build script validates contract schema, duplicate ids,
selector tokens, file globs, and effect/activation shapes.

## Commands

```bash
cargo test -p shuck-linter build_script
cargo test -p shuck-linter ambient_contracts
cargo test -p shuck-cli contract
```
