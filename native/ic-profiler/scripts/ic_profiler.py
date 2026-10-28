#!/usr/bin/env python3
"""Assemble and run the pinned IC integration using only Python's standard library."""

from __future__ import annotations

import argparse
import contextlib
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import shutil
import stat
import subprocess
import sys
import tempfile
from datetime import datetime, timezone


SCHEMA = 1
MANIFEST = "integration/manifest.json"
OWNER = "ic-profiler-owner.json"
RECEIPT = "ic-profiler-receipt.json"
BUILD_ENVIRONMENT = (
    "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC", "RUSTDOC", "CARGO_BUILD_JOBS",
    "CARGO_PROFILE_DEV_DEBUG", "CARGO_PROFILE_DEV_INCREMENTAL", "CARGO_PROFILE_DEV_OPT_LEVEL",
    "CARGO_PROFILE_RELEASE_DEBUG", "CARGO_PROFILE_RELEASE_INCREMENTAL", "CARGO_PROFILE_RELEASE_OPT_LEVEL",
    "CARGO_PROFILE_RELEASE_LTO",
    "PROTOC", "PROTOC_INCLUDE", "MAINNET_ROUTING_TABLE",
)


class Error(Exception):
    pass


def run(args, *, cwd=None, capture=True, env=None):
    result = subprocess.run(
        [str(arg) for arg in args], cwd=cwd, env=env, check=False,
        stdout=subprocess.PIPE if capture else None,
        stderr=subprocess.PIPE if capture else None,
    )
    if result.returncode:
        detail = result.stderr.decode(errors="replace").strip() if capture else ""
        raise Error(f"Command failed ({result.returncode}): {' '.join(map(str, args))}"
                    + (f"\n{detail}" if detail else ""))
    return result.stdout if capture else b""


def git(path, *args):
    return run(["git", "-C", path, *args])


def digest_bytes(data):
    return hashlib.sha256(data).hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def read_json(path):
    try:
        return json.loads(path.read_text())
    except (OSError, ValueError) as exc:
        raise Error(f"Cannot read {path}: {exc}") from exc


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    try:
        with os.fdopen(descriptor, "w") as stream:
            json.dump(value, stream, indent=2, sort_keys=True)
            stream.write("\n")
        os.replace(temporary, path)
    finally:
        if os.path.exists(temporary):
            os.unlink(temporary)


def relative(value, label):
    if not isinstance(value, str) or not value or "\\" in value or "\0" in value or "\n" in value:
        raise Error(f"Invalid {label}: {value!r}")
    path = PurePosixPath(value)
    if path.is_absolute() or any(part in ("", ".", "..", ".git") for part in value.split("/")):
        raise Error(f"Unsafe {label}: {value!r}")
    return path


def inside(root, value, label):
    rel = relative(value, label)
    path = root.joinpath(*rel.parts)
    current = root
    for part in rel.parts:
        current /= part
        if current.is_symlink():
            raise Error(f"Symlinks are not allowed in {label}: {current}")
    return path


def file_snapshot(path, name):
    try:
        info = path.lstat()
    except OSError as exc:
        raise Error(f"Cannot read {name}: {exc}") from exc
    if not stat.S_ISREG(info.st_mode):
        raise Error(f"Expected a regular file: {name}")
    return {"path": name, "sha256": digest_bytes(path.read_bytes()),
            "executable": bool(info.st_mode & 0o111)}


def tree_files(root, label):
    if not root.is_dir() or root.is_symlink():
        raise Error(f"Expected a real directory for {label}: {root}")
    result = []
    # Respect the owning repository's ignores so local tests/builds can create
    # caches without turning them into profiler source. Tracked files remain
    # included even if an ignore rule later matches them.
    names = set(git(root, "ls-files", "--cached", "--others", "--exclude-standard", "-z", "--", ".").split(b"\0"))
    for raw_name in sorted(names):
        if not raw_name:
            continue
        name = os.fsdecode(raw_name)
        path = inside(root, name, label)
        if not path.exists():
            # The index also lists tracked files deleted in the working tree.
            # Deletions are owned edits; refresh reconciles their inventory.
            continue
        result.append(file_snapshot(path, name))
    return result


def load_manifest(root):
    manifest = read_json(root / MANIFEST)
    if manifest.get("schema_version") != SCHEMA:
        raise Error(f"Unsupported {MANIFEST} schema_version")
    try:
        upstream = manifest["upstream"]
        for field in ("commit", "baseline_commit"):
            value = upstream[field]
            if not isinstance(value, str) or len(value) != 40 or any(c not in "0123456789abcdef" for c in value):
                raise Error(f"upstream.{field} must be a full lowercase Git commit SHA")
        if not isinstance(upstream["url"], str) or not upstream["url"]:
            raise Error("upstream.url must be present")
        inside(root, upstream["path"], "upstream.path")
        inside(root, manifest["profiler"]["path"], "profiler.path")
        relative(manifest["profiler"]["destination"], "profiler.destination")
        inside(root, manifest["patches"]["directory"], "patches.directory")
        inside(root, manifest["patches"]["series"], "patches.series")
        inside(root, manifest["overlay"]["path"], "overlay.path")
    except (KeyError, TypeError) as exc:
        raise Error(f"Malformed {MANIFEST}: missing or invalid {exc}") from exc
    return manifest


def patch_series(root, manifest):
    path = inside(root, manifest["patches"]["series"], "patches.series")
    try:
        lines = path.read_text().splitlines()
    except OSError as exc:
        raise Error(f"Cannot read patch series: {exc}") from exc
    names = []
    for line in lines:
        name = line.strip()
        if not name or name.startswith("#"):
            continue
        relative(name, "patch series entry")
        if name in names:
            raise Error(f"Duplicate patch series entry: {name}")
        names.append(name)
    return names


def consumer_directory(root):
    configured = os.environ.get("IC_PROFILER_CONSUMER_ROOT")
    if configured is None:
        return root
    path = Path(configured)
    if not path.is_absolute() or path.is_symlink() or not path.is_dir():
        raise Error("IC_PROFILER_CONSUMER_ROOT must name a physical absolute consumer directory")
    return path.resolve()


def source_state(root, manifest, *, verify=True):
    """Hash owned sources, checking the reviewed overlay and patch inventory."""
    profiler_root = inside(root, manifest["profiler"]["path"], "profiler.path")
    profiler = tree_files(profiler_root, "profiler")
    consumer = consumer_directory(root)
    if consumer != root:
        provider_files = {entry["path"] for entry in profiler}
        consumer_files = tree_files(inside(consumer, "profiler", "consumer profiler"), "consumer profiler")
        overlap = provider_files.intersection(entry["path"] for entry in consumer_files)
        if overlap:
            raise Error(f"Consumer source overlaps Platform provider ownership: {sorted(overlap)}")
        profiler = [{**entry, "origin": "provider"} for entry in profiler]
        profiler += [{**entry, "origin": "consumer"} for entry in consumer_files]
        profiler.sort(key=lambda entry: entry["path"])
    if not profiler:
        raise Error("The profiler source tree is empty")
    patch_root = inside(root, manifest["patches"]["directory"], "patches.directory")
    names = patch_series(root, manifest)
    patches = [file_snapshot(inside(patch_root, name, "patch path"), name) for name in names]
    declared_patches = manifest["patches"].get("files")
    if verify:
        expected = [{"path": item["path"], "sha256": item["sha256"]} for item in patches]
        if declared_patches != expected:
            raise Error("Patch series/integrity pins changed. Review the edits and run scripts/refresh-manifest.")
    overlay_root = inside(root, manifest["overlay"]["path"], "overlay.path")
    overlay_discovered = tree_files(overlay_root, "overlay")
    declarations = manifest["overlay"].get("files", [])
    if not isinstance(declarations, list):
        raise Error("overlay.files must be an array")
    overlay = []
    destinations = set()
    declared_local = set()
    for entry in declarations:
        try:
            destination = str(relative(entry["path"], "overlay destination"))
            mapped = entry.get("source")
            if not verify and not mapped:
                # A refresh may intentionally add or remove ordinary overlay files.
                continue
            if mapped:
                source = inside(root, mapped, "overlay source")
                if PurePosixPath(mapped).parts[0] in (".build", "upstream", ".git"):
                    raise Error(f"Overlay source must be an owned source file: {mapped}")
            else:
                source = inside(overlay_root, destination, "overlay path")
                declared_local.add(destination)
            snapshot = file_snapshot(source, destination)
            if mapped:
                snapshot["source"] = mapped
            if verify and snapshot["sha256"] != entry["sha256"]:
                raise Error(f"Overlay integrity pin changed for {destination}. Review the edit and run scripts/refresh-manifest.")
        except (KeyError, TypeError) as exc:
            raise Error(f"Malformed overlay entry: {entry!r}") from exc
        if destination in destinations:
            raise Error(f"Duplicate overlay destination: {destination}")
        destinations.add(destination)
        overlay.append(snapshot)
    discovered = {item["path"] for item in overlay_discovered}
    if verify and declared_local != discovered:
        raise Error("Overlay file inventory changed. Review the edits and run scripts/refresh-manifest.")
    if not verify:
        # Mapped entries stay mapped; ordinary overlay files are discovered afresh.
        overlay = [item for item in overlay if "source" in item]
        mapped_destinations = {item["path"] for item in overlay}
        for item in overlay_discovered:
            if item["path"] in mapped_destinations:
                raise Error(f"Overlay destination has both mapped and ordinary sources: {item['path']}")
            overlay.append(item)
    overlay.sort(key=lambda item: item["path"])
    return {"profiler": profiler, "patches": patches, "overlay": overlay}


def validate_upstream(root, manifest):
    upstream = inside(root, manifest["upstream"]["path"], "upstream.path")
    pin = manifest["upstream"]["commit"]
    entry = git(root, "ls-files", "--stage", "--", manifest["upstream"]["path"]).decode().strip()
    expected = f"160000 {pin} 0\t{manifest['upstream']['path']}"
    if entry != expected:
        raise Error("The upstream gitlink does not match integration/manifest.json. Review and stage the pin together.")
    # Git searches parent directories: an empty, uninitialized submodule can
    # otherwise resolve this repository's HEAD and report a false pin mismatch.
    if not (upstream / ".git").exists():
        raise Error("Initialize the pinned submodule first: git submodule update --init upstream/ic")
    try:
        top = Path(git(upstream, "rev-parse", "--show-toplevel").decode().strip()).resolve()
        if top != upstream.resolve():
            raise Error("The upstream path is not an independent Git checkout")
        actual = git(upstream, "rev-parse", "HEAD").decode().strip()
    except Error as exc:
        raise Error("Initialize the pinned submodule first: git submodule update --init upstream/ic") from exc
    if actual != pin:
        raise Error(f"Upstream checkout is {actual}; expected pinned commit {pin}")
    status = git(upstream, "status", "--porcelain=v1", "--untracked-files=all", "--ignored").decode()
    if status:
        raise Error(f"The upstream submodule must be pristine (including ignored files):\n{status[:4000]}")
    # Status can omit files marked assume-unchanged or skip-worktree. Verify the
    # actual checkout against Git blobs without depending on the index's cache.
    for record in git(upstream, "ls-tree", "-r", "-z", pin).split(b"\0"):
        if not record:
            continue
        metadata, raw_name = record.split(b"\t", 1)
        mode, kind, expected_sha = metadata.split(b" ")
        name = os.fsdecode(raw_name)
        rel = relative(name, "upstream file")
        parent = upstream
        for part in rel.parts[:-1]:
            parent /= part
            if parent.is_symlink():
                raise Error(f"Upstream directory is a symlink: {name}")
        path = upstream.joinpath(*rel.parts)
        info = path.lstat()
        if kind != b"blob":
            raise Error(f"Nested upstream submodules are not supported: {name}")
        if mode == b"120000":
            if not stat.S_ISLNK(info.st_mode):
                raise Error(f"Upstream symlink changed: {name}")
            data = os.fsencode(os.readlink(path))
        else:
            if not stat.S_ISREG(info.st_mode) or bool(info.st_mode & 0o111) != (mode == b"100755"):
                raise Error(f"Upstream file type or executable mode changed: {name}")
            data = path.read_bytes()
        actual_sha = hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest().encode()
        if actual_sha != expected_sha:
            raise Error(f"Upstream file bytes changed from the pinned commit: {name}")
    return upstream


def build_directory(root):
    path = consumer_directory(root) / ".build"
    if path.is_symlink() or (path.exists() and not path.is_dir()):
        raise Error(f"Refusing unsafe build directory: {path}")
    path.mkdir(exist_ok=True)
    return path


@contextlib.contextmanager
def assembly_lock(root):
    path = build_directory(root) / ".assembly.lock"
    try:
        descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    except FileExistsError as exc:
        raise Error(f"Another assembly/build is active. If its process has exited, remove stale lock {path}") from exc
    try:
        os.write(descriptor, f"{os.getpid()}\n".encode())
        os.close(descriptor)
        yield
    finally:
        path.unlink(missing_ok=True)


def verify_owned(root, path):
    if path.is_symlink() or not path.is_dir() or (path / ".git").is_symlink() or not (path / ".git").is_dir():
        raise Error(f"Refusing to replace a checkout without a real owned Git directory: {path}")
    owner = read_json(path / ".git" / OWNER)
    if owner != {"schema_version": SCHEMA, "root": str(root), "destination": str(path)}:
        raise Error(f"Refusing to replace a checkout not owned by this repository: {path}")


def checkout_digest(path):
    """Cover tracked changes, the index, and every untracked/ignored file."""
    hasher = hashlib.sha256()
    for value in (git(path, "rev-parse", "HEAD"), git(path, "ls-files", "--stage", "-z")):
        hasher.update(len(value).to_bytes(8, "big"))
        hasher.update(value)
    # Read actual bytes even when Git's index flags/stat cache would hide edits.
    names = sorted(set(git(path, "ls-files", "--cached", "--others", "-z").split(b"\0")))
    for name in names:
        if not name:
            continue
        rel = os.fsdecode(name)
        relative(rel, "generated file")
        file = path / rel
        info = file.lstat()
        if stat.S_ISLNK(info.st_mode):
            value = {"path": rel, "symlink": os.readlink(file)}
        else:
            value = file_snapshot(file, rel)
        hasher.update(canonical(value))
        hasher.update(b"\0")
    return hasher.hexdigest()


def patch_paths(checkout, patch):
    records = iter(git(checkout, "apply", "--numstat", "-z", "--", str(patch)).split(b"\0"))
    result = []
    for record in records:
        if not record:
            continue
        fields = record.split(b"\t", 2)
        if len(fields) != 3:
            raise Error(f"Cannot inspect patch paths: {patch}")
        if fields[2]:
            result.append(os.fsdecode(fields[2]))
        else:
            try:
                result.extend((os.fsdecode(next(records)), os.fsdecode(next(records))))
            except StopIteration as exc:
                raise Error(f"Cannot inspect renamed patch paths: {patch}") from exc
    if not result:
        raise Error(f"Empty patch is not allowed: {patch}")
    return [str(relative(name, "patched path")) for name in result]


def copy_file(source, destination):
    if destination.exists() or destination.is_symlink():
        raise Error(f"Owned sources would overwrite an existing IC path: {destination}")
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, destination)
    destination.chmod(0o755 if source.stat().st_mode & 0o111 else 0o644)


def publish_prepare_receipt(root, destination, receipt):
    build = build_directory(root)
    if destination == build / "ic":
        write_json(build / "receipts" / "latest-prepare.json", {**receipt, "destination": str(destination)})


def prepare(root, manifest, *, force=False, revision=None, destination=None):
    upstream = validate_upstream(root, manifest)
    state = source_state(root, manifest)
    commit = revision or manifest["upstream"]["commit"]
    destination = destination or build_directory(root) / "ic"
    recipe = {"schema_version": SCHEMA, "upstream_commit": commit,
              "provider_git_commit": git(root, "rev-parse", "--verify", "HEAD").decode().strip(),
              "manifest_sha256": digest_bytes(canonical(manifest)), "sources": state}
    recipe_sha = digest_bytes(canonical(recipe))
    if destination.exists() or destination.is_symlink():
        verify_owned(root, destination)
        if not force:
            receipt = read_json(destination / ".git" / RECEIPT)
            current = checkout_digest(destination)
            if receipt.get("recipe_sha256") == recipe_sha and receipt.get("checkout_sha256") == current:
                publish_prepare_receipt(root, destination, receipt)
                print(f"Verified existing assembly: {destination}")
                return destination, receipt
            raise Error(f"Generated checkout or source inputs changed: {destination}\n"
                        "Review/preserve any generated edits, then run scripts/prepare --force to regenerate.")
    destination.parent.mkdir(parents=True, exist_ok=True)
    if destination.parent.is_symlink():
        raise Error(f"Refusing symlink destination parent: {destination.parent}")
    temporary = Path(tempfile.mkdtemp(prefix=f".{destination.name}.prepare-", dir=destination.parent))
    try:
        run(["git", "clone", "--quiet", "--no-hardlinks", "--dissociate", "--no-checkout",
             "--no-recurse-submodules", upstream, temporary])
        if (temporary / ".git" / "objects" / "info" / "alternates").exists():
            raise Error("Generated clone unexpectedly uses Git object alternates")
        try:
            git(temporary, "cat-file", "-e", f"{commit}^{{commit}}")
        except Error:
            git(temporary, "fetch", "--quiet", "--no-tags", str(upstream), commit)
        git(temporary, "checkout", "--quiet", "--detach", commit)
        upstream_paths = set(os.fsdecode(name) for name in git(temporary, "ls-tree", "-r", "-z", "--name-only", commit).split(b"\0") if name)
        patch_root = inside(root, manifest["patches"]["directory"], "patches.directory")
        for entry in state["patches"]:
            patch = inside(patch_root, entry["path"], "patch path")
            for name in patch_paths(temporary, patch):
                if name not in upstream_paths:
                    raise Error(f"Integration patch touches a path not owned by upstream: {name}. Put new owned files in overlay/.")
            git(temporary, "apply", "--check", "--binary", "--", str(patch))
            git(temporary, "apply", "--binary", "--", str(patch))
        profiler_root = inside(root, manifest["profiler"]["path"], "profiler.path")
        profiler_destination = str(relative(manifest["profiler"]["destination"], "profiler.destination"))
        if (temporary / profiler_destination).exists():
            raise Error(f"Profiler destination already exists in upstream: {profiler_destination}")
        for entry in state["profiler"]:
            source_root = consumer_directory(root) / "profiler" if entry.get("origin") == "consumer" else profiler_root
            copy_file(source_root / entry["path"], inside(temporary, profiler_destination + "/" + entry["path"], "profiler destination"))
        overlay_root = inside(root, manifest["overlay"]["path"], "overlay.path")
        for entry in state["overlay"]:
            name = entry["path"]
            if name in upstream_paths or name == profiler_destination or name.startswith(profiler_destination + "/"):
                raise Error(f"Overlay destination overlaps upstream or profiler ownership: {name}")
            source = inside(root, entry["source"], "overlay source") if "source" in entry else inside(overlay_root, name, "overlay path")
            copy_file(source, inside(temporary, name, "overlay destination"))
        receipt = {**recipe, "recipe_sha256": recipe_sha, "checkout_sha256": checkout_digest(temporary)}
        write_json(temporary / ".git" / OWNER, {"schema_version": SCHEMA, "root": str(root), "destination": str(destination)})
        write_json(temporary / ".git" / RECEIPT, receipt)
        # Build first so a failed patch or copy never destroys the previous assembly.
        if destination.exists():
            verify_owned(root, destination)
            old = destination.with_name(f".{destination.name}.previous-{os.getpid()}")
            if old.exists() or old.is_symlink():
                raise Error(f"Refusing to overwrite an unexpected backup: {old}")
            os.replace(destination, old)
            try:
                os.replace(temporary, destination)
            except BaseException:
                os.replace(old, destination)
                raise
            shutil.rmtree(old)
        else:
            os.replace(temporary, destination)
        publish_prepare_receipt(root, destination, receipt)
        print(f"Prepared {destination} at upstream {commit}")
        return destination, receipt
    finally:
        if temporary.exists():
            shutil.rmtree(temporary)


def refresh(root, manifest, check):
    if check:
        source_state(root, manifest)
        print("Manifest source inventory and integrity pins verified")
        return
    state = source_state(root, manifest, verify=False)
    manifest["patches"]["files"] = [{"path": item["path"], "sha256": item["sha256"]} for item in state["patches"]]
    manifest["overlay"]["files"] = [{key: item[key] for key in ("path", "source", "sha256") if key in item} for item in state["overlay"]]
    write_json(root / MANIFEST, manifest)
    print(f"Refreshed {MANIFEST}; review its diff before preparing")


def tool_version(checkout, command):
    return run(command, cwd=checkout).decode(errors="replace").strip()


def executable_paths(checkout, env):
    search = os.pathsep.join(str(Path(entry) if Path(entry).is_absolute() else checkout / entry)
                             for entry in env.get("PATH", os.defpath).split(os.pathsep))
    result = {}
    for name in ("cargo", "rustc"):
        command = env.get("RUSTC", "rustc") if name == "rustc" else "cargo"
        if "/" in command and not Path(command).is_absolute():
            command = str(checkout / command)
        path = shutil.which(command, path=search)
        if not path:
            raise Error(f"Cannot find {name} in PATH for {checkout}")
        result[f"{name}_path"] = path
        result[f"{name}_resolved_path"] = str(Path(path).resolve())
    return result


def cargo_command(root, manifest, command, arguments, force):
    checkout, assembly = prepare(root, manifest, force=force)
    env = os.environ.copy()
    embedded_inputs = []
    routing = checkout / "mainnet-routing-table.json"
    if routing.is_file():
        env.setdefault("MAINNET_ROUTING_TABLE", str(routing))
    if "MAINNET_ROUTING_TABLE" in env:
        routing = Path(env["MAINNET_ROUTING_TABLE"]).expanduser().resolve()
        read_json(routing)
        env["MAINNET_ROUTING_TABLE"] = str(routing)
        embedded_inputs.append(file_snapshot(routing, str(routing)))
    target = Path(env.get("CARGO_TARGET_DIR", str(build_directory(root) / "target"))).expanduser()
    if not target.is_absolute():
        target = Path.cwd() / target
    env["CARGO_TARGET_DIR"] = str(target.resolve())
    if arguments and arguments[0] == "--":
        arguments = arguments[1:]
    if any(arg == "--message-format" or arg.startswith("--message-format=") for arg in arguments):
        raise Error("The scripts manage Cargo's message format to record artifact hashes")
    package = "pocket-ic-server" if command == "build" else "ic-system-profiler"
    # An explicit package selection replaces the default, allowing native
    # integration tests whose test target does not exist in the core package.
    selected = any(arg in ("-p", "--package", "--workspace") or arg.startswith(("-p", "--package="))
                   for arg in arguments)
    packages = [] if selected else ["-p", package]
    args = ["cargo", command, "--locked", *packages, "--message-format=json-render-diagnostics", *arguments]
    try:
        profiler_commit = git(consumer_directory(root), "rev-parse", "--verify", "HEAD").decode().strip()
    except Error:
        profiler_commit = None
    provenance = {"schema_version": SCHEMA, "command": args, "cwd": str(checkout),
                  "upstream_commit": assembly["upstream_commit"],
                  "assembly_recipe_sha256": assembly["recipe_sha256"],
                  "profiler_sha256": digest_bytes(canonical(assembly["sources"]["profiler"])),
                  "patches_sha256": digest_bytes(canonical(assembly["sources"]["patches"])),
                  "overlay_sha256": digest_bytes(canonical(assembly["sources"]["overlay"])),
                  "profiler_git_commit": profiler_commit,
                  "provider_git_commit": git(root, "rev-parse", "--verify", "HEAD").decode().strip(),
                  "orchestration_sha256": digest_bytes(Path(__file__).read_bytes()),
                  "profiler_git_status": git(consumer_directory(root), "status", "--porcelain=v1", "--untracked-files=all").decode(),
                  "cargo_version": tool_version(checkout, ["cargo", "--version"]),
                  "rustc_version": tool_version(checkout, [env.get("RUSTC", "rustc"), "--version", "--verbose"]),
                  "cargo_target_dir": env["CARGO_TARGET_DIR"],
                  "build_environment": {name: env[name] for name in BUILD_ENVIRONMENT if name in env},
                  "embedded_inputs": embedded_inputs,
                  "native_server": file_snapshot(Path(env["POCKET_IC_BIN"]).resolve(), env["POCKET_IC_BIN"])
                  if command == "test" and env.get("POCKET_IC_BIN") else None,
                  **executable_paths(checkout, env),
                  "started_at": datetime.now(timezone.utc).isoformat(), "artifacts": []}
    toolchain = checkout / "rust-toolchain.toml"
    if not toolchain.is_file():
        toolchain = checkout / "rust-toolchain"
    provenance["toolchain_file"] = {"path": toolchain.name, "sha256": digest_bytes(toolchain.read_bytes()), "contents": toolchain.read_text()}
    print(f"Running {' '.join(args)} in {checkout}", flush=True)
    process = subprocess.Popen(args, cwd=checkout, env=env, stdout=subprocess.PIPE, text=True)
    artifacts = set()
    assert process.stdout is not None
    for line in process.stdout:
        try:
            message = json.loads(line)
        except ValueError:
            print(line, end="", flush=True)
            continue
        if message.get("reason") == "compiler-artifact" and message.get("executable"):
            artifacts.add(message["executable"])
        elif message.get("reason") == "compiler-message":
            rendered = message.get("message", {}).get("rendered")
            if rendered:
                print(rendered, end="", file=sys.stderr, flush=True)
    code = process.wait()
    for path in sorted(artifacts):
        provenance["artifacts"].append(file_snapshot(Path(path), path))
    provenance["exit_code"] = code
    provenance["finished_at"] = datetime.now(timezone.utc).isoformat()
    provenance["checkout_sha256_after"] = checkout_digest(checkout)
    receipts = build_directory(root) / "receipts"
    identity = provenance["started_at"].replace(":", "").replace(".", "")
    receipt_path = receipts / f"{command}-{identity}-{os.getpid()}.json"
    if receipt_path.exists():
        raise Error(f"Refusing to replace an earlier command receipt: {receipt_path}")
    write_json(receipt_path, provenance)
    write_json(receipts / f"latest-{command}.json", provenance)
    print(f"Recorded {command} provenance: {receipt_path}")
    if code:
        raise Error(f"cargo {command} failed with exit code {code}")
    if provenance["checkout_sha256_after"] != assembly["checkout_sha256"]:
        raise Error("Cargo changed the generated source checkout; inspect the checkout and receipt before regenerating")


def check_native_graph(root, manifest):
    checkout, _ = prepare(root, manifest)
    graph = run(["cargo", "tree", "--locked", "--offline", "-p", "pocket-ic-server",
                 "--edges", "normal,build", "--prefix", "none", "--format", "{p}"], cwd=checkout).decode()
    excluded = {"lifeline", "ic-nns-test-utils", "ic-nns-init"}
    present = {line.split()[0] for line in graph.splitlines() if line.strip()}
    if excluded & present:
        raise Error(f"CLI-only/Motoko dependencies entered the native provider graph: {sorted(excluded & present)}")
    print("Platform native graph verified: no lifeline or CLI-only NNS initialization")


def native_verify(root, manifest):
    run(["bash", root / "scripts/dev/native-tools.sh", "check"], capture=False)
    check_native_graph(root, manifest)
    previous = {name: os.environ.get(name) for name in
                ("CARGO_TARGET_DIR", "POCKET_IC_BIN", "SYSTEM_PROFILE_TEST_SERVER_URL")}
    try:
        target = build_directory(root) / "target"
        os.environ["CARGO_TARGET_DIR"] = str(target)
        os.environ.pop("SYSTEM_PROFILE_TEST_SERVER_URL", None)
        cargo_command(root, manifest, "build", [], False)
        cargo_command(root, manifest, "test", ["--", "--lib"], False)
        os.environ["POCKET_IC_BIN"] = str(target / "debug/pocket-ic-server")
        cargo_command(root, manifest, "test", ["--", "-p", "pocket-ic", "--test", "system_profiling"], False)
    finally:
        for name, value in previous.items():
            if value is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = value


def update_upstream(root, manifest, revision, fetch, force):
    upstream = validate_upstream(root, manifest)
    if revision.startswith("-") or not revision.strip():
        raise Error("Expected an upstream revision, not a Git option")
    if fetch:
        git(upstream, "fetch", "--no-tags", "origin", revision)
        revision = "FETCH_HEAD"
    commit = git(upstream, "rev-parse", "--verify", "--end-of-options", f"{revision}^{{commit}}").decode().strip()
    destination = build_directory(root) / "candidates" / commit
    if destination.parent.is_symlink():
        raise Error("Refusing symlink candidates directory")
    checkout, receipt = prepare(root, manifest, force=force, revision=commit, destination=destination)
    report = {"schema_version": SCHEMA, "pinned_commit": manifest["upstream"]["commit"],
              "candidate_commit": commit, "candidate_checkout": str(checkout),
              "assembly_recipe_sha256": receipt["recipe_sha256"],
              "status": "patches-and-owned-sources-apply; build-and-review-required", "pin_changed": False}
    report_path = build_directory(root) / "receipts" / f"upstream-{commit}.json"
    write_json(report_path, report)
    print(f"Candidate prepared for review: {checkout}\nThe gitlink and manifest pin are unchanged. Report: {report_path}")


def main(argv=None, *, root=None):
    root = Path(root).resolve() if root else Path(__file__).resolve().parent.parent
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    command = commands.add_parser("prepare", help="Assemble the pinned integration without fetching")
    command.add_argument("--force", action="store_true", help="Regenerate only a checkout already owned by this repository")
    commands.add_parser("check-native-graph", help="Check the selected native server graph without building")
    commands.add_parser("native-verify", help="Build and qualify this provider against the actual server")
    for name in ("build", "test"):
        command = commands.add_parser(name, help=f"Prepare and run native cargo {name}; record provenance")
        command.add_argument("--force", action="store_true", help="Regenerate an existing owned checkout after source edits")
        command.add_argument("cargo_args", nargs=argparse.REMAINDER, help="Additional native Cargo arguments after --")
    command = commands.add_parser("refresh-manifest", help="Refresh reviewed overlay and patch source pins")
    command.add_argument("--check", action="store_true", help="Read-only inventory and hash verification")
    command = commands.add_parser("update-upstream", help="Probe a candidate in isolation; do not change the pin")
    command.add_argument("revision")
    command.add_argument("--fetch", action="store_true", help="Fetch this revision from upstream origin before probing")
    command.add_argument("--force", action="store_true", help="Regenerate an existing owned candidate checkout")
    options = parser.parse_args(argv)
    try:
        manifest = load_manifest(root)
        if options.command == "refresh-manifest":
            refresh(root, manifest, options.check)
        else:
            with assembly_lock(root):
                if options.command == "prepare":
                    prepare(root, manifest, force=options.force)
                elif options.command == "check-native-graph":
                    check_native_graph(root, manifest)
                elif options.command == "native-verify":
                    native_verify(root, manifest)
                elif options.command in ("build", "test"):
                    cargo_command(root, manifest, options.command, options.cargo_args, options.force)
                else:
                    update_upstream(root, manifest, options.revision, options.fetch, options.force)
        return 0
    except (Error, OSError, KeyError, TypeError, ValueError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
