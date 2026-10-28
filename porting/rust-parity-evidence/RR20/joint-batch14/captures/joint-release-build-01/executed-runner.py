from pathlib import Path
import subprocess,os,json,time,hashlib,fcntl,signal,shutil
b=Path(__file__).parent;w=Path('/workspace/worktrees/builderbob/syson/batch14-history-summary-joint');o=b/'joint-release-build-01';o.mkdir();scratch=o/'scratch';scratch.mkdir();(o/'executed-runner.py').write_bytes(Path(__file__).read_bytes());lock=open('/workspace/build/builderbob/syson-audit-20261026/build/compiler.lock','r');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
for name in ['joint-browser-01','joint-callers-01','joint-callers-remainder-01','joint-baseline-red-02']:
 r=json.loads((b/name/'receipt.json').read_text());assert not r['final_descendant_cleanup']['remaining']
env=dict(os.environ,CARGO_BUILD_JOBS='1',CARGO_INCREMENTAL='0',RUST_TEST_THREADS='1',CARGO_TARGET_DIR=str(b/'target'),TMPDIR=str(scratch),PYTHONDONTWRITEBYTECODE='1');sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
old=json.loads((b/'joint-build-02/receipt.json').read_text());snap=lambda:{rel:sha(w/rel) for rel in old['source_after']}
r={'status':'RUNNING','profile':'release;default-features;wasm32-unknown-unknown','source_before':snap(),'commands':[],'native_test_basis':'151default/debugRustcases alreadypassed; unchangedsource; releaseWASMactualUIrequalificationrequired','debug_comparison':'Debugtimingobservationsremainseparate; no inferred release speedratio.'}
try:
 argv=['cargo','build','--offline','--locked','--release','--jobs','1','--manifest-path',str(w/'rust/crates/syson-web/Cargo.toml'),'--target','wasm32-unknown-unknown'];started=time.monotonic();p=subprocess.Popen(argv,cwd=w,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
 try:out,err=p.communicate(timeout=300);timed=False
 except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);out,err=p.communicate();timed=True
 (o/'00-stdout.txt').write_bytes(out);(o/'00-stderr.txt').write_bytes(err);r['commands'].append({'argv':argv,'exit_code':p.returncode,'timed_out':timed,'wall_seconds':time.monotonic()-started});assert p.returncode==0 and not timed,err.decode()[-1000:]
 wasm=b/'target/wasm32-unknown-unknown/release/syson_web.wasm';shutil.copyfile(wasm,o/'syson_web.wasm');r['wasm_sha256']=sha(wasm);r['wasm_bytes']=wasm.stat().st_size;r['status']='PASS_RELEASE_WASM_BUILD;ACTUAL_UI_QUALIFICATION_PENDING'
finally:
 r['source_after']=snap();r['source_unchanged']=r['source_before']==r['source_after'];assert r['source_unchanged'];r['artifacts']={p.name:sha(p) for p in o.iterdir() if p.is_file() and p.name!='receipt.json'};(o/'receipt.json').write_text(json.dumps(r,indent=2)+'\n')
print(json.dumps({'status':r['status'],'wasm_bytes':r.get('wasm_bytes'),'wasm_sha256':r.get('wasm_sha256')}),flush=True)
