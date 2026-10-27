# SPDX-License-Identifier: EPL-2.0
"""Root-executed evidence recipe. No network, server, database or product edits."""
import argparse
import copy
from dataclasses import dataclass
import fcntl
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import signal
import stat
import subprocess
import tomllib

@dataclass(frozen=True)
class NumberToken:
    token: str

def reject_constant(value):
    raise ValueError("non-JSON constant: " + value)

def unique_object(pairs):
    result = {}
    for key, value in pairs:
        assert key not in result, ("duplicate oracle/output key", key)
        result[key] = value
    return result

def literal(raw):
    assert len(raw) <= 64 * 1024 * 1024
    return json.loads(raw, parse_int=NumberToken, parse_float=NumberToken,
                      parse_constant=reject_constant, object_pairs_hook=unique_object)

def encode(value):
    if value is None:
        return "null"
    if value is True or value is False:
        return "true" if value else "false"
    if isinstance(value, NumberToken):
        return value.token
    if isinstance(value, (str, int)):
        return json.dumps(value, ensure_ascii=False)
    if isinstance(value, list):
        return "[" + ",".join(map(encode, value)) + "]"
    assert isinstance(value, dict)
    return "{" + ",".join(json.dumps(k) + ":" + encode(v) for k, v in value.items()) + "}"

def digest(path):
    hasher = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            hasher.update(chunk)
    return hasher.hexdigest()

def main():
    parser = argparse.ArgumentParser()
    for key in ["candidate", "target", "output", "compiler-lock"]:
        parser.add_argument("--" + key, type=Path, required=True)
    args = parser.parse_args()
    candidate = args.candidate.resolve(strict=True)
    target, output, lock_path = [p.resolve() for p in [args.target, args.output, args.compiler_lock]]
    assert candidate.is_relative_to("/workspace")
    assert all(p.is_relative_to("/workspace/build") and not p.is_relative_to(candidate)
               for p in [target, output, lock_path])
    output.mkdir(parents=True, exist_ok=False)
    scratch = output / "scratch"
    scratch.mkdir()
    evidence = candidate / "porting/rust-parity-evidence/RR23"
    provenance = candidate / "porting/rust-parity-prerequisites/RR23/source-manifest.json"
    specification = json.loads((evidence / "cases.json").read_text())
    manifest = json.loads(provenance.read_text())
    receipt = {"status": "FAILED_ATTEMPT", "commands": [], "native_epoch": "NOT_RUN",
               "scope": "Selected joint reader source/captured-contract qualification; full C01/UI/parity OPEN"}
    (output / "executed-runner.py").write_bytes(Path(__file__).read_bytes())
    generated = {}
    receipt["generated_inputs"] = generated
    receipt["generated_symlinks"] = {}
    env = dict(os.environ, CARGO_BUILD_JOBS="1", CARGO_INCREMENTAL="0", RUST_TEST_THREADS="1",
               CARGO_TARGET_DIR=str(target), TMPDIR=str(scratch), SYSON_TEST_DIR=str(scratch),
               SYSON_DB_CONTRACT_TEST_ROOT=str(scratch), SYSON_PROJECT_TEMPLATES_TEST_ROOT=str(scratch))
    def inputs():
        paths = {provenance, *(candidate / row["path"] for row in manifest["sources"])}
        paths.update(p for p in evidence.rglob("*") if p.is_file())
        assert all(stat.S_ISREG(p.lstat().st_mode) for p in paths)
        return {str(p.relative_to(candidate)): digest(p) for p in sorted(paths)}
    def command(name, argv, expected=0, timeout=240):
        stdout, stderr = output / (name + "-stdout.txt"), output / (name + "-stderr.txt")
        entry = {"name": name, "argv": list(map(str, argv)), "expected_exit_code": expected}
        receipt["commands"].append(entry)
        with stdout.open("wb") as out, stderr.open("wb") as err:
            try:
                process = subprocess.Popen(entry["argv"], cwd=candidate, env=env, stdout=out,
                                           stderr=err, start_new_session=True)
                code = process.wait(timeout=timeout)
                entry["exit_code"] = code
            except subprocess.TimeoutExpired:
                entry["timeout"] = True
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                entry["exit_code"] = process.wait()
                raise
        entry.update(stdout_sha256=digest(stdout), stderr_sha256=digest(stderr))
        assert code == expected, (name, code, expected)
        if expected != 0:
            assert stdout.stat().st_size == 0 and stderr.stat().st_size != 0
        return stdout
    def fixture(name, value):
        path = scratch / (name + ".json")
        path.write_text(encode(value))
        generated[str(path)] = digest(path)
        return path
    def cli(name, tool, arguments, expected=0):
        executable = target / "debug" / ("syson-" + tool)
        path = command("cli-" + name, [executable, *arguments], expected, 30)
        if expected == 0:
            assert (output / ("cli-" + name + "-stderr.txt")).stat().st_size == 0
        return path.read_bytes()
    def number(value):
        assert isinstance(value, NumberToken)
        return value.token
    try:
        with lock_path.open("a") as compiler_lock:
            fcntl.flock(compiler_lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            assert re.fullmatch("[0-9a-f]{40}", specification["candidate_base"])
            head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=candidate, text=True).strip()
            assert re.fullmatch("[0-9a-f]{40}", head)
            subprocess.check_call(["git", "merge-base", "--is-ancestor", specification["candidate_base"], head], cwd=candidate)
            receipt.update(candidate_base=specification["candidate_base"], candidate_head=head)
            changes = subprocess.check_output(["git", "diff", specification["candidate_base"], "--name-only"], cwd=candidate, text=True).splitlines()
            changes += subprocess.check_output(["git", "ls-files", "--others", "--exclude-standard"], cwd=candidate, text=True).splitlines()
            assert all(path.startswith("porting/rust-parity-evidence/RR23/") for path in changes)
            receipt["source_before"] = inputs()
            assert manifest["selected_executable_closure_files"] == 43 and len(manifest["sources"]) == 44
            for row in manifest["sources"]:
                path = PurePosixPath(row["path"])
                assert not path.is_absolute() and ".." not in path.parts
                assert re.fullmatch("[0-9a-f]{40}", row["git_blob"])
                data = subprocess.check_output(["git", "--no-replace-objects", "cat-file", "blob", row["git_blob"]], cwd=candidate)
                local = candidate / path
                assert hashlib.sha256(data).hexdigest() == row["sha256"] == digest(local)
                assert row["git_mode"] in ["100644", "100755"]
                assert bool(local.stat().st_mode & stat.S_IXUSR) == (row["git_mode"] == "100755")
            packages = [candidate / "rust/tools" / tool / "Cargo.toml" for tool in specification["tools"]]
            consumer = evidence / "consumer/Cargo.toml"
            tuples = lambda doc: {(p["name"], p["version"], p.get("checksum")) for p in doc["package"] if p.get("source", "").startswith("registry+")}
            known = set().union(*(tuples(tomllib.loads(p.with_name("Cargo.lock").read_text())) for p in packages))
            assert tuples(tomllib.loads(consumer.with_name("Cargo.lock").read_text())) <= known
            receipt["registry_tuples"] = sorted(known)
            rust = [candidate / p for p in receipt["source_before"] if p.endswith(".rs")]
            command("format", ["rustfmt", "--edition", "2024", "--config", "skip_children=true", "--check", *rust])
            providers = packages + [candidate / "rust/crates/syson-json-value/Cargo.toml"]
            for package, expected in zip(providers + [consumer], specification["suite_counts"], strict=True):
                name = package.parent.name
                stdout = command(name + "-test", ["cargo", "test", "--offline", "--locked", "--manifest-path", package, "--", "--test-threads=1", "--nocapture"])
                assert sum(map(int, re.findall(r"test result: ok\. (\d+) passed", stdout.read_text()))) == expected
                command(name + "-clippy", ["cargo", "clippy", "--offline", "--locked", "--manifest-path", package, "--all-targets", "--", "-D", "warnings"])
                extra = ["--tests"] if package == consumer else []
                command(name + "-build", ["cargo", "build", "--offline", "--locked", "--manifest-path", package, *extra])
            receipt["executables"] = {tool: digest(target / "debug" / ("syson-" + tool)) for tool in specification["tools"]}
            data = {key: literal((candidate / path).read_bytes()) for key, path in specification["fixtures"].items()}
            g, d, m, p, t = [data[key] for key in ["graphql", "db", "mappings", "probes", "templates"]]
            gql, db, http, templates = specification["tools"]
            paths = {key: candidate / path for key, path in specification["fixtures"].items()}
            actual = literal(cli("graphql-native", gql, [paths["graphql"]]))
            assert actual["schema"] == g["data"]["__schema"] and actual["counts"]["types"] == NumberToken("671") and actual["counts"]["members"] == NumberToken("3148")
            for name, flag, key in [("query", "--query", "query"), ("capabilities", "--capabilities", "capabilities")]:
                assert cli("graphql-" + name, gql, [flag]) == (candidate / specification[key]).read_bytes()
            cli("graphql-badfaith", gql, [candidate / specification["badfaith"]], 1)
            vendor = {"number": {"$serde_json::private::Number": "123"}, "raw": {"$serde_json::private::RawValue": '{"trap":[true]}'}, "n": NumberToken("-0"), "huge": NumberToken("9" * 4096)}
            edited = copy.deepcopy(g); edited["data"]["__schema"]["vendor"] = vendor
            assert literal(cli("graphql-markers", gql, [fixture("graphql-markers", edited)]))["schema"]["vendor"] == vendor
            native = literal(cli("db-native", db, [paths["db"]]))
            assert native == literal((candidate / specification["db_oracle"]).read_bytes())
            assert len(native["obligations"]) == 164 and len(native["migration_order"]) == 20
            for name in ["marker-oid", "marker-capture", "negative-zero"]:
                edited = copy.deepcopy(d)
                if name == "marker-oid":
                    edited["tables"][0]["oid"] = {"$serde_json::private::Number": number(edited["tables"][0]["oid"])}
                elif name == "marker-capture":
                    edited["capture"] = {"$serde_json::private::RawValue": encode(edited["capture"])}
                else:
                    edited["changelog"][0]["orderexecuted"] = NumberToken("-0")
                cli("db-" + name, db, [fixture("db-" + name, edited)], 1)
            for mode, original in [("mappings", m), ("probes", p)]:
                lane = "mappings" if mode == "mappings" else "capture"
                actual = literal(cli("http-native-" + mode, http, [mode, paths[mode]]))
                assert actual[lane] == original
                if mode == "mappings":
                    assert actual["summary"]["handlers"] == NumberToken("54") and actual["summary"]["mvc_handlers"] == NumberToken("33")
                else:
                    assert actual["summary"]["probes"] == NumberToken("22")
                    shell = next(row for row in p["observations"] if row["id"] == "graphql-get")["response"]["body_utf8"]
                    assert len(shell.encode()) == 504 and sum(row["response"].get("body_utf8") == shell for row in p["observations"]) == 9
            assert cli("http-manifest", http, ["--probes"]) == (candidate / specification["http_manifest"]).read_bytes()
            for mode, original in [("mappings", m), ("probes", p)]:
                edited = copy.deepcopy(original); edited["vendor"] = vendor
                lane = "mappings" if mode == "mappings" else "capture"
                assert literal(cli("http-" + mode + "-literals", http, [mode, fixture(mode + "-literals", edited)]))[lane] == edited
            edited = copy.deepcopy(p)
            zero = next(row for row in edited["observations"] if row["response"]["body_bytes"] == NumberToken("0"))
            zero["response"]["body_bytes"] = NumberToken("-0")
            assert literal(cli("http-zero-body", http, ["probes", fixture("zero-body", edited)]))["capture"] == edited
            edited = copy.deepcopy(m)
            context = next(iter(edited["contexts"].values()))
            entries = next(v for v in context["mappings"]["dispatcherServlets"].values() if v)
            entries[0]["handler"] = {"$serde_json::private::RawValue": '"handler"'}
            cli("http-typed-handler-marker", http, ["mappings", fixture("handler-marker", edited)], 1)
            edited = copy.deepcopy(p); edited["context"]["anonymous"] = {"$serde_json::private::RawValue": "true"}
            cli("http-typed-context-marker", http, ["probes", fixture("context-marker", edited)], 1)
            for name, extra, selection in specification["template_selections"]:
                actual = literal(cli("templates-" + name, templates, [paths["templates"], *extra]))
                assert actual["capture"] == t and actual["selection"] == selection
                if name == "all":
                    views = {row["id"]: row for row in actual["observations"]}
                    assert len(views) == 10
                    for failed in specification["template_errors"]:
                        assert views[failed]["outcome"] == "GRAPHQL_ERROR" and views[failed]["choices"] is None
                    assert views["modal-small-next"]["outcome"] == "SUCCESS" and views["modal-small-next"]["choices"] == []
            invalid = scratch / "invalid.json"; invalid.write_text("{"); generated[str(invalid)] = digest(invalid)
            link = scratch / "symlink.json"; link.symlink_to(paths["templates"])
            receipt["generated_symlinks"][str(link)] = str(paths["templates"])
            for name, arguments in [("usage", []), ("invalid", [invalid]), ("symlink", [link]), ("unknown", [paths["templates"], "all-templates", "missing"]), ("failed-context", [paths["templates"], "browser-small-first", "blank-project"]), ("wrong-control", [paths["templates"], "modal-large", "upload-project"])]:
                cli("templates-" + name, templates, arguments, 1)
            edited = copy.deepcopy(t); edited["context"]["vendor"] = vendor
            row = next(v for v in edited["observations"] if v["id"] == "modal-large")
            row["request"]["variables"]["page"] = NumberToken("-0")
            body = literal(row["body_utf8"]); body["data"]["viewer"]["projectTemplates"]["pageInfo"]["count"] = NumberToken("-0")
            row["body_utf8"] = encode(body)
            actual = literal(cli("templates-literal-and-typed-zero", templates, [fixture("template-literals", edited), "all-templates", "sysmlv2-template"]))
            assert actual["capture"] == edited
            edited = copy.deepcopy(t); row = next(v for v in edited["observations"] if v["id"] == "all-templates")
            body = literal(row["body_utf8"]); body["data"]["viewer"]["allProjectTemplates"][0]["id"] = {"$serde_json::private::RawValue": '"sysmlv2-template"'}
            row["body_utf8"] = encode(body)
            cli("templates-nested-id-marker", templates, [fixture("nested-id-marker", edited)], 1)
            names = [entry["name"][4:] for entry in receipt["commands"] if entry["name"].startswith("cli-")]
            assert names == specification["cli_cases"] and len(names) == 30
            assert all(digest(Path(path)) == checksum for path, checksum in generated.items())
            assert link.is_symlink() and link.readlink() == paths["templates"]
            assert receipt["executables"] == {tool: digest(target / "debug" / ("syson-" + tool)) for tool in specification["tools"]}
            receipt["source_after"] = inputs()
            assert receipt["source_before"] == receipt["source_after"]
            receipt["status"] = "PASS_SELECTED_JOINT_CONSUMERS_AND_30_CLI;FULL_ACCEPTANCE_OPEN"
    finally:
        if "source_before" in receipt:
            try:
                receipt.setdefault("source_after", inputs())
                receipt["source_unchanged"] = receipt["source_before"] == receipt["source_after"]
            except Exception as error:
                receipt["source_after_error"] = repr(error)
                receipt["source_unchanged"] = False
        (output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")

if __name__ == "__main__":
    main()
