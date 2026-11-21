# Buildlib adoption: Python wheel release builder

This is the BLD-3 record for `shck`. It covers the verified release-archive handling in
`scripts/build-python-release.py`; it does not change the release workflow, the published
asset names, the wheel layout or any public Make target.

## 1. Bound change

- **Base revision.** `9f56f4002090bdf186eba48d2798bb673959e531`.
- **Entrypoint.** `scripts/build-python-release.py` with the existing `build-wheel` and
  `build-release-wheels` commands, plus a new read-only `verify` command and a
  `provenance` command. Existing arguments, option names, exit behaviour and output
  wording are unchanged.
- **Local implementation replaced.** The local `sha256_file` loop over `hashlib` (six
  lines) and the unchecked sidecar digest. The script now calls
  `buildlib.hashing.sha256_file` and validates the published digest with
  `buildlib.hashing.is_sha256` before comparing it.
- **Buildlib API.** `buildlib.hashing.sha256_file`, `buildlib.hashing.is_sha256`, the
  `buildlib` package identity, and (in the tests) `buildlib.targets.TRIPLES`. The wheel
  is pinned at `laputa-buildlib==0.2.0` and imported through the PEP 723 environment, or
  through `bootstrap/buildlib-run` when uv is unavailable.
- **Supported targets.** The six wheel targets stay local:
  `aarch64-apple-darwin`, `aarch64-unknown-linux-gnu`, `aarch64-unknown-linux-musl`,
  `x86_64-pc-windows-msvc`, `x86_64-unknown-linux-gnu` and
  `x86_64-unknown-linux-musl`. Buildlib models only the three supported release triples
  and has no wheel-platform mapping; gnu, Windows and every wheel tag remain shuck
  policy. The tests assert that the local map still covers every buildlib triple.
- **Output and error contracts.** Archives, sidecars, wheel names and the build command
  are unchanged. Failures still use `SystemExit` with the same wording where the check
  already existed; a malformed sidecar digest now fails explicitly instead of being
  compared as text.
- **Consumer policy that stays local.** `TargetSpec` and its archive names, packaged
  binary names and wheel tags; the `<archive>.sha256` sidecar format; tar/zip extraction
  and its `data` filter; finding exactly one packaged binary; the setuptools wheel build;
  and the release asset completeness check. Buildlib has no extraction, sidecar or
  wheel-tag API, so none of that was replaced.
- **Missing input at the base revision.** The `python/` wheel source directory is absent
  at `9f56f40` (removed by `69da21aae3`, which kept the script). The outcome comparison
  in section 4 therefore materialises the same `python/` tree from the last revision that
  carried it, `69da21aae3^`, into a scratch root for both the base and the new script.
  No `python/` file is added to this slice.

## 2. Pins and provenance

Vendored from `laputa-systems/buildlib` at
`30ad4a0a8b697031a008b1de3b5359fc5cf6f2fe` (release 0.2.0), both files byte-identical to
that revision:

| File | SHA-256 |
| --- | --- |
| `bootstrap/buildlib-run` | `e7f957ca6af0b151d25a9ff5bedb5deca0bfc86ba05ff3225a5405d56fa95957` |
| `bootstrap/python.lock` | `cab50541ce8b2c2a6a41829f26aac19344532d239535f4d6795d80d5af933832` |

`bootstrap/python.lock` pins the interpreter archives, uv 0.13.7 with Python 3.15.2, and
wheel `laputa_buildlib-0.2.0-py3-none-any.whl` with SHA-256
`aaee620f0478544b1d32075b44de48e69bbcbd8b974d5f599d037494d59f4b4a`.

The PEP 723 header declares `laputa-buildlib==0.2.0` from the stuff index, and
`scripts/build-python-release.py.lock` records the same wheel digest
`aaee620f...f4b4a`. The script's `require_pinned_buildlib()` refuses any other installed
version, and `provenance` prints both the version and the imported path:

```
$ TMPDIR=$PWD/.work/tmp uv run --script scripts/build-python-release.py provenance
laputa-buildlib 0.2.0 at .../environments-v2/build-python-release-.../lib/python3.15/site-packages/buildlib/__init__.py
```

The no-uv path imports the wheel directly from the bootstrap cache:

```
$ BUILDLIB_CACHE=$PWD/.work/tmp/buildlib-cache bootstrap/buildlib-run -- \
    -c 'import buildlib; print(buildlib.__file__)'
.../.work/tmp/buildlib-cache/aaee620f0478544b1d32075b44de48e69bbcbd8b974d5f599d037494d59f4b4a/buildlib.whl/buildlib/__init__.py
```

Package pin, uv lock and bootstrap lock all name the same wheel digest. The source
revision is recorded separately from the package version. There are no local differences
in either vendored file. Reverting this slice restores the base script and leaves every
pin untouched; no release archive, sidecar or lock format changed.

## 3. Boundary tests

`tests/test_build_python_release.py` runs the real script boundary through
`bootstrap/buildlib-run` (18 tests, `make test-python-release`, `Ran 18 tests ... OK`).
The injected faults each fail before any wheel is built:

| Fault | Injected by | Observed error |
| --- | --- | --- |
| wrong digest | published digest zeroed, valid archive | `checksum mismatch ...: expected 000...` |
| malformed digest | sidecar digest replaced with text | `invalid sha256 in ...` |
| truncated archive | archive cut short, original digest | `checksum mismatch ...` |
| wrong asset name | sidecar names another asset | `lists other.tar.xz, expected ...` |
| missing sidecar | sidecar removed | `missing checksum file for release archive ...` |
| empty sidecar | zero-byte sidecar | `empty checksum file ...` |
| missing binary | archive without `shuck` | `expected exactly one shuck inside ...` |
| two binaries | two `shuck` entries | `expected exactly one shuck ... found 2` |
| wrong target | `riscv64-unknown-linux-musl` | argparse `invalid choice` and exit 2 |
| incomplete assets | empty assets directory | `missing expected release assets` |

A wrong size is not a separate pin in shuck: the published sidecar records only the
digest, so a truncated or oversized archive is rejected by the digest comparison. The
tests also prove the six-target map covers every buildlib triple and that the imported
buildlib is 0.2.0.

## 4. Outcome comparison

Both packagers were run with the same reconstructed `python/` tree and the same fixture
binary for `aarch64-apple-darwin` (the host target), in scratch roots that differ only in
the script version:

| Measurement | Base script | New script |
| --- | --- | --- |
| Wheel name | `shuck_cli-0.2.8-py3-none-macosx_11_0_arm64.whl` | same |
| Zip member names, in order | 8 members | same |
| Member bytes | `METADATA b45d7e9b...`, `WHEEL ccada4d0...`, `RECORD 835096c8...`, `bin/shuck 4c324b93...` | identical |
| Second run, member bytes | identical to the first | identical to the first |

The whole-wheel SHA-256 differs between runs only because setuptools stamps the zip
members with the build time: base run 1 `086eee1f...` versus run 2 `597fdb38...`, and new
run 1 `c2cd0a25...` versus run 2 a different digest. The member names, member bytes,
`METADATA`, `WHEEL` and `RECORD` are byte-identical between base and new and across two
runs. This is the base builder's pre-existing metadata nondeterminism, not a change in
this slice; the plan's rule against requiring equality with a previously nondeterministic
archive applies, and no archive, manifest or checksum asset is published.

The builder emits no `SHA256SUMS` or manifest of its own; it consumes the release
sidecars and the wheel's `RECORD` carries the checksums. Those `RECORD` bytes are equal
before and after.

## 5. Environment

**Host.** macOS 27.2 arm64, Python 3.15.1, uv 0.13.9. Both trees used
`CARGO_TARGET_DIR=.work/tmp/cargo-shared` and `TMPDIR=.work/tmp`.

| Command | Base `9f56f40` | After |
| --- | --- | --- |
| `TMPDIR=.work/tmp make test` | exit 0, 41 suites, 5932 passed, 0 failed, 18 ignored | exit 0, 41 suites, 5932 passed, 0 failed, 18 ignored |
| `TMPDIR=.work/tmp make lint` | exit 2, `No rule to make target 'lint'` | exit 2, same |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | exit 101, `wildcard_enum_match_arm` at `crates/shuck-benchmark/examples/large_corpus_profile.rs:590` | exit 101, same error |
| `TMPDIR=.work/tmp make check-scripts` | exit 0 | exit 0 |
| `TMPDIR=.work/tmp make test-python-release` | not applicable (no script) | exit 0, 18 tests |
| `uv run --script scripts/build-python-release.py provenance` | not applicable | exit 0, buildlib 0.2.0 |
| `uv run --script scripts/build-python-release.py verify ...` | not applicable | exit 0 |

The clippy failure and the absent `lint` target are pre-existing at the base revision:
the repository's lint entrypoint is `make check`, and its clippy step fails there too.
Nix and `cargo-shear` are absent on this host, so `make check` cannot reach its second
step at either revision; the clippy and script steps were run separately instead.

**Cold capsules.** PENDING.

## 6. Benefit and cost

- **Duplicate code removed.** The local SHA-256 read loop is gone; the script delegates
  digest and digest-shape validation to buildlib. The `sha256_file` helper is no longer
  defined locally.
- **Shared operations used.** SHA-256 file digest and digest validation; the pinned
  wheel is exercised through both the uv path and the vendored bootstrap.
- **Correctness gain.** A malformed sidecar digest is rejected with an explicit error
  before hashing, and the runtime refuses a buildlib that is not 0.2.0. The no-uv path
  verifies the interpreter archives, uv archive and wheel by size and SHA-256 before use.
- **Cold setup.** PENDING.
- **Warm no-op.** PENDING.
- **Not justified / not done.** The target map, archive extraction, the sidecar format
  and the wheel tags were not moved to buildlib: it models neither gnu/Windows targets
  nor wheel tags, and it has no extraction or sidecar reader. No release manifest,
  archive or `SHA256SUMS` helper was imported; this builder consumes release assets
  rather than producing them.
