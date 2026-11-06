# Managed shells

`shuck-run` resolves, installs, and executes shell interpreters. It backs three commands:

- `shuck run [options] SCRIPT [ARGS...]` resolves an interpreter and executes the script.
- `shuck install [options] [SHELL] [VERSION]` installs or lists managed interpreters.
- `shuck shell [options]` starts an interactive session with a managed interpreter.

## Supported shells

`Shell` is `bash`, `gbash`, `bashkit`, `dash`, `mksh`, or `busybox`. `sh` aliases to `dash` and `ksh` to `mksh`. Busybox is Linux-only. Zsh is not managed.

`Shell::infer(source, path)` reads the shebang, then the file extension. If neither identifies a shell, resolution falls through to the default `bash`.

Versions use tokenized constraints with prefix matching, so `5.2` accepts `5.2.x`. A resolved interpreter records the shell, version, path, and whether it is managed or the system binary.

## Resolution

Shell precedence:

1. `--shell`
2. inline `# /// shuck` metadata
3. `[run].shell`
4. shebang inference
5. extension inference
6. `bash`

Version precedence:

1. `--shell-version`
2. inline metadata, only when the metadata shell matches the resolved shell
3. `[run].shell-version`
4. `[run.shells].<shell>`
5. `latest`

`shuck run` falls back to the system interpreter when every source is implicit and no version constraint is requested. `shuck shell` does not. An explicit `--system` always uses the system binary and validates the requested constraint against its `--version` output. A shell name without a version resolves to `latest`, not to the system interpreter.

## Inline metadata

A script can declare its interpreter in a comment block:

```sh
# /// shuck
# shell = "bash"
# version = ">=5.1"
# ///
```

The block is TOML after stripping `# ` and must appear before the script body. More than one block, a block after the body, or an unknown key is an error. Metadata supplies the shell and an optional version constraint.

## Registry, cache, and installation

The cache root is `~/.shuck/shells/`; `SHUCK_SHELLS_DIR` overrides it. The registry root defaults to `https://ewhauser.github.io/shuck-shells/`; `SHUCK_RUN_REGISTRY_URL` overrides it. Release tarballs come from the URLs in the registry manifests.

- The root index is cached as `index.json`; per-shell versions live under `shells/<shell>/`.
- Documents younger than 24 hours are trusted. `--refresh` forces a fetch. A failed fetch falls back to a stale cached document unless a refresh was requested.
- Platform keys are `x86_64-linux-gnu`, `x86_64-linux-musl`, `aarch64-linux-gnu`, `aarch64-linux-musl`, `x86_64-darwin`, and `aarch64-darwin`. Unsupported platforms are a hard error.
- Installation downloads to a temporary file, verifies the SHA-256 digest, extracts into a temporary directory in the cache, then renames it into place. The installed interpreter is smoke-tested with `--version`. Reinstalling the same version is a no-op.

## Execution

`shuck run` resolves the interpreter, then exports `SHUCK_SHELL`, `SHUCK_SHELL_VERSION`, and `SHUCK_SHELL_PATH`. For managed non-busybox shells it also sets `SHELL`.

- `--command CMD` runs `<shell> -c CMD shuck-run [args]`; it cannot be combined with a script path.
- A script path runs `<shell> script [args]`.
- `-` or no script reads from stdin with `-s [-- args]`.
- Bashkit uses a temporary script file, so the process waits instead of `exec`-replacing itself.
- On Unix the child replaces the shuck process, preserving the shell's PID, except when a temporary script must outlive it.
- `--dry-run` prints `shell version (path)` and does not execute.
- `--verbose` reports registry and download activity.

## Configuration

```toml
[run]
shell = "bash"
shell-version = "5.2"

[run.shells]
bash = "5.2"
mksh = "latest"
```

`[run.shells]` keys are validated against the supported shell names. Individual values can also be overridden with `--config run.shell-version=...` and the other global override forms.
