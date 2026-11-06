# shuck

A fast shell script linter and formatter, written in Rust.

Shuck parses, analyzes, and formats shell scripts. It catches bugs, style issues, security hazards, performance traps, and portability problems, and formats shell sources with configurable layout rules. A caching layer keeps incremental runs fast.

## Features

- Fast — benchmarks compare against ShellCheck on real-world scripts
- Linting with rules across correctness, security, performance, portability, and style categories
- Safe and unsafe fix support for selected diagnostics
- Shell formatting with write, check, diff, stdin, and configuration-file modes
- Multi-dialect support: bash, sh/POSIX, dash, ksh, and mksh
- Automatic file discovery via extensions and shebang detection
- ShellCheck suppression compatibility (`# shellcheck disable=SC2086`)

## Installation

### Homebrew

```sh
brew install ewhauser/tap/shuck-cli
```

### PyPI

```sh
pip install shuck-cli
```

The PyPI package is named `shuck-cli`, but it still installs the `shuck`
command.

### Cargo

```sh
cargo install shuck-cli
```

### Pre-built binaries

Pre-built binaries are published on the [releases page](https://github.com/ewhauser/shuck/releases) for macOS (aarch64) and Linux (x86_64 and aarch64).

### GitHub Actions

Use [`ewhauser/shuck-action`](https://github.com/ewhauser/shuck-action) to install Shuck and report lint findings as native GitHub annotations:

```yaml
steps:
  - uses: actions/checkout@v4
  - uses: ewhauser/shuck-action@v1
    with:
      mode: check
      paths: |
        scripts
```

See the action's documentation for setup-only and formatting modes, version pinning, and additional inputs.

## Usage

| Workflow | Command |
| --- | --- |
| Lint files and directories | `shuck check` |
| Format shell files | `shuck format` |
| Run a script with a managed shell interpreter | `shuck run` |
| Pre-install a managed shell interpreter, or list available versions | `shuck install` |
| Start an interactive session with a managed shell interpreter | `shuck shell` |
| Remove project cache entries | `shuck clean` |

### Lint

```sh
# Check files and directories
shuck check script.sh src/

# Check the current directory
shuck check .

# Read from stdin
echo 'echo $foo' | shuck check -

# Read from stdin using a logical filename for dialect and project settings
echo 'echo $foo' | shuck check --stdin-filename script.bash

# Apply safe fixes automatically
shuck check --fix .

# Apply opt-in unsafe fixes too
shuck check --unsafe-fixes .

# Skip the cache
shuck check --no-cache .

# Override the cache location
shuck --cache-dir .tmp/shuck-cache check .
```

### Format

```sh
# Format files in place
shuck format .

# Check formatting without modifying files
shuck format --check .

# Show unified diffs for files that would change
shuck format --diff .

# Format stdin and infer the dialect from a filename
printf 'foo(){\necho hi\n}\n' | shuck format --stdin-filename script.bash -

# Override formatting options for a single run
shuck format --indent-style space --indent-width 2 --function-next-line .
```

`shuck format` uses the same file discovery, exclusion, gitignore, project-root, and cache behavior as `shuck check`, but it only rewrites standalone shell files.

In `--check` and `--diff` modes, `shuck format` exits with status `1` when files would change and leaves the files untouched. Formatter parse errors exit with status `2`.

### Clean caches

```sh
# Remove cache entries for the current project
shuck clean
```

## Output

`shuck check` prints rich code-frame diagnostics by default:

```
warning[C001]: variable `tmp` is assigned but never used
 --> deploy.sh:14:1
   |
14 | tmp=$(mktemp)
   | ^^^
   |
```

Use `--output-format concise` for one-line diagnostics, or `json` and `json-lines` for machine-readable output:

```
path:line:col: severity[CODE] message
```

```
deploy.sh:14:1: warning[C001] variable `tmp` is assigned but never used
deploy.sh:31:10: error[C006] undefined variable `DEPLY_ENV`
deploy.sh:45:3: warning[S005] legacy backtick command substitution
```

### Exit codes

| Code | Meaning |
| --- | --- |
| `0` | No issues found |
| `1` | Lint violations or parse errors detected |
| `2` | Runtime error (bad arguments, I/O failure) |

## Rules

Shuck ships with rules organized into five categories:

| Category | Prefix | Description |
| --- | --- | --- |
| Correctness | C | Bugs, errors, and likely mistakes. Enabled by default. |
| Style | S | Code quality and best-practice suggestions. |
| Performance | P | Inefficient patterns that have simpler or faster alternatives. |
| Portability | X | Bash-isms and shell-specific constructs that break under POSIX or other shells. |
| Security | K | Potentially dangerous shell patterns such as risky deletion, unsafe evaluation, or local expansion. |

Each rule has a short code (e.g., `C006`, `S001`) that appears in diagnostics and can be used in suppression directives. Diagnostics are classified as error, warning, or hint depending on severity.

### ShellCheck compatibility

Where possible, shuck rules align with ShellCheck rules. Shuck supports ShellCheck suppression syntax (`# shellcheck disable=SC2086`) and maps ShellCheck codes to their shuck equivalents, so existing suppression comments continue to work without changes. Both suppression syntaxes accept either code namespace, and native `# shuck: disable=...` follows ShellCheck's scope rules: before the first statement it is file-wide, otherwise it applies to the next command.

Shuck is not a port of ShellCheck. It is a clean-room reimplementation built on its own parser and analysis engine, so results will sometimes differ. Compatibility is continuously validated against a large corpus of popular open-source shell projects; the corpus is defined in [`scripts/corpus-download.sh`](scripts/corpus-download.sh).

## Suppression

Suppress diagnostics with inline comments. Both native and ShellCheck-style directives are supported.

```sh
# Suppress a specific rule for the next command
# shuck:disable=C001
unused_var="ok"

# Suppress multiple rules
# shuck:disable=C001,S001
code_here

# Suppress for the entire file (place anywhere)
# shuck:disable-file=S001,S002

# ShellCheck-compatible syntax (also works)
# shellcheck disable=SC2034,SC2086

# Code aliases are interchangeable in either style
# shuck: disable=SC2086
# shellcheck disable=S001

# Before the first statement, disable becomes file-wide
# shuck: disable=S001
```

## Sourced files

When a `source`/`.` path is computed (for example `source "$DIR/lib.sh"`), shuck
usually cannot resolve it statically and reports it as an untracked source.
Current-file anchors such as `$(dirname "${BASH_SOURCE[0]}")/lib.sh` are
recognized conservatively. For other computed paths, point shuck at the real
file with a hint comment on (or just above) the `source` line:

```sh
# Import the file's definitions so references resolve, and stop the
# untracked-source warning. The file itself is not linted.
# shuck: source=lib/util.sh
source "$DIR/util.sh"

# Same, and also lint the target as an additional input.
# shuck: source=lib/util.sh lint=true
source "$DIR/util.sh"

# Nothing to include here; just silence the warning.
# shuck: source=/dev/null
source "$maybe_present"
```

The path is resolved relative to the annotating file's directory, then against
any configured `source-paths`; the nearest match wins. A `lint=true` target is
linted like a directly checked file: its own `source` statements are imported
for symbol resolution, and nested `lint=true` directives inside it are honored
transitively. The ShellCheck-compatible `# shellcheck source=<path>` directive
is recognized too.

Configure resolution and target linting in `.shuck.toml`:

```toml
[lint]
# Extra directories (relative to the project root) searched when resolving
# `# shuck: source=` directive targets.
source-paths = ["lib", "scripts"]

# When false, lint=true directives only import symbols; the targets are not
# linted. Default: true.
lint-sources = true
```

When the target lives somewhere the annotating file cannot reach relatively,
give the hint only the file name and add its directory to `source-paths`:

```sh
# scripts/deploy.sh
# shuck: source=util.sh lint=true
source "$SHARED_DIR/util.sh"
greet   # defined in lib/util.sh
```

```toml
# .shuck.toml (at the project root)
[lint]
source-paths = ["lib"]
```

Resolution tries the hint against the annotating file's own directory, then each
`source-paths` root. The hint may be a bare name (`util.sh`), a subpath
(`net/http.sh`), or an absolute path. `source-paths` roots are relative to the
project root; the token `SCRIPTDIR` means the annotating file's directory.

## Configuration

Project settings live in `.shuck.toml` or `shuck.toml`. Shuck walks up from each
input to find the nearest of these files and treats that directory as the project
root.

If no project config is found, Shuck falls back to a user-level global config at
`$XDG_CONFIG_HOME/shuck/shuck.toml` (or `~/.config/shuck/shuck.toml`) and
accepts `.shuck.toml` there too. Set `SHUCK_CONFIG_HOME` to point at a different
directory for the global config. A project config always takes precedence over
the global one, and `--config <file>` or `--isolated` bypass global config
entirely.

```toml
[per-file-shell]
# Override shell dialect inference for matching files in check and format.
"scripts/bash/**" = "bash"
"vendor/**/*.sh" = "sh"
"dot_env*" = "bash"

[format]
# Configure `shuck format`.
exclude = ["generated/**"] # project-relative globs that should never be formatted
indent-style = "tab"       # tab | space
indent-width = 4           # used when indent-style = "space"
binary-next-line = false   # put binary operators on continuation lines
switch-case-indent = false # indent case branch bodies
space-redirects = false    # add spaces around redirection operators
keep-padding = false       # preserve safe horizontal padding
function-next-line = false # put function opening braces on their own line
never-split = false        # prefer compact layouts
```

`format.exclude` applies to directory discovery and explicitly named files. This
makes it suitable for generated files that must remain untouched.

`per-file-shell` patterns are resolved relative to the project root and apply to
`shuck check` and direct and stdin formatting. Supported values are `sh`, `bash`,
`dash`, `ksh`, and `mksh`; generic `ksh` mappings remain available to the linter
but cannot be used by the formatter. Overlapping patterns may select the same
dialect, but selecting different dialects for one file is an error. Existing
`[lint].per-file-shell` and `[lint].extend-per-file-shell` settings remain
supported as compatibility aliases.

An explicit `shuck format --dialect bash|posix|mksh` override takes precedence
over `per-file-shell`. Unmatched files continue to infer their dialect from the
file name, shebang, and source.

## File discovery

When given a directory, shuck recursively discovers standalone shell scripts by:

1. **Extension**: `.sh`, `.bash`, `.ksh`, `.dash`, `.mksh`, `.bats`
2. **Shebang**: files starting with `#!/bin/bash`, `#!/usr/bin/env sh`, etc.

The following directories are skipped by default: `.git`, `.hg`, `.svn`, `.jj`, `.bzr`, `.cache`, `node_modules`, `vendor`, `.shuck_cache`.

Gitignore and `.ignore` files are respected by default. Use `--no-respect-gitignore` to disable.

## Caching

Shuck caches lint and format results per file in a shared cache root outside the project tree by default. The default location follows the OS cache directory convention, which is typically `~/Library/Caches/shuck` on macOS and `$XDG_CACHE_HOME/shuck` or `~/.cache/shuck` on Linux.

Override the cache root with `--cache-dir` or `SHUCK_CACHE_DIR`.

Disable caching with `--no-cache` or remove a project's cache entries with `shuck clean [PATH]`.

## Rust API

Shuck's parser and analysis crates are published for embedders. Their supported pre-1.0 integration path and selective compatibility guarantees are documented in [Rust API compatibility](docs/rust-api-compatibility.md).

## Acknowledgements

Shuck builds on ideas and inspiration from several open-source projects. This section is a thank-you to those communities — it does not imply endorsement, affiliation, or any formal relationship between shuck and these projects.

- **[bashkit](https://github.com/everruns/bashkit)** — shuck-parser was originally forked from bashkit's bash lexer and parser; it has since evolved substantially for linter needs (comment and trivia preservation, error recovery, multi-dialect views, extended AST coverage).
- **[Ruff](https://github.com/astral-sh/ruff)** — Linter architecture inspiration, particularly caching, rule organization, and diagnostic output.
- **[ShellCheck](https://github.com/koalaman/shellcheck)** — The original source of inspiration for shuck.
- **[gbash](https://github.com/ewhauser/gbash)** — Lessons from this earlier project carried forward into shuck.

## License

MIT
