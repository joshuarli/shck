# Formatter dead-code audit

Scope: `crates/shuck-formatter`, on the pinned nightly and aarch64 macOS.
The crate has no build script, target-specific dependencies or explicit binary
registration; its library and integration tests are Cargo entry points. The only
feature, `benchmarking`, is enabled by `crates/shuck-benchmark/Cargo.toml` and
its `build_formatter_facts` entry point is used by `benches/formatter.rs`.
Both default and all-feature builds were checked on the host; other platforms
were inspected but not compiled or exercised. No public API or output was
intentionally changed.

## Removals

All five removed functions had `#[allow(dead_code)]` and no conditional
compilation. Searches covered all workspace source and manifests, including CLI,
config, benchmark, and the separate fuzz workspace, plus build and package
references. No registration, build script, feature or package entry refers to
any removed function. They were internal and could not be external entry points.

- `command::spans::words_span`: no callers anywhere; existing statement and
  compound span entry points use other helpers. Removed the unused function.
- `CompoundBodySite::brace` and `CompoundBodyBounds::split`: `brace` had no
  callers; `split` was called only by `brace`. The live body-site constructors
  serve loops, branches and function groups through facts and streaming. Removed
  both functions and imports used only by `brace`; the live function-group
  constructor remains.
- `write_parenthesized_word_list` and `write_space_separated_words`: the former
  had no callers and the latter was called only by the former. Streaming's live
  `write_for_in_words` uses its own word-list path. Removed both functions.

The formatter tests and workspace suite below exercise the local path dependency
used by CLI, config and benchmark. Fuzz targets were inspected, not executed;
their formatter dependency and target registrations are commented out in
`fuzz/Cargo.toml`. These removals cannot affect those consumers because none
can call the internal functions.

The private `comments` and `simplify` modules had reachable in-crate items
spelled `pub`; their visibility is now `pub(crate)`, with no external API
change. The unused `command` re-export of `normalized_close_keyword_span` was
removed; its defining module still uses the helper.

## Audit and validation

All formatter manifest dependencies have source or test references: `memchr`,
`rustc-hash`, AST, indexer, parser and `similar` in production; `rayon`,
`deser`, `deser-yaml`, `sha2` and linter in tests. No unused feature was found.
`cargo-machete` and `cargo-udeps` were unavailable; manifest and source
searches were used for dependency evidence. The debloat procedure was consulted;
its broader live-code and structural sweep is outside this dead-code-only slice. No dead enum variants or
unreachable branches were found by the compiler checks. This is not a proof of
all possible semantic reachability.

Compiler warning allowlist: Cargo's `manual_readme` advisory for
`crates/shuck-formatter/Cargo.toml:12` is pre-existing package metadata; the
explicit README is not dead code. The two compiler scans emitted no formatter
source warnings after the changes. Other crates and third-party dependencies
emit warnings under these flags and are outside this slice.

Validation, with `TMPDIR="$PWD/.work/tmp"` for each command:

- `RUSTFLAGS='-W unused -W unreachable_pub' cargo check -p shuck-formatter
  --all-targets --all-features`: exit 0; no formatter source warnings.
- `RUSTFLAGS='-W unused -W unreachable_pub --force-warn dead_code' cargo check
  -p shuck-formatter --all-targets --all-features`: exit 0; no formatter
  source warnings, including suppressed dead code.
- `cpu-guard --cores 4 -- cargo test -p shuck-formatter`: exit 0; 551 format
  cases passed. Two optional shfmt oracle tests were ignored by default.
- `cpu-guard --cores 4 -- make test`: exit 0; workspace consumer tests passed
  against this checkout's path dependencies.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0. Cargo also
  reported its pre-existing formatter README metadata advisory and the
  benchmark's unused `tikv-jemallocator` manifest advisory.
