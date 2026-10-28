from pathlib import Path
import subprocess,os,json,time,hashlib,fcntl,signal,re,shutil
b=Path(__file__).parent;w=Path('/workspace/worktrees/builderbob/syson/batch14-history-summary-joint');o=b/'joint-build-01';o.mkdir();s=o/'scratch';s.mkdir();(o/'executed-runner.py').write_bytes(Path(__file__).read_bytes());lock=open('/workspace/build/builderbob/syson-audit-20261026/build/compiler.lock','r');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
env=dict(os.environ,CARGO_BUILD_JOBS='1',CARGO_INCREMENTAL='0',RUST_TEST_THREADS='1',CARGO_TARGET_DIR=str(b/'target'),TMPDIR=str(s),PYTHONDONTWRITEBYTECODE='1');sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
subprocess.run(['git','add','rust/crates/syson-model/src/lib.rs','rust/crates/syson-model/tests/rr20_regression.rs','rust/crates/syson-model/tests/batch14_history_summary.rs','porting/rust-parity-evidence/RR20/joint-batch14/history_summary_ui.cjs'],cwd=w,check=True)
paths=subprocess.check_output(['git','ls-files','-z','rust','porting/rust-parity-evidence/RR20/joint-batch14','porting/rust-parity-evidence/RR18b3a/fixtures'],cwd=w).decode().split('\0');snap=lambda:{p:sha(w/p) for p in paths if p and (w/p).is_file()}
r={'status':'RUNNING','worktree':str(w),'base':subprocess.check_output(['git','rev-parse','HEAD'],cwd=w,text=True).strip(),'executed_tree':subprocess.check_output(['git','write-tree'],cwd=w,text=True).strip(),'source_before':snap(),'commands':[],'scope':'Default-only RR20+borrowedsummary/currentUI composition; no rawfeature/graph/factory executions or main/UI/persistence/fullacceptance claim.'}
def run(argv):
 t=time.monotonic();p=subprocess.Popen(argv,cwd=w,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
 try:out,err=p.communicate(timeout=300);timed=False
 except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);out,err=p.communicate();timed=True
 i=len(r['commands']);(o/f'{i:02}-stdout.txt').write_bytes(out);(o/f'{i:02}-stderr.txt').write_bytes(err);entry={'argv':argv,'exit_code':p.returncode,'timed_out':timed,'wall_seconds':time.monotonic()-t,'tests_passed':sum(map(int,re.findall(rb'test result: ok\. (\d+) passed;',out)))};r['commands'].append(entry);(o/'receipt.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(entry),flush=True);assert p.returncode==0 and not timed,err.decode()[-1200:]
try:
 run(['rustfmt','--edition','2024','--config','skip_children=true','--check',*[str(w/p) for p in ['rust/crates/syson-model/src/lib.rs','rust/crates/syson-model/tests/rr20_regression.rs','rust/crates/syson-model/tests/batch14_history_summary.rs']]])
 for crate in ['syson-model','syson-web']:
  m=str(w/'rust/crates'/crate/'Cargo.toml')
  run(['cargo','test','--offline','--locked','--jobs','1','--manifest-path',m,'--','--test-threads=1','--nocapture'])
  run(['cargo','clippy','--offline','--locked','--jobs','1','--manifest-path',m,'--all-targets','--','-D','warnings'])
 run(['cargo','build','--offline','--locked','--jobs','1','--manifest-path',str(w/'rust/crates/syson-web/Cargo.toml'),'--target','wasm32-unknown-unknown'])
 wasm=b/'target/wasm32-unknown-unknown/debug/syson_web.wasm';shutil.copyfile(wasm,o/'syson_web.wasm');r['wasm_sha256']=sha(wasm);r['status']='PASS_JOINT_DEFAULT_MODEL_WEB_AND_WASM;UI_QUALIFICATION_PENDING'
finally:
 r['source_after']=snap();r['source_unchanged']=r['source_before']==r['source_after'];assert r['source_unchanged'];r['artifacts']={p.name:sha(p) for p in o.iterdir() if p.is_file() and p.name!='receipt.json'};(o/'receipt.json').write_text(json.dumps(r,indent=2)+'\n')
