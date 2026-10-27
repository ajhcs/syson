#!/usr/bin/env python3
"""RR16a: declared, committed, scoped checks; never full integration acceptance."""
import argparse
import datetime
import hashlib
import functools
import io
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import signal
import stat
import subprocess
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[1]
HELD = {"rust/crates/syson-initializer-graph/Cargo.toml"}
LIMITATIONS = ["RR16b full composed qualification remains OPEN",
               "Historical/external/toolchain completeness is declaration-only",
               "Before/after hashes do not detect hostile change then restoration",
               "Isolated candidate requires no other writer; no whole-host I/O proof"]


def need(ok, message):
    if not ok:
        raise ValueError(message)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def now():
    return datetime.datetime.now(datetime.timezone.utc).isoformat().replace("+00:00", "Z")


def relative(text):
    p = PurePosixPath(text)
    need(text and not p.is_absolute() and ".." not in p.parts and ":" not in text,
         "invalid repository input path")
    return p.as_posix()


def regular(path):
    meta = path.lstat()
    need(stat.S_ISREG(meta.st_mode), "nonregular/symlink input: " + str(path))
    return {"sha256": digest(path.read_bytes()),
            "mode": "100755" if meta.st_mode & stat.S_IXUSR else "100644"}


def git(root, *args, data=None):
    env = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
    env.update(GIT_CONFIG_GLOBAL=os.devnull, GIT_CONFIG_NOSYSTEM="1")
    return subprocess.check_output([str(Path(shutil.which("git")).resolve()), "--no-replace-objects", "-C", str(root),
                                    "-c", "core.worktree=" + str(root), *args], input=data,
                                   env=env, stderr=subprocess.PIPE)


@functools.lru_cache(maxsize=1)
def expected_tree(root, candidate):
    entries = [raw.split(b"\t", 1) for raw in git(root, "ls-tree", "-rz", "--full-tree", candidate).split(b"\0") if raw]
    objects = b"".join(info.split()[2] + b"\n" for info, _ in entries)
    stream = io.BytesIO(git(root, "cat-file", "--batch", data=objects))
    result = {}
    for info, name in entries:
        mode, kind, blob = info.decode().split()
        path = relative(name.decode())
        need(kind == "blob" and mode in {"100644", "100755"}, "unsupported Git mode: " + path)
        oid, object_type, size = stream.readline().decode().split()
        need(oid == blob and object_type == "blob", "unexpected Git object response")
        raw = stream.read(int(size))
        need(len(raw) == int(size) and stream.read(1) == b"\n", "truncated Git object")
        result[path] = {"sha256": digest(raw), "mode": mode, "git_blob": blob}
    return result


def snapshot(root, admission, profile):
    for parent in root.parents:
        need(not any(((parent / ".cargo" / name).exists() or (parent / ".cargo" / name).is_symlink()) for name in ["config", "config.toml"]), "unsupported ancestor Cargo configuration")
    if profile["env"].get("CARGO_HOME"):
        home = Path(profile["env"]["CARGO_HOME"])
        need(not any(((home / name).exists() or (home / name).is_symlink()) for name in ["config", "config.toml"]), "unsupported CARGO_HOME configuration")
    candidate = profile["candidate"]
    need(re.fullmatch(r"[0-9a-f]{40}", candidate), "full candidate commit required")
    need(git(root, "cat-file", "-t", candidate) == b"commit\n", "candidate object must be a commit")
    need(git(root, "rev-parse", "HEAD").decode().strip() == candidate, "candidate HEAD drift")
    for flags in [("--others", "--exclude-standard"),
                  ("--others", "--ignored", "--exclude-standard")]:
        need(not git(root, "ls-files", "-z", *flags), "untracked/ignored candidate inputs")
    rows = {}
    for path, expected in expected_tree(root, candidate).items():
        actual = regular(root / path)
        need(actual == {k: expected[k] for k in ["sha256", "mode"]}, "candidate byte/mode drift: " + path + " actual=" + str(actual))
        rows[path] = dict(actual, git_blob=expected["git_blob"])
    need(git(root, "diff", "--no-ext-diff", "--no-textconv", "--cached", "--name-only", candidate) == b"", "candidate index drift")
    external = {str(admission): regular(admission)}
    for item in profile["external_inputs"] + profile["tools"]:
        path = Path(item["path"])
        need(path.is_absolute(), "absolute external/tool input required")
        actual = regular(path)
        need(actual["sha256"] == item["sha256"], "external/tool drift: " + str(path))
        external[str(path)] = actual
    pins = {}
    for pin in profile["historical_pins"]:
        need(re.fullmatch(r"[0-9a-f]{40}", pin["revision"]), "full historical commit required")
        need(git(root, "cat-file", "-t", pin["revision"]) == b"commit\n", "historical revision must be a commit")
        path = relative(pin["path"])
        source = pin["revision"] + ":" + path
        need(git(root, "cat-file", "-t", source) == b"blob\n", "historical input must be a blob")
        value = digest(git(root, "cat-file", "blob", source))
        need(value == pin["sha256"] and pin["id"] not in pins, "historical pin mismatch/duplicate")
        pins[pin["id"]] = dict(pin)
    return {"repository": rows, "external": external, "historical": pins}


def inventory(root, files):
    manifests = {}
    workspaces = []
    for path in sorted(p for p in files if PurePosixPath(p).name == "Cargo.toml"):
        data = tomllib.loads((root / path).read_text())
        package = data.get("package", {})
        if data.get("workspace", {}).get("members"):
            workspaces.append(PurePosixPath(path).parent)
        reason = None
        if not package or package.get("workspace") or data.get("workspace", {}).get("members") or data.get("patch") or data.get("replace"):
            reason = "unsupported workspace closure"
        if data.get("features", {}).get("default"):
            reason = "unsupported default-feature closure"
        base = (root / path).parent
        targets = [data.get("lib", {})]
        for key in ["bin", "test", "example", "bench"]:
            targets += data.get(key, [])
        for target in targets:
            if target.get("proc-macro"):
                reason = "unsupported proc-macro execution closure"
            if "path" in target:
                source = (base / target["path"]).resolve()
                if not source.is_relative_to(root) or source.relative_to(root).as_posix() not in files:
                    reason = "target source outside committed closure"
        if package.get("build") is not False and (package.get("build") or (base / "build.rs").exists()):
            reason = "unsupported build-script closure"
        if str((base / "Cargo.lock").relative_to(root)) not in files:
            reason = "standalone lock missing"
        deps = []
        groups = [data]
        groups += list(data.get("target", {}).values())
        for group in groups:
            for key in ["dependencies", "dev-dependencies", "build-dependencies"]:
                for spec in group.get(key, {}).values():
                    if isinstance(spec, dict) and (spec.get("features") or spec.get("workspace")):
                        reason = "unsupported dependency feature/workspace activation"
                    if isinstance(spec, dict) and spec.get("optional"):
                        continue  # Feature flags/default feature activation refuse above/below.
                    if not isinstance(spec, dict) or "path" not in spec:
                        reason = "unsupported external dependency closure"
                        continue
                    target = (base / spec["path"] / "Cargo.toml").resolve()
                    if not target.is_relative_to(root):
                        reason = "out-of-repository local dependency"
                    else:
                        deps.append(target.relative_to(root).as_posix())
        manifests[path] = {"package": package.get("name"), "local_dependencies": deps,
                           "unsupported": reason, "disposition": "NOT_SELECTED"}
    for path, item in manifests.items():
        if any(PurePosixPath(path).parent != parent and PurePosixPath(path).is_relative_to(parent) for parent in workspaces):
            item["unsupported"] = "unsupported enclosing workspace closure"
    return manifests


def readiness(path, manifests, held, visiting=None):
    visiting = set() if visiting is None else visiting
    if path in held:
        return "HELD_NOT_RUN: " + held[path]
    if path not in manifests:
        return "unlisted local manifest: " + path
    if path in visiting:
        return "unsupported dependency cycle"
    item = manifests[path]
    if item["unsupported"]:
        return item["unsupported"]
    for dep in item["local_dependencies"]:
        error = readiness(dep, manifests, held, visiting | {path})
        if error:
            return error
    return None


def validate(profile, before, manifests):
    need(profile["schema"] == "RR16a/1", "unsupported admission schema")
    held = dict(profile["held_manifests"])
    held.update({path: "active historical initializer ownership hold" for path in HELD})
    held.update({path: "active factory validation ownership hold" for path in manifests
                 if path.startswith("rust/validation/factory/")})
    for path, item in manifests.items():
        error = readiness(path, manifests, held)
        if error:
            item.update(disposition="HELD_NOT_RUN" if path in held else "UNSUPPORTED_NOT_RUN", reason=error)
    ids = set()
    tools = {item["path"] for item in profile["tools"]}
    need(str(Path(shutil.which("git")).resolve()) in tools and str(Path(sys.executable).resolve()) in tools, "Git/Python tool declaration missing")
    for command in profile["commands"]:
        name, argv, kind = command["id"], command["argv"], command["kind"]
        need(name and re.fullmatch(r"[A-Za-z0-9_-]+", name) and name not in ids, "invalid/duplicate command ID")
        ids.add(name)
        need(argv and argv[0] in tools and all(isinstance(a, str) for a in argv), "undeclared command tool")
        need(0 < command["timeout_s"] <= 1800, "unsupported command deadline")
        need(set(command["required_pins"]) <= before["historical"].keys(), "required historical pin missing")
        need(set(command["required_inputs"]) <= before["repository"].keys(), "required repository input missing")
        if kind in {"cargo-test", "cargo-build"}:
            path = relative(command["manifest"])
            need(readiness(path, manifests, held) is None, "manifest not admitted: " + str(readiness(path, manifests, held)))
            expected = [argv[0], "test" if kind == "cargo-test" else "build", "--offline", "--locked", "--jobs", "1", "--manifest-path", path]
            need(argv[:8] == expected, "unsupported Cargo argv/manifest/feature scope")
            tails = [[], ["--", "--test-threads=1"]] if kind == "cargo-test" else [[], ["--target", "wasm32-unknown-unknown"]]
            need(argv[8:] in tails, "unsupported Cargo feature/workspace/config/filter scope")
            manifests[path]["disposition"] = "SELECTED"
        else:
            need(kind == "generated-check" and "--check" in argv and len(argv) >= 3,
                 "unsupported command kind")
            need(argv[1] in command["required_inputs"] and command["required_pins"],
                 "generated check requires script and historical pins")
    need(ids, "explicit nonempty selected scope required")
    return held


def run_command(command, root, env, evidence, index):
    started = now()
    try:
        process = subprocess.Popen(command["argv"], cwd=root, env=env,
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
    except OSError as error:
        return dict(command, started_at_utc=started, ended_at_utc=now(), exit_code=None,
                    timed_out=False, streams={}, reported_passed_tests=0, spawn_error=str(error))
    timed_out = False
    try:
        stdout, stderr = process.communicate(timeout=command["timeout_s"])
    except subprocess.TimeoutExpired:
        timed_out = True
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        stdout, stderr = process.communicate()  # Kill/wait the newly owned process group.
    streams = {}
    for label, raw in [("stdout", stdout), ("stderr", stderr)]:
        filename = f"{index:03d}-{command['id']}-{label}.txt"
        (evidence / filename).write_bytes(raw)
        portable = raw.replace(str(root).encode(), b"<candidate>").replace(env["CARGO_TARGET_DIR"].encode(), b"<target>")
        (evidence / (filename + ".portable")).write_bytes(portable)
        streams[label] = {"path": filename, "sha256": digest(raw), "portable_sha256": digest(portable)}
    text = (stdout + stderr).decode("utf-8", errors="replace")
    summaries = [line for line in text.splitlines() if line.startswith("test result:")]
    passed = sum(map(int, re.findall(r"test result: ok\. (\d+) passed;", text)))
    return dict(command, process_group=process.pid, started_at_utc=started, ended_at_utc=now(), exit_code=process.returncode, signal=(-process.returncode if process.returncode < 0 else None),
                timed_out=timed_out, streams=streams, reported_passed_tests=passed,
                test_result_lines=summaries)


def unique_object(items):
    result = {}
    for key, value in items:
        need(key not in result, "duplicate admission member")
        result[key] = value
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--build-dir", type=Path, required=True)
    parser.add_argument("--evidence-dir", type=Path, required=True)
    parser.add_argument("--admission", type=Path, required=True)
    parser.add_argument("--start-at")
    args = parser.parse_args()
    root = ROOT.resolve()
    need(args.build_dir.is_absolute() and args.evidence_dir.is_absolute() and args.admission.is_absolute(), "absolute paths required")
    build, evidence, admission = args.build_dir.resolve(strict=True), args.evidence_dir.resolve(), args.admission
    need(os.name == "posix", "POSIX owned process-group execution required")
    need(build.is_dir() and build.is_relative_to("/workspace") and not build.is_relative_to(root), "owned outside-candidate workspace build directory required")
    need(evidence.is_relative_to("/workspace") and not evidence.is_relative_to(root) and not evidence.exists(), "new outside-candidate evidence directory required")
    evidence.mkdir(parents=True)
    report = {"schema": "RR16a/1", "status": "REFUSED", "full_qualification": False,
              "limitations": LIMITATIONS, "started_at_utc": now(), "commands": []}
    code = 1
    try:
        regular(admission)
        raw_admission = admission.read_bytes()
        profile = json.loads(raw_admission, object_pairs_hook=unique_object)
        (evidence / "admission.json").write_bytes(raw_admission)
        report.update(candidate=profile["candidate"], admission_sha256=digest(raw_admission))
        before = snapshot(root, admission, profile)
        need(before["external"][str(admission)]["sha256"] == report["admission_sha256"], "admission changed while loading")
        report["source_sha256"] = {p: v["sha256"] for p, v in before["repository"].items()}
        report["before"] = before
        need(not any(PurePosixPath(p).name in {"config", "config.toml"} and ".cargo" in PurePosixPath(p).parts for p in before["repository"]), "unsupported repository Cargo configuration")
        manifests = inventory(root, before["repository"])
        report["inventory"] = manifests
        report["held_manifests"] = validate(profile, before, manifests)
        commands = profile["commands"]
        if args.start_at:
            index = [c["id"] for c in commands].index(args.start_at)
            report["unexecuted_prefix"] = [c["id"] for c in commands[:index]]
            skipped = commands[:index]
            commands = commands[index:]
            active = {c["manifest"] for c in commands}
            for item in skipped:
                if item["manifest"] and item["manifest"] not in active:
                    manifests[item["manifest"]]["disposition"] = "NOT_RUN_RESUME_PREFIX"
        report["selected_scope"] = [c["id"] for c in commands]
        need(not (build / "target").exists(), "fresh target directory required")
        (build / "target").mkdir()
        need(all(isinstance(k, str) and isinstance(v, str) for k, v in profile["env"].items()), "string environment required")
        allowed = {"HOME", "PATH", "CARGO_HOME", "RUSTUP_HOME", "RUSTC", "RUSTDOC", "TMPDIR", "SYSON_TEST_DIR", "SYSON_BUILD_DIR"}
        need(all(k in allowed or k.startswith("RR16_") for k in profile["env"]), "unsupported environment closure")
        env = dict(profile["env"], CARGO_BUILD_JOBS="1", CARGO_TARGET_DIR=str(build / "target"), PYTHONDONTWRITEBYTECODE="1", GIT_NO_REPLACE_OBJECTS="1", GIT_CONFIG_GLOBAL=os.devnull, GIT_CONFIG_NOSYSTEM="1")
        need(not any("RUSTFLAGS" in k or "RUSTDOCFLAGS" in k or "WRAPPER" in k or k.startswith("CARGO_ALIAS_") for k in env), "unsupported compiler environment")
        if any(c["kind"].startswith("cargo-") for c in commands):
            cargo_home = Path(env["CARGO_HOME"])
            need(cargo_home.is_absolute() and cargo_home.resolve().is_relative_to("/workspace"), "workspace CARGO_HOME required")
            need(not any((cargo_home / p).exists() for p in ["config", "config.toml"]), "unsupported CARGO_HOME configuration")
            need(env.get("RUSTC") in {t["path"] for t in profile["tools"]}, "declared explicit rustc required")
        for index, command in enumerate(commands):
            check = snapshot(root, admission, profile)
            need(check == before, "input closure changed before command")
            result = run_command(command, root, env, evidence, index)
            result["before"] = check
            report["commands"].append(result)
            try:
                result["after"] = snapshot(root, admission, profile)
                need(result["after"] == before, "input closure changed after command")
            except Exception as error:
                result["closure_error"] = str(error)
                raise
            need(not result["timed_out"] and result["exit_code"] == 0, "command failed or timed out")
            need(command["kind"] != "cargo-test" or result["reported_passed_tests"] > 0,
                 "Cargo test reported no passed cases; no test-scope success")
        report["after"] = snapshot(root, admission, profile)
        need(report["after"] == before, "final input closure changed")
        report["status"] = "SELECTED_CHECKS_PASSED"
        code = 0
    except Exception as error:
        report["status"] = "FAILED_SCOPED" if report["commands"] else "REFUSED"
        report["error"] = str(error)
    finally:
        report["not_run_commands"] = [name for name in report.get("selected_scope", []) if name not in {c["id"] for c in report["commands"]}]
        report["ended_at_utc"] = now()
        (evidence / "receipt.json").write_text(json.dumps(report, indent=2) + "\n")
    return code


if __name__ == "__main__":
    raise SystemExit(main())
