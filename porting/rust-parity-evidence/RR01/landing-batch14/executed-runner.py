from pathlib import Path
import subprocess,os,json,time,hashlib,fcntl,signal,re
b=Path(__file__).parent;o=b/'foundation-checks-01';o.mkdir();s=o/'scratch';s.mkdir();target=b/'target';target.mkdir();(o/'executed-runner.py').write_bytes(Path(__file__).read_bytes())
lock=open('/workspace/build/builderbob/syson-audit-20261026/build/compiler.lock','r');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
env=dict(os.environ,CARGO_BUILD_JOBS='1',CARGO_INCREMENTAL='0',RUST_TEST_THREADS='1',CARGO_TARGET_DIR=str(target),TMPDIR=str(s),PYTHONDONTWRITEBYTECODE='1')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
r={'status':'RUNNING','scope':'Exact main-based standalone RR01 and S07 default model prerequisites only; no raw/graph/factory feature, no UI or full acceptance.','commands':[],'inputs':{}}
worktrees={90:Path('/workspace/worktrees/builderbob/syson/batch14-json-value-landing'),8:Path('/workspace/worktrees/builderbob/syson/batch14-model-landing')}
def snap(w):
 names=subprocess.check_output(['git','ls-files','-z','rust','porting/domain','porting/validation'],cwd=w).decode().split('\0')
 return {n:sha(w/n) for n in names if n and (w/n).is_file()}
for n,w in worktrees.items():r['inputs'][str(n)]={'worktree':str(w),'base':subprocess.check_output(['git','rev-parse','HEAD'],cwd=w,text=True).strip(),'staged_tree':subprocess.check_output(['git','write-tree'],cwd=w,text=True).strip(),'before':snap(w)}
def run(n,argv):
 t=time.monotonic();p=subprocess.Popen(argv,cwd=worktrees[n],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
 try:out,err=p.communicate(timeout=300);timed=False
 except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);out,err=p.communicate();timed=True
 i=len(r['commands']);(o/f'{i:02}-stdout.txt').write_bytes(out);(o/f'{i:02}-stderr.txt').write_bytes(err)
 r['commands'].append({'pr':n,'argv':argv,'exit_code':p.returncode,'timed_out':timed,'wall_seconds':time.monotonic()-t,'stdout':f'{i:02}-stdout.txt','stderr':f'{i:02}-stderr.txt','tests_passed':sum(map(int,re.findall(rb'test result: ok\. (\d+) passed;',out)))})
 (o/'receipt.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps({k:v for k,v in r['commands'][-1].items() if k!='argv'}),flush=True)
 return p.returncode==0 and not timed
try:
 for n,crate in [(90,'syson-json-value'),(8,'syson-model')]:
  w=worktrees[n];files=[str(p) for p in sorted((w/'rust/crates'/crate).rglob('*.rs'))]
  run(n,['rustfmt','--edition','2024','--config','skip_children=true','--check',*files])
  manifest=str(w/'rust/crates'/crate/'Cargo.toml')
  if run(n,['cargo','test','--offline','--locked','--jobs','1','--manifest-path',manifest,'--','--test-threads=1','--nocapture']):
   run(n,['cargo','clippy','--offline','--locked','--jobs','1','--manifest-path',manifest,'--all-targets','--','-D','warnings'])
   run(n,['cargo','check','--offline','--locked','--jobs','1','--manifest-path',manifest,'--target','wasm32-unknown-unknown'])
 r['status']='PASS_BOUNDED_PREREQUISITES' if all(c['exit_code']==0 and not c['timed_out'] for c in r['commands']) else 'PARTIAL_REPAIR_REQUIRED'
finally:
 for n,w in worktrees.items():
  r['inputs'][str(n)]['after']=snap(w);assert r['inputs'][str(n)]['before']==r['inputs'][str(n)]['after'],'source changed'
 r['artifact_sha256']={p.name:sha(p) for p in o.iterdir() if p.is_file() and p.name!='receipt.json'}
 (o/'receipt.json').write_text(json.dumps(r,indent=2)+'\n')
print(r['status'],flush=True)
