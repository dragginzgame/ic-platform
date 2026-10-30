"""Integration checks against tiny local repositories; never build or fetch IC.

Run with ``python3 -m unittest discover -s tests -v``. Each test copies the real
entrypoints into an isolated outer repository and uses a local Git submodule.
"""

from __future__ import annotations

import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest import mock


REPOSITORY = Path(__file__).resolve().parents[1]


class IntegrationScriptsTest(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="ic-profiler-scripts-")
        self.addCleanup(self.temporary.cleanup)
        self.home = Path(self.temporary.name)
        self.root = self.home / "profiler-repository"
        self.remote = self.home / "tiny-upstream"
        self.root.mkdir()
        self.remote.mkdir()
        self.env = os.environ.copy()
        for name in ("GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_OBJECT_DIRECTORY",
                     "GIT_ALTERNATE_OBJECT_DIRECTORIES", "IC_PROFILER_CONSUMER_ROOT"):
            self.env.pop(name, None)
        self.env.update({
            "GIT_CONFIG_GLOBAL": os.devnull,
            "GIT_CONFIG_NOSYSTEM": "1",
            "GIT_AUTHOR_NAME": "Integration Fixture",
            "GIT_AUTHOR_EMAIL": "fixture@example.invalid",
            "GIT_COMMITTER_NAME": "Integration Fixture",
            "GIT_COMMITTER_EMAIL": "fixture@example.invalid",
            "GIT_ALLOW_PROTOCOL": "file",
            "GIT_TERMINAL_PROMPT": "0",
        })
        self.git(self.remote, "init", "--quiet", "--initial-branch=main")
        self.write(self.remote, "src/base.txt", "upstream base\n")
        self.write(self.remote, ".gitignore", "/ignored-build/\n")
        self.write(self.remote, "rust-toolchain.toml", '[toolchain]\nchannel = "stable"\n')
        self.git(self.remote, "add", ".")
        self.git(self.remote, "commit", "--quiet", "-m", "Pristine fixture upstream")
        self.pin = self.git(self.remote, "rev-parse", "HEAD").strip()

        # The second patch only applies after the first one. Neither is a new file.
        self.write(self.remote, "src/base.txt", "first integration patch\n")
        patch_one = self.git(self.remote, "diff", "--binary", "--full-index")
        self.git(self.remote, "add", "src/base.txt")
        self.write(self.remote, "src/base.txt", "second integration patch\n")
        patch_two = self.git(self.remote, "diff", "--binary", "--full-index")
        self.git(self.remote, "reset", "--hard", "--quiet", self.pin)

        self.write(self.remote, "README.md", "Compatible next upstream revision\n")
        self.git(self.remote, "add", ".")
        self.git(self.remote, "commit", "--quiet", "-m", "Compatible candidate")
        self.candidate = self.git(self.remote, "rev-parse", "HEAD").strip()
        self.write(self.remote, "src/base.txt", "Conflicting next upstream content\n")
        self.git(self.remote, "add", ".")
        self.git(self.remote, "commit", "--quiet", "-m", "Conflicting candidate")
        self.conflicting_candidate = self.git(self.remote, "rev-parse", "HEAD").strip()

        shutil.copytree(REPOSITORY / "scripts", self.root / "scripts")
        self.git(self.root, "init", "--quiet", "--initial-branch=main")
        self.git(self.root, "-c", "protocol.file.allow=always", "submodule", "add",
                 "--quiet", str(self.remote), "upstream/ic")
        self.upstream = self.root / "upstream/ic"
        self.git(self.upstream, "checkout", "--quiet", "--detach", self.pin)
        self.write(self.root, ".gitignore", "/.build/\n__pycache__/\n*.py[cod]\n")
        self.write(self.root, "profiler/server.rs", "// profiler server\n")
        self.write(self.root, "profiler/viewer/fixture_client.py", "VALUE = 'fixture'\n")
        self.write(self.root, "profiler/system/.metadata", "hidden owned source\n")
        self.write(self.root, "profiler/bin/inspect", "#!/bin/sh\nexit 0\n")
        (self.root / "profiler/bin/inspect").chmod(0o755)
        self.write(self.root, "overlay/rs/embedders/owned_hook.rs", "// owned integration hook\n")
        self.write(self.root, "docs/profiling-design.md", "Canonical profiler design\n")
        self.write(self.root, "patches/ic/0001-first.patch", patch_one)
        self.write(self.root, "patches/ic/0002-second.patch", patch_two)
        self.write(self.root, "patches/ic/series", "# Apply in this order\n0001-first.patch\n0002-second.patch\n")
        self.manifest_path = self.root / "integration/manifest.json"
        self.manifest_path.parent.mkdir()
        self.write_manifest({
            "schema_version": 1,
            "upstream": {"path": "upstream/ic", "commit": self.pin,
                         "baseline_commit": self.pin, "url": str(self.remote)},
            "profiler": {"path": "profiler", "destination": "rs/pocket_ic_server/profiling"},
            "patches": {"directory": "patches/ic", "series": "patches/ic/series", "files": []},
            "overlay": {"path": "overlay", "files": [
                {"path": "docs/profiling-design.md", "source": "docs/profiling-design.md",
                 "sha256": "0" * 64},
            ]},
        })
        self.script("refresh-manifest")
        self.git(self.root, "add", ".")
        self.git(self.root, "commit", "--quiet", "-m", "Profiler integration fixture")
        self.checkout = self.root / ".build/ic"

    def process(self, args, cwd):
        return subprocess.run([str(arg) for arg in args], cwd=cwd, env=self.env,
                              stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
                              timeout=30, check=False)

    def git(self, directory, *args):
        result = self.process(["git", "-C", directory, *args], directory)
        self.assertEqual(result.returncode, 0, result.stderr)
        return result.stdout

    def script(self, command, *args, success=True):
        result = self.process([self.root / "scripts" / command, *args], self.root)
        if success:
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        else:
            self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        return result

    @staticmethod
    def write(directory, relative, content):
        destination = directory / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(content)
        return destination

    def manifest(self):
        return json.loads(self.manifest_path.read_text())

    def write_manifest(self, value):
        self.manifest_path.write_text(json.dumps(value, indent=2) + "\n")

    def receipt(self, checkout=None):
        return json.loads(((checkout or self.checkout) / ".git/ic-profiler-receipt.json").read_text())

    def assert_pristine_pin(self):
        self.assertEqual(self.git(self.upstream, "rev-parse", "HEAD").strip(), self.pin)
        self.assertEqual(self.git(self.upstream, "status", "--porcelain=v1", "--ignored"), "")
        self.assertEqual(self.git(self.root, "ls-files", "--stage", "--", "upstream/ic").strip(),
                         f"160000 {self.pin} 0\tupstream/ic")

    def pin_fixture_upstream_edit(self):
        """Review/stage an intentional upstream fixture commit in both pin locations."""
        self.git(self.upstream, "add", ".")
        self.git(self.upstream, "commit", "--quiet", "-m", "Additional upstream fixture")
        self.pin = self.git(self.upstream, "rev-parse", "HEAD").strip()
        self.git(self.root, "add", "upstream/ic")
        manifest = self.manifest()
        manifest["upstream"]["commit"] = self.pin
        self.write_manifest(manifest)

    def test_prepare_applies_ordered_patches_and_mapped_owned_sources(self):
        self.script("prepare")
        self.assertEqual((self.checkout / "src/base.txt").read_text(), "second integration patch\n")
        self.assertEqual((self.checkout / "rs/pocket_ic_server/profiling/server.rs").read_bytes(),
                         (self.root / "profiler/server.rs").read_bytes())
        self.assertEqual((self.checkout / "rs/pocket_ic_server/profiling/system/.metadata").read_text(),
                         "hidden owned source\n")
        self.assertTrue(os.access(self.checkout / "rs/pocket_ic_server/profiling/bin/inspect", os.X_OK))
        self.assertEqual((self.checkout / "docs/profiling-design.md").read_bytes(),
                         (self.root / "docs/profiling-design.md").read_bytes())
        self.assertTrue((self.checkout / "rs/embedders/owned_hook.rs").is_file())
        receipt = self.receipt()
        self.assertEqual(receipt["upstream_commit"], self.pin)
        self.assertEqual([entry["path"] for entry in receipt["sources"]["patches"]],
                         ["0001-first.patch", "0002-second.patch"])
        for field in ("manifest_sha256", "recipe_sha256", "checkout_sha256"):
            self.assertEqual(len(receipt[field]), 64)
        self.assertFalse((self.checkout / ".git/objects/info/alternates").exists())
        self.assert_pristine_pin()

    def test_prepare_is_idempotent_after_verifying_content(self):
        self.script("prepare")
        receipt_bytes = (self.checkout / ".git/ic-profiler-receipt.json").read_bytes()
        original_inode = self.checkout.stat().st_ino
        result = self.script("prepare")
        self.assertIn("Verified existing assembly", result.stdout)
        self.assertEqual(self.checkout.stat().st_ino, original_inode)
        self.assertEqual((self.checkout / ".git/ic-profiler-receipt.json").read_bytes(), receipt_bytes)

    def test_consumer_recorder_and_provider_adapters_have_separate_origins(self):
        consumer = self.home / "consumer"
        consumer.mkdir()
        self.git(consumer, "init", "--quiet")
        self.write(consumer, "profiler/system/core.rs", "// IC-free recorder\n")
        self.env["IC_PROFILER_CONSUMER_ROOT"] = str(consumer)
        self.script("prepare")
        checkout = consumer / ".build/ic"
        self.assertEqual((checkout / "rs/pocket_ic_server/profiling/system/core.rs").read_text(),
                         "// IC-free recorder\n")
        state = self.receipt(checkout)["sources"]["profiler"]
        self.assertEqual({entry["origin"] for entry in state}, {"provider", "consumer"})
        latest = consumer / ".build/receipts/latest-prepare.json"
        self.assertEqual(json.loads(latest.read_text()),
                         {**self.receipt(checkout), "destination": str(checkout)})
        self.assertFalse((self.root / ".build/receipts/latest-prepare.json").exists())
        before = self.receipt(checkout)["recipe_sha256"]
        self.write(consumer, "profiler/system/core.rs", "// revised recorder\n")
        self.script("prepare", success=False)
        self.script("prepare", "--force")
        self.assertNotEqual(self.receipt(checkout)["recipe_sha256"], before)
        self.assertEqual(json.loads(latest.read_text())["recipe_sha256"],
                         self.receipt(checkout)["recipe_sha256"])
        self.assert_pristine_pin()

    def test_consumer_prepare_receipt_refreshes_cached_assembly_but_not_candidate(self):
        consumer = self.home / "consumer"
        consumer.mkdir()
        self.git(consumer, "init", "--quiet")
        self.write(consumer, "profiler/system/core.rs", "// consumer recorder\n")
        self.env["IC_PROFILER_CONSUMER_ROOT"] = str(consumer)
        self.script("prepare")
        checkout = consumer / ".build/ic"
        latest = consumer / ".build/receipts/latest-prepare.json"
        expected = {**self.receipt(checkout), "destination": str(checkout)}
        self.assertEqual(json.loads(latest.read_text()), expected)
        latest.write_text('{"stale":true}\n')
        self.script("prepare")
        self.assertEqual(json.loads(latest.read_text()), expected)
        before = latest.read_bytes()
        self.script("update-upstream", self.candidate)
        self.assertEqual(latest.read_bytes(), before)
        self.assertEqual(self.receipt(consumer / ".build/candidates" / self.candidate)["upstream_commit"],
                         self.candidate)

    def test_consumer_cannot_replace_provider_adapters(self):
        consumer = self.home / "consumer"
        self.write(consumer, "profiler/server.rs", "// consumer must not replace an adapter\n")
        self.git(consumer, "init", "--quiet")
        self.env["IC_PROFILER_CONSUMER_ROOT"] = str(consumer)
        result = self.script("prepare", success=False)
        self.assertIn("overlaps Platform provider ownership", result.stderr)
        self.assertFalse((consumer / ".build/ic").exists())

    def test_generated_ignored_python_cache_does_not_change_owned_source_inventory(self):
        self.script("prepare")
        original_receipt = self.receipt()
        result = self.process([sys.executable, "-c", "import py_compile; "
                               "py_compile.compile('profiler/viewer/fixture_client.py', doraise=True)"],
                              self.root)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(list((self.root / "profiler/viewer/__pycache__").glob("*.pyc")))
        result = self.script("prepare")
        self.assertIn("Verified existing assembly", result.stdout)
        self.assertEqual(self.receipt(), original_receipt)
        self.assertFalse((self.checkout / "rs/pocket_ic_server/profiling/viewer/__pycache__").exists())
        # A real source edit must still regenerate, without copying cache artifacts.
        self.write(self.root, "profiler/server.rs", "// revised owned source\n")
        self.script("prepare", "--force")
        self.assertNotEqual(self.receipt()["recipe_sha256"], original_receipt["recipe_sha256"])
        self.assertFalse((self.checkout / "rs/pocket_ic_server/profiling/viewer/__pycache__").exists())

    def test_generated_edits_are_preserved_until_explicit_force(self):
        self.script("prepare")
        for relative in ("src/base.txt", "personal-notes.txt", "ignored-build/notes.txt"):
            with self.subTest(relative=relative):
                edited = self.write(self.checkout, relative, "Do not lose this local edit\n")
                self.script("prepare", success=False)
                self.assertEqual(edited.read_text(), "Do not lose this local edit\n")
                self.script("prepare", "--force")
                if relative == "src/base.txt":
                    self.assertEqual(edited.read_text(), "second integration patch\n")
                else:
                    self.assertFalse(edited.exists())

    def test_generated_edits_hidden_from_git_status_are_detected(self):
        self.script("prepare")
        for flag in ("--assume-unchanged", "--skip-worktree"):
            with self.subTest(flag=flag):
                self.git(self.checkout, "update-index", flag, "src/base.txt")
                edited = self.write(self.checkout, "src/base.txt", "hidden generated edit\n")
                self.script("prepare", success=False)
                self.assertEqual(edited.read_text(), "hidden generated edit\n")
                self.script("prepare", "--force")
                self.assertEqual(edited.read_text(), "second integration patch\n")

    def test_profiler_edit_changes_recipe_and_requires_force(self):
        self.script("prepare")
        before = self.receipt()["recipe_sha256"]
        self.write(self.root, "profiler/server.rs", "// revised profiler source\n")
        self.script("prepare", success=False)
        self.assertEqual((self.checkout / "rs/pocket_ic_server/profiling/server.rs").read_text(),
                         "// profiler server\n")
        self.script("prepare", "--force")
        self.assertNotEqual(self.receipt()["recipe_sha256"], before)
        self.assertEqual((self.checkout / "rs/pocket_ic_server/profiling/server.rs").read_text(),
                         "// revised profiler source\n")

    def test_tracked_untracked_and_ignored_upstream_edits_are_rejected(self):
        for relative in ("src/base.txt", "untracked.txt", "ignored-build/cache"):
            with self.subTest(relative=relative):
                edited = self.write(self.upstream, relative, "upstream must stay pristine\n")
                self.script("prepare", "--force", success=False)
                self.assertEqual(edited.read_text(), "upstream must stay pristine\n")
                if relative == "src/base.txt":
                    self.git(self.upstream, "restore", "src/base.txt")
                else:
                    edited.unlink()
        self.assertFalse(self.checkout.exists())

    def test_upstream_edits_hidden_from_git_status_are_rejected(self):
        for flag, clear in (("--assume-unchanged", "--no-assume-unchanged"),
                            ("--skip-worktree", "--no-skip-worktree")):
            with self.subTest(flag=flag):
                self.git(self.upstream, "update-index", flag, "src/base.txt")
                edited = self.write(self.upstream, "src/base.txt", "hidden upstream edit\n")
                self.assertEqual(self.git(self.upstream, "status", "--porcelain=v1"), "")
                self.script("prepare", "--force", success=False)
                self.assertEqual(edited.read_text(), "hidden upstream edit\n")
                self.git(self.upstream, "update-index", clear, "src/base.txt")
                self.git(self.upstream, "restore", "src/base.txt")
        self.assertFalse(self.checkout.exists())

    def test_upstream_executable_bit_drift_is_rejected_when_git_ignores_filemode(self):
        self.git(self.upstream, "config", "core.filemode", "false")
        source = self.upstream / "src/base.txt"
        source.chmod(0o755)
        self.assertEqual(self.git(self.upstream, "status", "--porcelain=v1"), "")
        self.script("prepare", success=False)
        self.assertTrue(os.access(source, os.X_OK))
        self.assertFalse(self.checkout.exists())

    def test_wrong_upstream_head_is_rejected(self):
        self.git(self.upstream, "checkout", "--quiet", "--detach", self.candidate)
        self.script("prepare", success=False)
        self.assertFalse(self.checkout.exists())

    def test_uninitialized_submodule_does_not_resolve_parent_head(self):
        self.git(self.root, "submodule", "deinit", "--force", "upstream/ic")
        result = self.script("prepare", success=False)
        self.assertIn("Initialize the pinned submodule first", result.stderr)
        self.assertNotIn("expected pinned commit", result.stderr)
        self.assertFalse(self.checkout.exists())

    def fake_cargo(self):
        tools = self.home / "fake-tools"
        tools.mkdir()
        self.write(tools, "cargo", '''#!/bin/sh
if [ "$1" = --version ]; then printf 'cargo fixture\n'; exit 0; fi
printf '%s\n' "$@" > "$IC_PROFILER_FIXTURE_ARGS"
printf '{"reason":"compiler-artifact","executable":"%s"}\n' "$IC_PROFILER_FIXTURE_EXECUTABLE"
exit "${IC_PROFILER_FIXTURE_EXIT:-0}"
''')
        self.write(tools, "rustc", "#!/bin/sh\nprintf 'rustc fixture\\n'\n")
        for tool in (tools / "cargo", tools / "rustc"):
            tool.chmod(0o755)
        artifact = self.write(self.home, "fixture-executable", "immutable executable bytes\n")
        artifact.chmod(0o755)
        self.env.update({"PATH": str(tools) + os.pathsep + self.env["PATH"],
                         "RUSTC": str(tools / "rustc"),
                         "IC_PROFILER_FIXTURE_ARGS": str(self.home / "cargo-arguments"),
                         "IC_PROFILER_FIXTURE_EXECUTABLE": str(artifact)})

    def test_native_package_selection_and_immutable_failure_receipts(self):
        self.fake_cargo()
        self.env.update({"PROTOC": str(self.home / "custom protoc"),
                         "PROTOC_INCLUDE": str(self.home / "proto includes")})
        self.script("test", "--", "--lib")
        first = json.loads((self.root / ".build/receipts/latest-test.json").read_text())
        self.assertIn("ic-system-profiler", first["command"])
        self.assertEqual(first["build_environment"]["PROTOC"], self.env["PROTOC"])
        self.assertEqual(first["build_environment"]["PROTOC_INCLUDE"], self.env["PROTOC_INCLUDE"])
        self.script("test", "--", "-p", "pocket-ic", "--test", "system_profiling")
        second = json.loads((self.root / ".build/receipts/latest-test.json").read_text())
        self.assertNotIn("ic-system-profiler", second["command"])
        self.assertIn("pocket-ic", second["command"])
        self.assertEqual(second["artifacts"][0]["sha256"], hashlib.sha256(b"immutable executable bytes\n").hexdigest())
        self.env["IC_PROFILER_FIXTURE_EXIT"] = "7"
        self.script("build", success=False)
        failure = json.loads((self.root / ".build/receipts/latest-build.json").read_text())
        self.assertEqual(failure["exit_code"], 7)
        immutable = list((self.root / ".build/receipts").glob("test-*.json"))
        self.assertEqual(len(immutable), 2)
        self.assertIn(first, [json.loads(path.read_text()) for path in immutable])
        self.assertEqual(len(list((self.root / ".build/receipts").glob("build-*.json"))), 1)

    def test_native_build_records_the_pinned_routing_snapshot(self):
        self.fake_cargo()
        contents = '[{"fixture": "routing snapshot"}]\n'
        self.write(self.upstream, "mainnet-routing-table.json", contents)
        self.pin_fixture_upstream_edit()
        self.script("build")
        receipt = json.loads((self.root / ".build/receipts/latest-build.json").read_text())
        self.assertEqual(receipt["build_environment"]["MAINNET_ROUTING_TABLE"],
                         str(self.checkout / "mainnet-routing-table.json"))
        self.assertEqual(receipt["embedded_inputs"][0]["sha256"], hashlib.sha256(contents.encode()).hexdigest())

    def test_invalid_routing_override_does_not_reach_cargo(self):
        self.fake_cargo()
        path = self.write(self.home, "invalid-routing.json", "")
        self.env["MAINNET_ROUTING_TABLE"] = str(path)
        self.script("build", success=False)
        self.assertFalse(Path(self.env["IC_PROFILER_FIXTURE_ARGS"]).exists())

    def test_stale_gitlink_is_rejected(self):
        self.git(self.root, "update-index", "--cacheinfo", f"160000,{self.candidate},upstream/ic")
        self.script("prepare", success=False)
        self.assertFalse(self.checkout.exists())

    def test_stale_manifest_pin_is_rejected(self):
        manifest = self.manifest()
        manifest["upstream"]["commit"] = self.candidate
        self.write_manifest(manifest)
        self.script("prepare", success=False)
        self.assertFalse(self.checkout.exists())

    def test_modified_patch_and_overlay_pins_are_rejected_even_with_force(self):
        self.script("prepare")
        for relative in ("patches/ic/0001-first.patch", "overlay/rs/embedders/owned_hook.rs",
                         "docs/profiling-design.md"):
            with self.subTest(relative=relative):
                source = self.root / relative
                original = source.read_bytes()
                source.write_bytes(original + b"\n")
                self.script("refresh-manifest", "--check", success=False)
                self.script("prepare", "--force", success=False)
                self.assertEqual((self.checkout / "src/base.txt").read_text(), "second integration patch\n")
                source.write_bytes(original)

    def test_series_order_is_pinned_and_reordering_fails_without_losing_checkout(self):
        self.script("prepare")
        self.write(self.root, "patches/ic/series", "0002-second.patch\n0001-first.patch\n")
        self.script("prepare", "--force", success=False)
        self.script("refresh-manifest")
        self.script("prepare", "--force", success=False)
        self.assertEqual((self.checkout / "src/base.txt").read_text(), "second integration patch\n")

    def test_refresh_check_is_read_only_and_refresh_reconciles_inventory(self):
        before = self.manifest_path.read_bytes()
        self.script("refresh-manifest", "--check")
        self.assertEqual(self.manifest_path.read_bytes(), before)
        (self.root / "overlay/rs/embedders/owned_hook.rs").unlink()
        added = self.write(self.root, "overlay/new/owned.txt", "new owned source\n")
        self.script("refresh-manifest", "--check", success=False)
        self.assertEqual(self.manifest_path.read_bytes(), before)
        self.script("refresh-manifest")
        entries = {entry["path"]: entry for entry in self.manifest()["overlay"]["files"]}
        self.assertEqual(set(entries), {"new/owned.txt", "docs/profiling-design.md"})
        self.assertEqual(entries["new/owned.txt"]["sha256"], hashlib.sha256(added.read_bytes()).hexdigest())
        self.assertEqual(entries["docs/profiling-design.md"]["source"], "docs/profiling-design.md")
        self.script("refresh-manifest", "--check")
        self.script("prepare")
        self.assertTrue((self.checkout / "new/owned.txt").is_file())

    def test_overlay_cannot_overwrite_upstream_owned_file(self):
        self.script("prepare")
        self.write(self.root, "overlay/src/base.txt", "attempted overwrite\n")
        self.script("refresh-manifest")
        self.script("prepare", "--force", success=False)
        self.assertEqual((self.checkout / "src/base.txt").read_text(), "second integration patch\n")
        self.assert_pristine_pin()

    def test_overlay_cannot_overwrite_profiler_owned_file(self):
        self.write(self.root, "overlay/rs/pocket_ic_server/profiling/server.rs", "collision\n")
        self.script("refresh-manifest")
        self.script("prepare", success=False)
        self.assertFalse(self.checkout.exists())

    def test_overlay_rejects_upstream_symlink_parent_without_writing_outside(self):
        outside = self.home / "outside-overlay"
        sentinel = self.write(outside, "keep.txt", "external file\n")
        (self.upstream / "escape").symlink_to(outside, target_is_directory=True)
        self.pin_fixture_upstream_edit()
        self.write(self.root, "overlay/escape/new.txt", "must never escape checkout\n")
        self.script("refresh-manifest")
        self.script("prepare", success=False)
        self.assertEqual(sentinel.read_text(), "external file\n")
        self.assertFalse((outside / "new.txt").exists())
        self.assertFalse(self.checkout.exists())

    def test_profiler_destination_rejects_upstream_symlink_parent(self):
        outside = self.home / "outside-profiler"
        sentinel = self.write(outside, "keep.txt", "external profiler parent\n")
        (self.upstream / "escape").symlink_to(outside, target_is_directory=True)
        self.pin_fixture_upstream_edit()
        manifest = self.manifest()
        manifest["profiler"]["destination"] = "escape/profiling"
        self.write_manifest(manifest)
        self.script("prepare", success=False)
        self.assertEqual(sentinel.read_text(), "external profiler parent\n")
        self.assertFalse((outside / "profiling").exists())
        self.assertFalse(self.checkout.exists())

    def test_manifest_destinations_reject_path_traversal(self):
        original = self.manifest()
        for destination in ("../escape", "/tmp/escape", "valid/../escape", ".git/config", "a\\escape"):
            with self.subTest(destination=destination):
                manifest = json.loads(json.dumps(original))
                manifest["overlay"]["files"][0]["path"] = destination
                self.write_manifest(manifest)
                self.script("prepare", success=False)
                self.assertFalse(self.checkout.exists())
        manifest = json.loads(json.dumps(original))
        manifest["profiler"]["destination"] = "../escape"
        self.write_manifest(manifest)
        self.script("prepare", success=False)
        self.assertFalse((self.root / ".build/escape").exists())

    def test_patch_series_rejects_path_traversal_and_duplicates(self):
        for series in ("../escape.patch\n", "/tmp/escape.patch\n", "0001-first.patch\n0001-first.patch\n"):
            with self.subTest(series=series):
                self.write(self.root, "patches/ic/series", series)
                self.script("refresh-manifest", success=False)
                self.script("prepare", success=False)
        self.assertFalse(self.checkout.exists())

    def test_patches_cannot_introduce_profiler_owned_files(self):
        self.write(self.root, "patches/ic/new.patch", "diff --git a/new.txt b/new.txt\n"
                   "new file mode 100644\n--- /dev/null\n+++ b/new.txt\n@@ -0,0 +1 @@\n+owned file\n")
        self.write(self.root, "patches/ic/series", "new.patch\n")
        self.script("refresh-manifest")
        self.script("prepare", success=False)
        self.assertFalse(self.checkout.exists())

    def test_sources_reject_symlinks(self):
        outside = self.write(self.home, "outside.txt", "outside source\n")
        for relative in ("profiler/link", "overlay/link"):
            with self.subTest(relative=relative):
                link = self.root / relative
                link.symlink_to(outside)
                self.script("refresh-manifest", success=False)
                self.script("prepare", success=False)
                link.unlink()
        mapped = self.root / "docs/profiling-design.md"
        mapped.unlink()
        mapped.symlink_to(outside)
        self.script("refresh-manifest", success=False)
        self.script("prepare", success=False)
        self.assertEqual(outside.read_text(), "outside source\n")

    def test_force_refuses_unowned_target(self):
        sentinel = self.write(self.checkout, "keep.txt", "unowned directory\n")
        self.script("prepare", "--force", success=False)
        self.assertEqual(sentinel.read_text(), "unowned directory\n")

    def test_force_refuses_git_checkout_owned_by_another_repository(self):
        self.script("prepare")
        owner_path = self.checkout / ".git/ic-profiler-owner.json"
        owner = json.loads(owner_path.read_text())
        owner["root"] = str(self.home / "different-profiler-repository")
        owner_path.write_text(json.dumps(owner))
        sentinel = self.write(self.checkout, "keep.txt", "foreign-owned checkout\n")
        self.script("prepare", "--force", success=False)
        self.assertEqual(sentinel.read_text(), "foreign-owned checkout\n")
        self.assertEqual(json.loads(owner_path.read_text()), owner)

    def test_force_repairs_owned_checkout_with_missing_or_invalid_receipt(self):
        self.script("prepare")
        for contents in (None, "not valid JSON\n"):
            with self.subTest(contents=contents):
                receipt = self.checkout / ".git/ic-profiler-receipt.json"
                if contents is None:
                    receipt.unlink()
                else:
                    receipt.write_text(contents)
                note = self.write(self.checkout, "local-note.txt", "preserve until explicit force\n")
                self.script("prepare", success=False)
                self.assertEqual(note.read_text(), "preserve until explicit force\n")
                self.script("prepare", "--force")
                self.assertFalse(note.exists())
                self.assertEqual(self.receipt()["upstream_commit"], self.pin)

    def test_force_refuses_target_symlink(self):
        outside = self.home / "outside-target"
        sentinel = self.write(outside, "keep.txt", "unowned symlink target\n")
        self.checkout.parent.mkdir()
        self.checkout.symlink_to(outside, target_is_directory=True)
        self.script("prepare", "--force", success=False)
        self.assertTrue(self.checkout.is_symlink())
        self.assertEqual(sentinel.read_text(), "unowned symlink target\n")

    def test_force_refuses_symlink_build_directory(self):
        outside = self.home / "outside-build"
        sentinel = self.write(outside, "keep.txt", "outside build directory\n")
        (self.root / ".build").symlink_to(outside, target_is_directory=True)
        self.script("prepare", "--force", success=False)
        self.assertEqual(sentinel.read_text(), "outside build directory\n")
        self.assertFalse((outside / "ic").exists())

    def test_force_refuses_symlink_git_directory(self):
        self.script("prepare")
        git_directory = self.checkout / ".git"
        saved = self.home / "saved-generated-git"
        git_directory.rename(saved)
        git_directory.symlink_to(saved, target_is_directory=True)
        self.script("prepare", "--force", success=False)
        self.assertTrue(git_directory.is_symlink())
        self.assertTrue((saved / "ic-profiler-owner.json").is_file())

    def test_failed_forced_rebuild_preserves_previous_generated_edits(self):
        self.script("prepare")
        edited = self.write(self.checkout, "src/base.txt", "preserve edited generated code\n")
        note = self.write(self.checkout, "local-note.txt", "preserve my note\n")
        self.write(self.root, "patches/ic/0002-second.patch", "not a valid Git patch\n")
        self.script("refresh-manifest")
        self.script("prepare", "--force", success=False)
        self.assertEqual(edited.read_text(), "preserve edited generated code\n")
        self.assertEqual(note.read_text(), "preserve my note\n")
        self.assertFalse(list(self.checkout.parent.glob(".ic.prepare-*")))

    def test_update_upstream_prepares_isolated_candidate_without_moving_pin(self):
        self.script("prepare")
        note = self.write(self.checkout, "local-note.txt", "main assembly edit survives candidate probe\n")
        before_manifest = self.manifest_path.read_bytes()
        before_index = self.git(self.root, "ls-files", "--stage")
        # A candidate already in the local object database needs no origin access.
        self.git(self.upstream, "remote", "set-url", "origin", "ssh://invalid.example.invalid/ic.git")
        self.script("update-upstream", self.candidate)
        candidate = self.root / ".build/candidates" / self.candidate
        self.assertEqual((candidate / "src/base.txt").read_text(), "second integration patch\n")
        self.assertEqual((candidate / "README.md").read_text(), "Compatible next upstream revision\n")
        self.assertEqual(self.receipt(candidate)["upstream_commit"], self.candidate)
        report = json.loads((self.root / ".build/receipts" / f"upstream-{self.candidate}.json").read_text())
        self.assertFalse(report["pin_changed"])
        self.assertEqual(self.manifest_path.read_bytes(), before_manifest)
        self.assertEqual(self.git(self.root, "ls-files", "--stage"), before_index)
        self.assertEqual(note.read_text(), "main assembly edit survives candidate probe\n")
        self.assert_pristine_pin()

    def test_update_upstream_conflict_preserves_pin_and_existing_assembly(self):
        self.script("prepare")
        before = self.manifest_path.read_bytes()
        self.script("update-upstream", self.conflicting_candidate, success=False)
        self.assertEqual(self.manifest_path.read_bytes(), before)
        self.assertFalse((self.root / ".build/candidates" / self.conflicting_candidate).exists())
        self.assertEqual((self.checkout / "src/base.txt").read_text(), "second integration patch\n")
        self.assert_pristine_pin()

    def test_update_force_refuses_unowned_candidate_directory(self):
        candidate = self.root / ".build/candidates" / self.candidate
        sentinel = self.write(candidate, "keep.txt", "unowned candidate\n")
        self.script("update-upstream", self.candidate, "--force", success=False)
        self.assertEqual(sentinel.read_text(), "unowned candidate\n")
        self.assert_pristine_pin()


class NativeVerifyGateTest(unittest.TestCase):
    def test_product_regression_failures_block_installed_scenarios_and_restore_environment(self):
        spec = importlib.util.spec_from_file_location("profiler_gate_fixture", REPOSITORY / "scripts/ic_profiler.py")
        provider = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(provider)
        root = Path("/fixture/provider")
        manifest = {"fixture": True}
        for refused in [False, True]:
            calls = []
            def cargo(_root, _manifest, command, args, force):
                calls.append((command, args))
                if "profiling::" in args and refused:
                    raise provider.Error("product regression failed")
            with mock.patch.dict(os.environ, {"CARGO_TARGET_DIR": "prior-target", "POCKET_IC_BIN": "prior-server", "SYSTEM_PROFILE_TEST_SERVER_URL": "prior-url"}), mock.patch.object(provider, "run"), mock.patch.object(provider, "check_native_graph"), mock.patch.object(provider, "build_directory", return_value=Path("/fixture/consumer/.build")), mock.patch.object(provider, "cargo_command", side_effect=cargo):
                if refused:
                    with self.assertRaises(provider.Error):
                        provider.native_verify(root, manifest)
                else:
                    provider.native_verify(root, manifest)
                product = [args for command, args in calls if "profiling::" in args]
                installed = [args for command, args in calls if "system_profiling" in args]
                self.assertEqual(product, [["--", "-p", "pocket-ic-server", "--lib", "profiling::"]])
                self.assertEqual(calls[0], ("clippy", ["--", "--no-deps", "--lib", "--", "-D", "warnings"]))
                self.assertEqual(bool(installed), not refused)
                self.assertEqual(os.environ["CARGO_TARGET_DIR"], "prior-target")
                self.assertEqual(os.environ["POCKET_IC_BIN"], "prior-server")
                self.assertEqual(os.environ["SYSTEM_PROFILE_TEST_SERVER_URL"], "prior-url")

    def test_native_lint_failure_stops_before_build_and_installation(self):
        spec = importlib.util.spec_from_file_location("profiler_lint_gate_fixture", REPOSITORY / "scripts/ic_profiler.py")
        provider = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(provider)
        calls = []
        def cargo(_root, _manifest, command, args, force):
            calls.append(command)
            raise provider.Error("native lint failed")
        with mock.patch.object(provider, "run"), mock.patch.object(provider, "check_native_graph"), mock.patch.object(provider, "build_directory", return_value=Path("/fixture/.build")), mock.patch.object(provider, "cargo_command", side_effect=cargo):
            with self.assertRaises(provider.Error):
                provider.native_verify(Path("/fixture"), {})
        self.assertEqual(calls, ["clippy"])


class NativeToolsTest(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="ic-profiler-native-tools-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.bin = self.root / "bin"
        self.bin.mkdir()
        self.log = self.root / "installation.log"
        self.installed = self.root / "installed"
        self.env = {**os.environ, "PATH": str(self.bin),
                    "NATIVE_TEST_LOG": str(self.log), "NATIVE_TEST_INSTALLED": str(self.installed)}
        self.env.pop("PROTOC", None)
        self.tool("protoc", "printf 'libprotoc fixture\\n'\n")
        self.tool("dpkg-query", '''for package do :; done
if [ "${NATIVE_TEST_QUERY_STATUS:-0}" != 0 ]; then exit "$NATIVE_TEST_QUERY_STATUS"; fi
if [ "$package" = protobuf-compiler ] && [ ! -f "$NATIVE_TEST_INSTALLED" ]; then exit 1; fi
printf installed
''')
        self.tool("id", "printf '1000\\n'\n")
        self.tool("sudo", 'printf "sudo\\n" >> "$NATIVE_TEST_LOG"\nexec "$@"\n')
        self.tool("apt-get", '''printf '%s\\n' "$@" >> "$NATIVE_TEST_LOG"
if [ "${NATIVE_TEST_APT_STATUS:-0}" != 0 ]; then exit "$NATIVE_TEST_APT_STATUS"; fi
: > "$NATIVE_TEST_INSTALLED"
''')

    def tool(self, name, body):
        path = self.bin / name
        path.write_text("#!/bin/sh\nset -eu\n" + body)
        path.chmod(0o755)
        return path

    def run_tools(self, operation):
        return subprocess.run(["/bin/bash", REPOSITORY / "scripts/dev/native-tools.sh", operation],
                              env=self.env, capture_output=True, text=True, timeout=10, check=False)

    def test_check_is_read_only_and_honors_explicit_selection(self):
        self.assertEqual(self.run_tools("check").returncode, 0)
        self.assertFalse(self.log.exists())
        self.env["PROTOC"] = str(self.root / "missing compiler")
        result = self.run_tools("check")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("make install-native-tools", result.stderr)
        self.assertFalse(self.log.exists())
        self.env["PROTOC"] = str(self.tool("protoc with spaces", "printf 'libprotoc explicit\\n'\n"))
        self.assertEqual(self.run_tools("check").returncode, 0)
        self.env.pop("PROTOC")
        (self.bin / "protoc").unlink()
        self.assertNotEqual(self.run_tools("check").returncode, 0)
        self.assertFalse(self.log.exists())

    def test_install_prepares_only_missing_packages_and_reuses_them(self):
        result = self.run_tools("install")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.log.read_text().splitlines(),
                         ["sudo", "install", "--yes", "--no-install-recommends", "protobuf-compiler"])
        before = self.log.read_bytes()
        self.assertEqual(self.run_tools("install").returncode, 0)
        self.assertEqual(self.run_tools("check").returncode, 0)
        self.assertEqual(self.log.read_bytes(), before)

    def test_install_failure_is_preserved(self):
        self.env["NATIVE_TEST_APT_STATUS"] = "19"
        self.assertEqual(self.run_tools("install").returncode, 19)
        self.assertFalse(self.installed.exists())

    def test_failed_package_inspection_does_not_start_installation(self):
        self.env["NATIVE_TEST_QUERY_STATUS"] = "2"
        self.assertEqual(self.run_tools("install").returncode, 2)
        self.assertFalse(self.log.exists())


if __name__ == "__main__":
    unittest.main()
