#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Start and stop an isolated reference SysON (official 2026.9.3 release jar).

  oracle.py up      start fresh Postgres + SysON on loopback, print state JSON
  oracle.py down    stop the processes recorded by the last `up`
  oracle.py status  print the recorded state and whether it still answers

Every `up` gets a new run directory with a fresh database. Nothing is registered
as a service, and only processes this script started are stopped. Logs and the
database directory are kept for inspection.
"""
import hashlib
import json
import os
import signal
import socket
import subprocess
import sys
import time
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

ASSETS = Path("/workspace/build/builderbob/syson-audit-20261026")
JAR = ASSETS / "syson-application-2026.9.3.jar"
JAR_SOURCE_REVISION = "b380dada415ce05155150f23cb3b74ae21cc45e4"
PG_ROOT = ASSETS / "postgres-root"
PG_BIN = PG_ROOT / "usr/lib/postgresql/15/bin"
PG_LIB = ASSETS / "postgres-lib"
JAVA = Path("/workspace/build/builderbob/ajhcs-sysml-java21-20261004/jdk-21.0.12.1+1-jre/bin/java")
NODE_BIN = Path("/workspace/build/builderbob/runtimes/node-v24.21.0-linux-x64/bin")
PINNED = {
    JAR: "85c031352695d6ac8f059f0215ca493c58a16805a111233998dd443d6990c9e1",
    PG_BIN / "initdb": "a5392ad4bb2d7998cb6b44535ad33b94c1ac3973dceab5b38612a40275383fcf",
    PG_BIN / "postgres": "2b94be1b4c0ae426325a3a598e952828790ff96d51d03631d5722171f1656ccd",
    JAVA: "999c141a6e620c71aa4230ccb62341a89162f37818753ff006d766bf935dcbe1",
}
RUNS = Path(os.environ.get("SYSON_ORACLE_RUNS", "/workspace/build/builderbob/syson-oracle/runs"))
STATE = RUNS.parent / "current.json"


def sha256(path):
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def free_port():
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def wait(predicate, proc, what, timeout):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if proc.poll() is not None:
            raise RuntimeError(f"{what} exited with {proc.returncode}")
        if predicate():
            return
        time.sleep(0.3)
    raise RuntimeError(f"{what} not ready after {timeout}s")


def port_open(port):
    try:
        with socket.create_connection(("127.0.0.1", port), timeout=0.3):
            return True
    except OSError:
        return False


def http_ok(url):
    try:
        with urllib.request.urlopen(url, timeout=2) as r:
            return r.status == 200
    except OSError:
        return False


def alive(pid, marker):
    try:
        return marker in Path(f"/proc/{pid}/cmdline").read_bytes()
    except OSError:
        return False


def up():
    if STATE.exists():
        state = json.loads(STATE.read_text())
        if any(alive(p["pid"], p["marker"].encode()) for p in state["processes"]):
            sys.exit(f"oracle already running: {state['url']} (run `down` first)")
    for path, digest in PINNED.items():
        if sha256(path) != digest:
            sys.exit(f"pinned asset changed: {path}")
    run = RUNS / datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    for sub in ("data", "socket", "tmp", "prefs"):
        (run / sub).mkdir(mode=0o700, parents=True)
    pg_port, app_port = free_port(), free_port()
    base_env = {"PATH": f"{NODE_BIN}:/usr/bin:/bin", "HOME": os.environ["HOME"],
                "LANG": "C", "LC_ALL": "C", "TMPDIR": str(run / "tmp")}
    pg_env = dict(base_env, LD_LIBRARY_PATH=str(PG_LIB))
    procs = []

    def spawn(name, argv, env, marker):
        out = (run / f"{name}.stdout.txt").open("wb")
        err = (run / f"{name}.stderr.txt").open("wb")
        p = subprocess.Popen(argv, cwd=run, env=env, stdout=out, stderr=err,
                             stdin=subprocess.DEVNULL, start_new_session=True)
        procs.append({"name": name, "pid": p.pid, "marker": marker})
        return p

    try:
        init = spawn("initdb", [str(PG_BIN / "initdb"), "-D", str(run / "data"),
                                "-L", str(PG_ROOT / "usr/share/postgresql/15"), "--locale=C",
                                "--encoding=UTF8", "--auth=trust", "--no-instructions"],
                     pg_env, "initdb")
        if init.wait(timeout=120) != 0:
            raise RuntimeError("initdb failed")
        procs.pop()
        db = spawn("postgres", [str(PG_BIN / "postgres"), "-D", str(run / "data"),
                                "-p", str(pg_port), "-h", "127.0.0.1", "-k", str(run / "socket"),
                                "-c", "unix_socket_permissions=0700", "-c", "shared_buffers=64MB",
                                "-c", "max_connections=30",
                                "-c", f"dynamic_library_path={PG_ROOT}/usr/lib/postgresql/15/lib"],
                   pg_env, str(run / "data"))
        wait(lambda: port_open(pg_port), db, "postgres", 30)
        props = run / "oracle.properties"
        props.write_text(
            f"server.address=127.0.0.1\nserver.port={app_port}\n"
            f"spring.datasource.url=jdbc:postgresql://127.0.0.1:{pg_port}/postgres\n"
            f"spring.datasource.username={os.environ.get('USER', 'builderbob')}\n"
            "spring.datasource.password=\n"
            "liquibase.analytics.enabled=false\nspring.liquibase.analytics-enabled=false\n"
            "management.server.port=-1\n"
            f"sirius.components.cors.allowedOriginPatterns=http://127.0.0.1:{app_port}\n")
        app = spawn("syson", [str(JAVA), "-Xms256m", "-Xmx2g", "-XX:ActiveProcessorCount=2",
                              f"-Djava.io.tmpdir={run / 'tmp'}",
                              f"-Djava.util.prefs.userRoot={run / 'prefs'}",
                              "-Dliquibase.analytics.enabled=false",
                              "-jar", str(JAR),
                              "--spring.config.location=classpath:/application.properties",
                              f"--spring.config.additional-location=file:{props}"],
                    base_env, str(props))
        url = f"http://127.0.0.1:{app_port}"
        wait(lambda: http_ok(url + "/"), app, "syson", 240)
    except BaseException:
        stop(procs)
        raise
    state = {"url": url, "run": str(run), "postgres_port": pg_port, "app_port": app_port,
             "started_utc": datetime.now(timezone.utc).isoformat(),
             "jar_sha256": PINNED[JAR], "jar_source_revision": JAR_SOURCE_REVISION,
             "processes": procs}
    STATE.write_text(json.dumps(state, indent=2) + "\n")
    (run / "state.json").write_text(STATE.read_text())
    print(json.dumps(state, indent=2))


def stop(procs):
    for p in reversed(procs):
        if alive(p["pid"], p["marker"].encode()):
            os.killpg(p["pid"], signal.SIGINT if p["name"] == "postgres" else signal.SIGTERM)
            for _ in range(100):
                if not alive(p["pid"], p["marker"].encode()):
                    break
                time.sleep(0.1)


def down():
    if not STATE.exists():
        print("no recorded oracle")
        return
    state = json.loads(STATE.read_text())
    stop(state["processes"])
    STATE.unlink()
    print(f"stopped; logs kept in {state['run']}")


def status():
    if not STATE.exists():
        print("no recorded oracle")
        return
    state = json.loads(STATE.read_text())
    state["answering"] = http_ok(state["url"] + "/")
    print(json.dumps(state, indent=2))


if __name__ == "__main__":
    {"up": up, "down": down, "status": status}.get(sys.argv[1] if len(sys.argv) > 1 else "", lambda: sys.exit(__doc__))()
