#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Run the synthetic reference journey before and after restarting owned Java."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import select
import signal
import socket
import subprocess
import sys
import time
import urllib.request
import zipfile

JAR_SHA = "85c031352695d6ac8f059f0215ca493c58a16805a111233998dd443d6990c9e1"
SOURCE = "b380dada415ce05155150f23cb3b74ae21cc45e4"
SCOPE = "syson-reference-restart"
CGROUP = Path("/proc/self/cgroup").read_bytes()


def check(condition, message):
    if not condition:
        raise RuntimeError(message)


def digest(content):
    return hashlib.sha256(content).hexdigest()


def dump(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def proc(record, run):
    pid = record["pid"]
    path = Path(f"/proc/{pid}")
    argv = (path / "cmdline").read_bytes().split(b"\0")[:-1]
    status = (path / "status").read_text().splitlines()
    uid_line = next(row for row in status if row.startswith("Uid:"))
    check(all(int(uid) == os.getuid() for uid in uid_line.split()[1:]), "wrong PID owner")
    check((path / "cgroup").read_bytes() == CGROUP, "PID escaped owned scope")
    check(record["marker"].encode() in b"\0".join(argv), "wrong process marker")
    cwd = (path / "cwd").resolve(strict=True)
    expected_cwd = run / "data" if record["name"] == "postgres" else run
    check(cwd == expected_cwd, "wrong process cwd")
    check(os.getpgid(pid) == pid, "wrong process group")
    stat_fields = (path / "stat").read_text().rsplit(")", 1)[1].split()
    env_bytes = (path / "environ").read_bytes()
    env = dict(part.split(b"=", 1) for part in env_bytes.split(b"\0") if part)
    return {
        "pid": pid, "start_tick": int(stat_fields[19]),
        "executable": str((path / "exe").resolve(strict=True)), "cwd": str(cwd),
        "argv_sha256": digest(b"\0".join(argv)),
    }, argv, env


def wait_for_java_identity(record, run, app):
    marker = record["marker"].encode()
    cmdline = Path(f"/proc/{app.pid}/cmdline")
    deadline = time.monotonic() + 4
    while time.monotonic() < deadline:
        check(app.poll() is None, "replacement Java exited before identity readback")
        try:
            if marker in cmdline.read_bytes():
                return proc(record, run)[0]
        except FileNotFoundError:
            pass
        time.sleep(0.025)
    raise RuntimeError("replacement Java command marker unavailable")


def stop(record, identity, run, stop_signal):
    with os.fdopen(os.pidfd_open(identity["pid"]), "rb") as descriptor:
        check(proc(record, run)[0] == identity, "PID identity changed before signal")
        signal.pidfd_send_signal(descriptor.fileno(), stop_signal)
        poller = select.poll()
        poller.register(descriptor, select.POLLIN)
        check(bool(poller.poll(10000)), "owned process did not terminate")


def owned_limits():
    path = Path("/sys/fs/cgroup") / os.fsdecode(CGROUP.strip().split(b":", 2)[2]).lstrip("/")
    keys = ("cpu.max", "memory.high", "memory.max", "memory.swap.max", "pids.max")
    limits = {key: (path / key).read_text().strip() for key in keys}
    check(limits == {
        "cpu.max": "200000 100000", "memory.high": "3221225472", "memory.max": "4294967296",
        "memory.swap.max": "0", "pids.max": "256",
    }, "unexpected resource limits")
    return limits, path


def listening(port):
    try:
        with socket.create_connection(("127.0.0.1", port), timeout=0.3):
            return True
    except OSError:
        return False


def ready(state, app):
    expires = time.monotonic() + 60
    while time.monotonic() < expires:
        check(app.poll() is None, "replacement Java exited during startup")
        try:
            with urllib.request.urlopen(state["url"] + "/", timeout=1) as reply:
                if reply.status == 200:
                    return
        except OSError:
            pass
        time.sleep(0.25)
    raise RuntimeError("replacement Java did not answer")


def db_identity(state, record, run):
    data = run / "data"
    stat = data.stat()
    pid_file = (data / "postmaster.pid").read_text().splitlines()
    check(pid_file[:2] == [str(record["pid"]), str(data)], "unexpected database pid/data")
    check(pid_file[3] == str(state["postgres_port"]) and listening(state["postgres_port"]),
          "expected database listener unavailable")
    return {"process": proc(record, run)[0], "data_dev": stat.st_dev, "data_inode": stat.st_ino,
            "data_directory": str(data), "port": state["postgres_port"]}


def objects(item):
    if isinstance(item, dict):
        yield item
        for child in item.values():
            yield from objects(child)
    elif isinstance(item, list):
        for child in item:
            yield from objects(child)


def archive_projection(path, saved):
    with zipfile.ZipFile(path) as z:
        entries = z.infolist()
        names = [entry.filename for entry in entries]
        check(len(names) == len(set(names)) and len(names) == 3 and
              all(Path(name).parts[0] == "RestartDemo" and ".." not in Path(name).parts for name in names),
              "unexpected project ZIP entries")
        check(all(entry.file_size < 1024 * 1024 for entry in entries), "oversized synthetic entry")
        check(z.testzip() is None, "bad ZIP CRC")
        doc = json.loads(z.read(next(n for n in names if "/documents/" in n)))
        rep = json.loads(z.read(next(n for n in names if "/representations/" in n)))
        manifest = json.loads(z.read("RestartDemo/manifest.json"))
        entries = {name: z.read(name) for name in sorted(names)}
        entry_hashes = {name: digest(content) for name, content in entries.items()}
        documentation_exported = any(
            b"Verified project restart documentation" in content for content in entries.values()
        )
    nodes = list(objects(doc))

    def unique(predicate, message):
        matches = [node for node in nodes if predicate(node)]
        check(len(matches) == 1, message)
        return matches[0]

    def named(kind, name):
        return unique(lambda node: node.get("eClass") == f"sysml:{kind}" and
                      node.get("data", {}).get("declaredName") == name,
                      f"missing or duplicate {kind} {name}")

    part_type = named("PartDefinition", "Wheel")
    usage = named("PartUsage", "wheel")
    package = named("Package", "ArchiveDemo")
    view = named("ViewUsage", "Archive overview")
    typing = unique(lambda node: node.get("eClass") == "sysml:FeatureTyping" and
                    node["data"].get("typedFeature") == usage["id"], "missing/duplicate FeatureTyping")
    check(typing["data"].get("type") == part_type["id"] and
          typing in usage["data"]["ownedRelationship"], "FeatureTyping endpoint or owner changed")
    check(rep["projectId"] == saved["projectId"] and rep["targetObjectId"] == view["id"] and
          rep["id"] == saved["route"].split("?", 1)[0].rsplit("/", 1)[1] and
          rep["label"] == "Archive overview" and rep["kind"] == "siriusComponents://representation?type=Diagram",
          "unexpected project or representation identity")
    rep_nodes = rep["representation"]["nodes"]
    check(len(rep_nodes) == 1 and rep_nodes[0]["targetObjectId"] == package["id"],
          "rendered package projection changed")
    return {
        "entry_names": sorted(names),
        "representation_documentation_text_exported": documentation_exported,
        "source_document_names": sorted(manifest["documentIdsToName"].values()),
        "ids": {k: v["id"] for k, v in [
            ("type", part_type), ("usage", usage), ("package", package), ("view", view), ("typing", typing)]},
        "typing": {key: typing["data"][key] for key in ("type", "typedFeature")},
        "representation": {key: rep[key] for key in ("id", "projectId", "descriptionId", "targetObjectId", "label", "kind")},
        "diagram_package_node": {key: rep_nodes[0][key] for key in ("id", "targetObjectId", "targetObjectKind")},
    }, entry_hashes


def browser(args, phase, out, state, environment):
    environment = dict(environment)
    browser_libraries = environment.get("SYSON_RESTART_BROWSER_LD_LIBRARY_PATH")
    if browser_libraries:
        environment["LD_LIBRARY_PATH"] = browser_libraries
    with (out / f"{phase}-browser.log").open("wb") as log:
        subprocess.run([args.node, str(Path(__file__).with_name("browser.cjs")),
                        phase, str(out), state["url"]], env=environment,
                       stdout=log, stderr=subprocess.STDOUT, check=True, timeout=65)


def main():
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument("oracle", type=Path, help="Existing pinned porting/oracle/oracle.py")
    cli.add_argument("output", type=Path, help="New private output directory in /workspace/build")
    cli.add_argument("--node", required=True)
    args = cli.parse_args()
    out = args.output.resolve()
    check(out.is_relative_to(Path("/workspace/build")), "write only to verified workspace build")
    check(f"/{SCOPE}-".encode() in CGROUP, "requires the owned resource scope")
    pg_socket = out / "runs" / "20000101T000000Z" / "socket" / ".s.PGSQL.65535"
    check(len(os.fsencode(pg_socket)) <= 107, "use a shorter workspace output path for PostgreSQL's socket")
    limits, cgroup_path = owned_limits()
    out.mkdir(mode=0o700)
    temp = out / "tmp"
    temp.mkdir(mode=0o700)
    environment = dict(os.environ, SYSON_ORACLE_RUNS=str(out / "runs"), TMPDIR=str(temp),
                       XDG_CONFIG_HOME=str(temp / "config"), XDG_CACHE_HOME=str(temp / "cache"))
    with (out / "oracle-up.json").open("wb") as stdout, (out / "oracle-up.stderr").open("wb") as stderr:
        subprocess.run([sys.executable, "-I", "-B", args.oracle.resolve(strict=True), "up"],
                       stdout=stdout, stderr=stderr, check=True, env=environment, timeout=110)
    state_path = out / "current.json"
    state = json.loads(state_path.read_text())
    check(state["jar_sha256"] == JAR_SHA and state["jar_source_revision"] == SOURCE, "wrong artifact")
    run = Path(state["run"]).resolve()
    check(run.parent == out / "runs", "unexpected run")
    records = {record["name"]: record for record in state["processes"]}
    pg, java = records["postgres"], records["syson"]
    props = run / "oracle.properties"
    restarted = None
    new_identity = None
    app_identity, argv, java_env = proc(java, run)
    pg_identity = proc(pg, run)[0]
    try:
        db_before = db_identity(state, pg, run)
        config_sha = digest(props.read_bytes())
        check(digest(Path(os.fsdecode(argv[argv.index(b"-jar") + 1])).read_bytes()) == JAR_SHA, "JAR bytes changed")
        check(f"postgresql://127.0.0.1:{db_before['port']}/postgres" in props.read_text(), "wrong JDBC target")
        browser(args, "before", out, state, environment)
        saved = json.loads((out / "journey.json").read_text())
        before, before_entry_hashes = archive_projection(out / "before.zip", saved)
        stop(java, app_identity, run, signal.SIGTERM)
        java = None
        check(not listening(state["app_port"]), "old app port remains open")
        db_between = db_identity(state, pg, run)
        check(db_between == db_before, "database changed during Java shutdown")
        check(digest(props.read_bytes()) == config_sha, "Java config changed")
        with (out / "java-after.stdout").open("wb") as stdout, (out / "java-after.stderr").open("wb") as stderr:
            restarted = subprocess.Popen(argv, cwd=run, env=java_env, stdout=stdout, stderr=stderr,
                                         stdin=subprocess.DEVNULL, start_new_session=True)
        java = {"pid": restarted.pid, "marker": str(props), "name": "syson"}
        new_identity = wait_for_java_identity(java, run, restarted)
        check(new_identity["pid"] != app_identity["pid"] and
              new_identity["start_tick"] > app_identity["start_tick"] and
              new_identity["executable"] == app_identity["executable"] and
              new_identity["argv_sha256"] == app_identity["argv_sha256"], "Java argv/identity changed unexpectedly")
        ready(state, restarted)
        browser(args, "after", out, state, environment)
        after, after_entry_hashes = archive_projection(out / "after.zip", saved)
        check(before == after, "semantic or representation projections differ after restart")
        db_after = db_identity(state, pg, run)
        check(db_after == db_before and digest(props.read_bytes()) == config_sha, "database or config changed")
        before_ui = json.loads((out / "before.json").read_text())
        after_ui = json.loads((out / "after.json").read_text())
        check(before_ui["dom"] == after_ui["dom"], "rendered Details projection changed")
        dump(out / "result.json", {
            "classification": "official release reference only",
            "release_sha256": JAR_SHA, "source_label": SOURCE,
            "resource_limits": limits,
            "observed_memory_peak_bytes": int((cgroup_path / "memory.peak").read_text()),
            "same_database": {"before": db_before, "between": db_between, "after": db_after},
            "java": {"before": app_identity, "after": new_identity, "config_sha256": config_sha, "port_closed_between": True},
            "ui": before_ui["dom"], "archive_projection": before,
            "archives": {phase: {**json.loads((out / f"{phase}.json").read_text())["download"],
                                 "sha256": digest((out / f"{phase}.zip").read_bytes()),
                                 "entry_hashes": hashes}
                         for phase, hashes in (("before", before_entry_hashes), ("after", after_entry_hashes))},
        })
    finally:
        try:
            if java is not None and (restarted is None or restarted.poll() is None):
                identity = app_identity if restarted is None else new_identity
                if identity is not None:
                    stop(java, identity, run, signal.SIGTERM)
        finally:
            stop(pg, pg_identity, run, signal.SIGINT)
        if restarted is not None:
            restarted.wait(timeout=10)
        state_path.unlink()
        check(not listening(state["app_port"]) and not listening(state["postgres_port"]), "owned listeners remain")


def timed_out(signum, frame):
    raise TimeoutError("reference restart scope reached its time ceiling")


if __name__ == "__main__":
    signal.signal(signal.SIGTERM, timed_out)
    main()
