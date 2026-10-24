#!/usr/bin/env python3
"""One parent-allocated combined naming/containment gate, two serial commands."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,os,re,subprocess,time
ROOT=Path(__file__).resolve().parents[3]
BASE=Path("/workspace/build/builderbob/syson/initializer-containment-01/test-gate-01")
def sha(b):return hashlib.sha256(b).hexdigest()
def now():return datetime.now(timezone.utc)
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument("--monitor-sample",required=True);p.add_argument("--attempt",required=True);a=p.parse_args()
 if not re.fullmatch(r"attempt-[0-9]{2}",a.attempt):raise SystemExit("invalid attempt name")
 sample=datetime.fromisoformat(a.monitor_sample.replace("Z","+00:00"))
 frozen_path=ROOT/"porting/runs/initializer-containment-01/source-freeze.json";frozen=json.loads(frozen_path.read_text())
 out=BASE/a.attempt;out.mkdir(parents=True,exist_ok=False)
 receipt={"job_id":"initializer-containment-01/test-gate-01","attempt":a.attempt,"started_at_utc":now().isoformat(),"monitor_sample_utc":sample.isoformat(),"source_kind":frozen["source_kind"],"source_sha256":frozen["source_sha256"],"source_freeze_sha256":sha(frozen_path.read_bytes()),"runner_sha256":sha(Path(__file__).read_bytes()),"timeout_seconds":120,"scope":"23 graph/naming/bridge owner tests then 8 model containment owner tests. No upstream/GUI/worker.","phases":[]}
 env=os.environ.copy();env.update(CARGO_BUILD_JOBS="1",CARGO_NET_OFFLINE="true",CARGO_TARGET_DIR=str(BASE/"target"))
 receipt["environment"]={k:env[k] for k in ["CARGO_BUILD_JOBS","CARGO_NET_OFFLINE","CARGO_TARGET_DIR"]}
 commands=[
 ["graph","/home/builderbob/.local/bin/cargo","test","--offline","--locked","--jobs","1","--manifest-path","rust/crates/syson-initializer-graph/Cargo.toml","--","--test-threads=1"],
 ["containment","/home/builderbob/.local/bin/cargo","test","--offline","--locked","--jobs","1","--manifest-path","rust/crates/syson-model/Cargo.toml","--test","containment","--","--test-threads=1"]]
 code=1;deadline=time.monotonic()+120
 try:
  for path,h in frozen["source_sha256"].items():
   if sha((ROOT/path).read_bytes())!=h:raise ValueError("frozen source mismatch "+path)
  for name,*command in commands:
   phase={"name":name,"command":command,"started_at_utc":now().isoformat(),"monitor_age_seconds_at_launch":(now()-sample).total_seconds()}
   receipt["phases"].append(phase)
   if not 0<=phase["monitor_age_seconds_at_launch"]<=120:
    phase.update(not_run=True,stop_reason="stale monitor sample");break
   remaining=deadline-time.monotonic()
   if remaining<=0:phase.update(not_run=True,stop_reason="combined deadline reached");break
   try:
    result=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,timeout=remaining)
    stdout,stderr=result.stdout,result.stderr;phase.update(exit_code=result.returncode,timed_out=False);code=result.returncode
   except subprocess.TimeoutExpired as exc:
    stdout,stderr=exc.stdout or b"",exc.stderr or b"";phase.update(exit_code=None,timed_out=True);code=124
   phase["finished_at_utc"]=now().isoformat()
   for label,raw in [("stdout",stdout),("stderr",stderr)]:
    path=out/(name+"-"+label+".txt");path.write_bytes(raw);phase[label]={"path":path.name,"bytes":len(raw),"sha256":sha(raw)}
   if code:break
  if len(receipt["phases"])!=2 or any(x.get("not_run") for x in receipt["phases"]):code=1
  receipt["exit_code"]=code
 except Exception as exc:receipt["orchestrator_error"]=type(exc).__name__+": "+str(exc)
 finally:
  receipt["source_changed_during_attempt"]=[p for p,h in frozen["source_sha256"].items() if sha((ROOT/p).read_bytes())!=h]
  receipt["finished_at_utc"]=now().isoformat();(out/"receipt.json").write_text(json.dumps(receipt,indent=2)+"\n")
 print(json.dumps({"receipt":str(out/"receipt.json"),"exit_code":receipt.get("exit_code"),"phases":[{"name":x["name"],"exit_code":x.get("exit_code"),"not_run":x.get("not_run",False)} for x in receipt["phases"]],"orchestrator_error":receipt.get("orchestrator_error")}),flush=True)
 return code
if __name__=="__main__":raise SystemExit(main())
