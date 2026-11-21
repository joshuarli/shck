from __future__ import annotations

import hashlib
import importlib.util
import os
import subprocess
import sys
import tarfile
import tempfile
import unittest
import zipfile
from importlib.metadata import version
from pathlib import Path

import buildlib
from buildlib import targets

SCRIPT_PATH = Path(__file__).resolve().parent.parent / "scripts" / "build-python-release.py"
SPEC = importlib.util.spec_from_file_location("build_python_release", SCRIPT_PATH)
assert SPEC is not None
assert SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)

HOST_TRIPLE = "aarch64-apple-darwin"
EXPECTED_TRIPLES = {
    "aarch64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "aarch64-unknown-linux-musl",
    "x86_64-pc-windows-msvc",
    "x86_64-unknown-linux-gnu",
    "x86_64-unknown-linux-musl",
}


class ReleaseTargetTests(unittest.TestCase):
    def test_target_map_matches_the_published_asset_names(self) -> None:
        self.assertEqual(set(MODULE.TARGETS), EXPECTED_TRIPLES)
        for triple, spec in MODULE.TARGETS.items():
            self.assertEqual(spec.target, triple)
            suffix = ".zip" if spec.packaged_binary_name == "shuck.exe" else ".tar.xz"
            self.assertEqual(spec.archive_name, f"shuck-cli-{triple}{suffix}")

    def test_target_map_covers_every_buildlib_triple(self) -> None:
        self.assertLessEqual(set(targets.TRIPLES), set(MODULE.TARGETS))

    def test_pinned_buildlib_version_is_imported(self) -> None:
        self.assertEqual(version("laputa-buildlib"), MODULE.BUILDLIB_VERSION)
        self.assertTrue(buildlib.__file__)


class ArchiveChecksumTests(unittest.TestCase):
    def setUp(self) -> None:
        self._temp = tempfile.TemporaryDirectory(prefix="shuck-release-test-")
        self.addCleanup(self._temp.cleanup)
        self.root = Path(self._temp.name)
        self.archive = self.root / "shuck-cli-aarch64-apple-darwin.tar.xz"
        self.archive.write_bytes(b"release bytes")
        self.checksum = self.archive.with_name(self.archive.name + ".sha256")

    def _write(self, text: str) -> None:
        self.checksum.write_text(text, encoding="utf-8")

    def test_accepts_the_published_sidecar(self) -> None:
        digest = hashlib.sha256(b"release bytes").hexdigest()
        self._write(f"{digest}  {self.archive.name}\n")
        self.assertEqual(MODULE.verify_archive_checksum(self.archive), digest)

    def test_rejects_a_wrong_digest(self) -> None:
        self._write(f"{'0' * 64}  {self.archive.name}\n")
        with self.assertRaises(SystemExit) as raised:
            MODULE.verify_archive_checksum(self.archive)
        self.assertIn("checksum mismatch", str(raised.exception))

    def test_rejects_a_malformed_digest(self) -> None:
        self._write(f"not-a-digest  {self.archive.name}\n")
        with self.assertRaises(SystemExit) as raised:
            MODULE.verify_archive_checksum(self.archive)
        self.assertIn("invalid sha256", str(raised.exception))

    def test_rejects_a_sidecar_naming_another_asset(self) -> None:
        digest = hashlib.sha256(b"release bytes").hexdigest()
        self._write(f"{digest}  other.tar.xz\n")
        with self.assertRaises(SystemExit) as raised:
            MODULE.verify_archive_checksum(self.archive)
        self.assertIn("lists other.tar.xz", str(raised.exception))

    def test_rejects_a_truncated_archive(self) -> None:
        digest = hashlib.sha256(b"release bytes").hexdigest()
        self._write(f"{digest}  {self.archive.name}\n")
        self.archive.write_bytes(b"release")
        with self.assertRaises(SystemExit) as raised:
            MODULE.verify_archive_checksum(self.archive)
        self.assertIn("checksum mismatch", str(raised.exception))

    def test_rejects_a_missing_sidecar(self) -> None:
        with self.assertRaises(SystemExit) as raised:
            MODULE.verify_archive_checksum(self.archive)
        self.assertIn("missing checksum file", str(raised.exception))

    def test_rejects_an_empty_sidecar(self) -> None:
        self._write("")
        with self.assertRaises(SystemExit) as raised:
            MODULE.verify_archive_checksum(self.archive)
        self.assertIn("empty checksum file", str(raised.exception))


class ArchiveExtractionTests(unittest.TestCase):
    def setUp(self) -> None:
        self._temp = tempfile.TemporaryDirectory(prefix="shuck-release-test-")
        self.addCleanup(self._temp.cleanup)
        self.root = Path(self._temp.name)

    def _tar(self, members: dict[str, bytes]) -> Path:
        archive = self.root / MODULE.TARGETS[HOST_TRIPLE].archive_name
        staging = self.root / "inputs"
        staging.mkdir()
        with tarfile.open(archive, mode="w:xz") as tar:
            for index, (name, payload) in enumerate(members.items()):
                source = staging / f"member-{index}"
                source.write_bytes(payload)
                tar.add(source, arcname=name)
        return archive

    def _zip(self, members: dict[str, bytes]) -> Path:
        spec = MODULE.TARGETS["x86_64-pc-windows-msvc"]
        archive = self.root / spec.archive_name
        with zipfile.ZipFile(archive, mode="w") as bundle:
            for name, payload in members.items():
                bundle.writestr(name, payload)
        return archive

    def _extract(self, archive: Path, triple: str = HOST_TRIPLE) -> Path:
        staging = self.root / "extracted"
        return MODULE.extract_archive(archive, MODULE.TARGETS[triple], staging)

    def test_finds_the_binary_in_a_tar_xz(self) -> None:
        archive = self._tar({"nested/shuck": b"binary"})
        found = self._extract(archive)
        self.assertEqual(found.read_bytes(), b"binary")

    def test_finds_the_binary_in_a_windows_zip(self) -> None:
        archive = self._zip({"shuck.exe": b"binary"})
        found = self._extract(archive, "x86_64-pc-windows-msvc")
        self.assertEqual(found.name, "shuck.exe")

    def test_rejects_an_archive_without_the_binary(self) -> None:
        archive = self._tar({"README": b"text"})
        with self.assertRaises(SystemExit) as raised:
            self._extract(archive)
        self.assertIn("expected exactly one shuck", str(raised.exception))

    def test_rejects_an_archive_with_two_binaries(self) -> None:
        archive = self._tar({"a/shuck": b"one", "b/shuck": b"two"})
        with self.assertRaises(SystemExit) as raised:
            self._extract(archive)
        self.assertIn("found 2", str(raised.exception))


class CommandTests(unittest.TestCase):
    def setUp(self) -> None:
        self._temp = tempfile.TemporaryDirectory(prefix="shuck-release-test-")
        self.addCleanup(self._temp.cleanup)
        self.root = Path(self._temp.name)

    def _run(self, *arguments: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, os.fspath(SCRIPT_PATH), *arguments],
            capture_output=True,
            text=True,
            check=False,
        )

    def _release_archive(self) -> Path:
        staging = self.root / "stage"
        staging.mkdir()
        (staging / "shuck").write_bytes(b"binary")
        (staging / "shuck").chmod(0o755)
        spec = MODULE.TARGETS[HOST_TRIPLE]
        archive = self.root / spec.archive_name
        with tarfile.open(archive, mode="w:xz") as tar:
            tar.add(staging / "shuck", arcname="shuck")
        digest = hashlib.sha256(archive.read_bytes()).hexdigest()
        archive.with_name(archive.name + ".sha256").write_text(
            f"{digest}  {archive.name}\n", encoding="utf-8"
        )
        return archive

    def test_verify_reports_the_checksum_and_binary(self) -> None:
        archive = self._release_archive()
        completed = self._run("verify", "--target", HOST_TRIPLE, "--archive", os.fspath(archive))
        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertIn("verified shuck-cli-aarch64-apple-darwin.tar.xz", completed.stdout)
        self.assertIn("shuck 6 bytes", completed.stdout)

    def test_verify_rejects_an_unsupported_target(self) -> None:
        archive = self._release_archive()
        completed = self._run(
            "verify", "--target", "riscv64-unknown-linux-musl", "--archive", os.fspath(archive)
        )
        self.assertEqual(completed.returncode, 2)
        self.assertIn("invalid choice", completed.stderr)

    def test_provenance_prints_the_pinned_wheel(self) -> None:
        completed = self._run("provenance")
        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertIn("laputa-buildlib 0.2.0 at ", completed.stdout)
        self.assertIn(buildlib.__file__, completed.stdout)

    def test_release_wheels_requires_every_asset(self) -> None:
        completed = self._run(
            "build-release-wheels",
            "--assets-dir",
            os.fspath(self.root),
            "--out-dir",
            os.fspath(self.root / "dist"),
        )
        self.assertEqual(completed.returncode, 1)
        self.assertIn("missing expected release assets", completed.stderr)


if __name__ == "__main__":
    unittest.main()
