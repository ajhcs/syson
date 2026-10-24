#!/usr/bin/env python3
"""One parent-allocated local naming query test attempt; no upstream/application run."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, os, re, subprocess, sys
ROOT = Path(__file__).resolve().parents[3]
BASE = Path("/workspace/build/builderbob/syson/initializer-naming-01/test-gate-01")
def sha(raw): return hashlib.sha256(raw).hexdigest()
def now(): return datetime.now(timezone.utc)
def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--monitor-sample", required=True)
    parser.add_argument("--attempt", required=True)
    args = parser.parse_args()
    if not re.fullmatch(r"attempt-[0-9]{2}", args.attempt):
        raise SystemExit("expected exact attempt-NN; no fallback path")
    sample = datetime.fromisoformat(args.monitor_sample.replace("Z","+00:00"))
    frozen_path = ROOT/"porting/runs/initializer-naming-01/source-freeze.json"
    frozen = json.loads(frozen_path.read_text())
    out = BASE/args.attempt
    out.mkdir(parents=True,exist_ok=False)
    receipt = {"job_id":"initializer-naming-01/test-gate-01","attempt":args.attempt,
        "started_at_utc":now().isoformat(),"monitor_sample_utc":sample.isoformat(),
        "source_kind":frozen["source_kind"],"source_sha256":frozen["source_sha256"],
        "source_freeze_sha256":sha(frozen_path.read_bytes()),
        "runner_sha256":sha(Path(__file__).read_bytes()),"timeout_seconds":120,
        "scope":"New raw graph/context/staged application tests only; no upstream/application/GUI/worker.",
        "manually_prepared_lock":"Included exact pre-run hash; Cargo --locked gate does not relabel its provenance."}
    command = ["/home/builderbob/.local/bin/cargo","test","--offline","--locked","--jobs","1",
        "--manifest-path","rust/crates/syson-initializer-graph/Cargo.toml","--","--test-threads=1"]
    env = os.environ.copy()
    env.update(CARGO_BUILD_JOBS="1",CARGO_TARGET_DIR=str(BASE/"target"),CARGO_NET_OFFLINE="true")
    receipt["command"]=command
    receipt["environment"]={k:env[k] for k in ["CARGO_BUILD_JOBS","CARGO_TARGET_DIR","CARGO_NET_OFFLINE"]}
    exit_code=1
    try:
        for path, expected in frozen["source_sha256"].items():
            if sha((ROOT/path).read_bytes()) != expected:
                raise ValueError("frozen source mismatch: "+path)
        age=(now()-sample).total_seconds()
        receipt["monitor_age_seconds_at_launch"]=age
        if age<0 or age>120:
            receipt.update(not_run=True,stop_reason="monitor sample stale/invalid before Cargo")
        else:
            receipt["cargo_started_at_utc"]=now().isoformat()
            try:
                result=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,timeout=120)
                stdout,stderr=result.stdout,result.stderr
                receipt.update(exit_code=result.returncode,timed_out=False)
                exit_code=result.returncode
            except subprocess.TimeoutExpired as exc:
                stdout,stderr=exc.stdout or b"",exc.stderr or b""
                receipt.update(exit_code=None,timed_out=True)
                exit_code=124
            receipt["cargo_finished_at_utc"]=now().isoformat()
            for label,data in [("stdout",stdout),("stderr",stderr)]:
                (out/(label+".txt")).write_bytes(data)
                receipt[label]={"path":label+".txt","bytes":len(data),"sha256":sha(data)}
        receipt["source_changed_during_attempt"]=[p for p,h in frozen["source_sha256"].items()
            if sha((ROOT/p).read_bytes())!=h]
    except Exception as exc:
        receipt["orchestrator_error"]=type(exc).__name__+": "+str(exc)
    finally:
        receipt["finished_at_utc"]=now().isoformat()
        (out/"receipt.json").write_text(json.dumps(receipt,indent=2)+"\n")
    print(json.dumps({"receipt":str(out/"receipt.json"),"exit_code":receipt.get("exit_code"),
        "not_run":receipt.get("not_run",False),"timed_out":receipt.get("timed_out"),
        "orchestrator_error":receipt.get("orchestrator_error")}),flush=True)
    return exit_code
if __name__=="__main__": raise SystemExit(main())
