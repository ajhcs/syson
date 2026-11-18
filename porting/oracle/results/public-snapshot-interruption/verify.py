"""Observe public snapshot interruption using only direct owned children."""
from hashlib import sha256
from pathlib import Path
import json
import os
import resource
import select
import signal
import stat
import subprocess
from time import monotonic_ns

ROOT = Path(__file__).resolve().parent
STATES = ROOT / "states"
LOG = ROOT / "capture" / "runs"
EXE = ROOT / "target/release/syson-public-snapshot-process-probe"
SCOPE = Path("/proc/self/cgroup").read_bytes()
STEPS = ["create_staged", "write_staged", "sync_staged", "publish", "sync_directory", "remove_staged"]
FINAL = "rig.emf.json"
STAGE = "rig.emf.staged"
ORIGINAL = "original.emf.json"
RENAME = "Link Rev A <&> 雪🧭"
ALTERNATE = "Unpublishable alternative"
BASELINE = "Baseline sentinel"
SEEN = set()


def digest(data):
    return sha256(data).hexdigest()


def obj(i, kind, **attrs):
    return {
        "eClass": f"sysml:{kind}",
        "id": f"71000000-0000-4000-8000-{i:012x}",
        "data": {"elementId": f"72000000-0000-4000-8000-{i:012x}", **attrs},
    }


def expected(name):
    definition = obj(2, "PartDefinition", declaredName=name)
    typing = obj(6, "FeatureTyping", type="72000000-0000-4000-8000-000000000002",
                 typedFeature="72000000-0000-4000-8000-000000000004")
    usage = obj(4, "PartUsage", declaredName="link", isComposite=True, ownedRelationship=[typing])
    memberships = [
        obj(3, "OwningMembership", ownedRelatedElement=[definition]),
        obj(5, "OwningMembership", ownedRelatedElement=[usage]),
    ]
    return {
        "json": {"version": "1.0", "encoding": "utf-8"},
        "ns": {"sysml": "http://www.eclipse.org/syson/sysml"},
        "content": [obj(1, "Package", declaredName="Rig", ownedRelationship=memberships)],
    }


def canonical(name):
    return json.dumps(expected(name), ensure_ascii=False, sort_keys=True, indent=2).encode()


def owned_directory(path):
    entry = path.lstat()
    assert stat.S_ISDIR(entry.st_mode) and entry.st_uid == os.geteuid()
    assert stat.S_IMODE(entry.st_mode) == 0o700


def cap_file_output():
    resource.setrlimit(resource.RLIMIT_FSIZE, (65536, 65536))


def record(label, stdout, stderr):
    assert len(stdout) < 65536 and len(stderr) < 65536
    for suffix, output in (("stdout", stdout), ("stderr", stderr)):
        with (LOG / f"{label}.{suffix}").open("xb") as stream:
            stream.write(output)
    return {"stdout_sha256": digest(stdout), "stderr_sha256": digest(stderr)}


def process(label, root, mode, status, document=None):
    start = monotonic_ns()
    with subprocess.Popen(
        [EXE, mode, root], cwd=root, stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE, stderr=subprocess.PIPE, preexec_fn=cap_file_output,
    ) as child:
        try:
            stdout, stderr = child.communicate(timeout=10)
        except subprocess.TimeoutExpired:
            child.terminate()
            child.wait(timeout=5)
            raise
        end = monotonic_ns()
        raw = record(f"{label}-{mode}", stdout, stderr)
        assert child.returncode == status, (label, mode, child.returncode, stderr)
        assert not (stdout if status else stderr)
        report = json.loads(stderr if status else stdout)
        assert report["pid"] == child.pid and child.pid not in SEEN
        assert not Path(f"/proc/{child.pid}").exists()
        SEEN.add(child.pid)
    if document is not None:
        assert report["document"] == expected(document)
    return {"pid": report["pid"], "exit": status, "started_ns": start, "reaped_ns": end, "report": report, **raw}


def file_state(path, name, links):
    info = path.lstat()
    assert stat.S_ISREG(info.st_mode) and info.st_uid == os.geteuid()
    assert stat.S_IMODE(info.st_mode) == 0o600 and info.st_nlink == links
    raw = path.read_bytes()
    assert raw == canonical(name)
    return {
        "device": info.st_dev, "inode": info.st_ino, "hard_links": links,
        "size": info.st_size, "mtime_ns": info.st_mtime_ns, "ctime_ns": info.st_ctime_ns,
        "sha256": digest(raw),
    }


def disk(root, has_stage, has_final, distinct_stage=False):
    names = {ORIGINAL} | ({STAGE} if has_stage else set()) | ({FINAL} if has_final else set())
    assert {entry.name for entry in root.iterdir()} == names
    linked = has_stage and has_final and not distinct_stage
    links = 2 if linked else 1
    stage = file_state(root / STAGE, ALTERNATE if distinct_stage else RENAME, links) if has_stage else None
    final = file_state(root / FINAL, RENAME, links) if has_final else None
    baseline = file_state(root / ORIGINAL, BASELINE, 1)
    if linked:
        assert stage == final
    if distinct_stage:
        assert (stage["device"], stage["inode"]) != (final["device"], final["inode"])
    for state in (stage, final):
        if state is not None:
            assert (state["device"], state["inode"]) != (baseline["device"], baseline["inode"])
    return {"original": baseline, "stage": stage, "final": final}


def fields(path):
    return dict(line.split(":", 1) for line in path.read_text().splitlines() if ":" in line)


def pidfd_pid(pidfd):
    assert os.readlink(f"/proc/self/fd/{pidfd}") == "anon_inode:[pidfd]"
    return int(fields(Path(f"/proc/self/fdinfo/{pidfd}"))["Pid"])


def identify(child, pidfd, argv, root):
    assert child.poll() is None and child.pid > 1 and child.pid not in SEEN
    assert pidfd_pid(pidfd) == child.pid
    proc = Path(f"/proc/{child.pid}")
    status = fields(proc / "status")
    assert int(status["PPid"]) == os.getpid()
    assert all(int(uid) == os.geteuid() for uid in status["Uid"].split())
    assert (proc / "exe").resolve(strict=True) == EXE
    assert (proc / "cwd").resolve(strict=True) == root
    assert (proc / "cgroup").read_bytes() == SCOPE
    assert (proc / "cmdline").read_bytes().split(b"\0") == [*map(os.fsencode, argv), b""]
    tokens = (proc / "stat").read_text().rsplit(")", 1)[1].split()
    return {
        "pid": child.pid, "supervisor_pid": os.getpid(), "start_tick": int(tokens[19]),
        "same_uid_cgroup_executable_args_and_cwd": True,
    }


def read_marker(pipe):
    deadline = monotonic_ns() + 5_000_000_000
    received = b""
    while not received.endswith(b"\n"):
        remaining = max(0, deadline - monotonic_ns()) / 1_000_000_000
        assert select.select([pipe], [], [], remaining)[0], "barrier marker never arrived"
        chunk = os.read(pipe.fileno(), 1024)
        assert chunk and len(received) + len(chunk) < 1024
        received += chunk
    assert received.count(b"\n") == 1
    return received


def run_interruption(point, root, has_stage, has_final):
    argv = [EXE, "interrupt", root, point]
    expected_steps = STEPS[:STEPS.index(point) + 1]
    start = monotonic_ns()
    child = subprocess.Popen(argv, cwd=root, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                             stderr=subprocess.PIPE, preexec_fn=cap_file_output)
    pidfd = None
    try:
        pidfd = os.pidfd_open(child.pid)
        marker_raw = read_marker(child.stderr)
        marker = json.loads(marker_raw)
        assert marker == {"pid": child.pid, "barrier": point, "attempted": expected_steps,
                          "completed": expected_steps}
        identity = identify(child, pidfd, argv, root)
        wrong = os.pidfd_open(os.getpid())
        try:
            assert pidfd_pid(wrong) == os.getpid() != child.pid
            try:
                identify(child, wrong, argv, root)
            except AssertionError:
                pass
            else:
                raise AssertionError("supervisor pidfd unexpectedly admitted")
        finally:
            os.close(wrong)
        before = disk(root, has_stage, has_final)
        assert identify(child, pidfd, argv, root) == identity
        signal.pidfd_send_signal(pidfd, signal.SIGKILL)
        assert select.select([pidfd], [], [], 5)[0], "direct writer did not exit"
        notice = os.waitid(os.P_PIDFD, pidfd, os.WEXITED | os.WNOWAIT | os.WNOHANG)
        assert (notice.si_pid, notice.si_code, notice.si_status) == (child.pid, os.CLD_KILLED, signal.SIGKILL)
        stdout, stderr = child.communicate(timeout=5)
        end = monotonic_ns()
        assert child.returncode == -signal.SIGKILL and pidfd_pid(pidfd) == -1
        assert not stdout and not stderr and not Path(f"/proc/{child.pid}").exists()
        SEEN.add(child.pid)
        receipt = record(f"{point}-writer", stdout, marker_raw + stderr)
    finally:
        try:
            if child.poll() is None:
                # Rejecting admission never sends the experimental signal.
                child.terminate()
                child.wait(timeout=5)
        finally:
            for stream in (child.stdin, child.stdout, child.stderr):
                stream.close()
            if pidfd is not None:
                os.close(pidfd)
    after = disk(root, has_stage, has_final)
    assert before == after
    return {
        **identity, **receipt, "exit": child.returncode, "waitid": {
            "pid": notice.si_pid, "code": notice.si_code, "signal": notice.si_status,
        },
        "wrong_pidfd_rejected": True, "started_ns": start, "reaped_ns": end,
        "barrier": marker, "before_signal": before, "after_wait": after,
    }


def seed(label):
    root = STATES / label
    root.mkdir(mode=0o700)
    owned_directory(root)
    prepared = process(label, root, "seed", 0, BASELINE)
    assert prepared["report"]["snapshot"] == ORIGINAL
    return root, prepared, disk(root, False, False)["original"]


def fresh(label, root, has_final, after):
    row = process(label, root, "read", 0 if has_final else 2, RENAME if has_final else None)
    assert after < row["started_ns"]
    if not has_final:
        assert row["report"]["operation"] == "load" and row["report"]["io_kind"] == "NotFound"
        assert row["report"]["raw_os_error"] == 2
    original = process(label, root, "read-original", 0, BASELINE)
    assert row["reaped_ns"] < original["started_ns"]
    return {"candidate": row, "original": original}


def main():
    assert ROOT == Path.cwd() and ROOT.resolve(strict=True) == ROOT
    owned_directory(ROOT)
    for directory in (STATES, LOG):
        directory.mkdir(mode=0o700)
        owned_directory(directory)
    prior = json.loads((ROOT / "prior/evidence.json").read_bytes())
    assert digest(canonical(RENAME)) == prior["published_sha256"]
    assert digest(canonical(ALTERNATE)) == prior["duplicate_diagnostic_stage"]["sha256"]
    for label, name in (("expected", RENAME), ("baseline", BASELINE), ("alternate", ALTERNATE)):
        (LOG / f"{label}.json").write_bytes(canonical(name))
    root, normal_seed, normal_original = seed("normal")
    writer = process("normal", root, "observe", 0, RENAME)
    assert writer["report"]["completed"] == writer["report"]["attempted"] == STEPS
    assert writer["report"]["snapshot"] == FINAL
    normal_disk = disk(root, False, True)
    assert normal_disk["original"] == normal_original
    loads = fresh("normal", root, True, writer["reaped_ns"])
    assert disk(root, False, True) == normal_disk
    normal = {
        "seed": normal_seed, "writer": writer, "disk": normal_disk, "fresh": loads,
    }
    cases = {}
    for point, has_stage, has_final in (
        ("sync_staged", True, False), ("publish", True, True),
        ("sync_directory", True, True), ("remove_staged", False, True),
    ):
        root, seeded, baseline = seed(point)
        killed = run_interruption(point, root, has_stage, has_final)
        assert seeded["reaped_ns"] < killed["started_ns"]
        after = killed["after_wait"]
        assert after["original"] == baseline
        loads = fresh(point, root, has_final, killed["reaped_ns"])
        assert disk(root, has_stage, has_final) == after
        retry = process(point, root, "retry", 2)
        report = retry["report"]
        assert loads["original"]["reaped_ns"] < retry["started_ns"]
        assert report["operation"] == "save" and report["io_kind"] == "AlreadyExists"
        assert report["raw_os_error"] == 17
        earliest = 0 if has_stage else STEPS.index("publish")
        assert report["attempted"] == STEPS[:earliest + 1]
        assert report["completed"] == STEPS[:earliest]
        after_retry = disk(root, True, has_final, distinct_stage=not has_stage)
        if has_stage:
            assert after_retry == after
        else:
            assert after_retry["original"] == after["original"] and after_retry["final"] == after["final"]
        final_loads = fresh(f"{point}-after-retry", root, has_final, retry["reaped_ns"])
        assert disk(root, True, has_final, distinct_stage=not has_stage) == after_retry
        cases[point] = {
            "seed": seeded, "writer": killed, "fresh": loads, "retry": retry,
            "first_blocked_operation": STEPS[earliest], "after_retry": after_retry,
            "fresh_after_retry": final_loads,
        }
    assert len(SEEN) == 32
    print(json.dumps({
        "status":"PASS_PUBLIC_SNAPSHOTSTORE_PROCESS_INTERRUPTION",
        "normal":normal, "interruption_cases":cases, "unique_and_reaped_children":len(SEEN),
        "model_sha256":prior["published_sha256"], "scope_cgroup": SCOPE.decode().strip(),
        "application_or_power_loss_qualified":False,
    }, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
