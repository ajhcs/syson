from pathlib import Path
import subprocess,os,json,time,hashlib,fcntl,signal
b=Path(__file__).parent;a=json.loads((b/'prerequisite-candidate.json').read_text());w=Path(a['worktree']);o=b/'prerequisite-checks-01';o.mkdir();s=o/'scratch';s.mkdir();target=b/'prerequisite-target';target.mkdir();sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
(o/'executed-runner.py').write_bytes(Path(__file__).read_bytes());lock=open('/workspace/build/builderbob/syson-audit-20261026/build/compiler.lock','r');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
env=dict(os.environ,CARGO_BUILD_JOBS='1',CARGO_INCREMENTAL='0',RUST_TEST_THREADS='1',CARGO_TARGET_DIR=str(target),TMPDIR=str(s),PYTHONDONTWRITEBYTECODE='1')
paths=subprocess.check_output(['git','ls-files','-z'],cwd=w).decode().split('\0');files=[p for p in paths if p and (w/p).is_file()];snap=lambda:{p:sha(w/p) for p in files}
r={'status':'FAILED_ATTEMPT','candidate':a,'source_before':snap(),'commands':[],'held_features':'No raw factory, initializer graph or browser execution; pure algorithm/registry foundation qualification only'}
def run(argv):
 t=time.monotonic();p=subprocess.Popen(argv,cwd=w,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
 try:out,err=p.communicate(timeout=300);timed=False
 except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);out,err=p.communicate();timed=True
 i=len(r['commands']);(o/f'{i:02}-stdout.txt').write_bytes(out);(o/f'{i:02}-stderr.txt').write_bytes(err);r['commands'].append({'argv':argv,'exit_code':p.returncode,'timed_out':timed,'wall_seconds':time.monotonic()-t});(o/'receipt.json').write_text(json.dumps(r,indent=2)+'\n');assert not timed and p.returncode==0,(argv,err.decode()[-1500:])
try:
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=w,text=True).strip()==a['head']
 for stage in a['stages']:
  c=w/'rust/crates'/stage['crate'];rust=[str(w/p) for p in stage['paths'] if p.endswith('.rs')]
  run(['rustfmt','--edition','2024','--config','skip_children=true','--check',*rust])
  if stage['crate']=='syson-metamodel':run(['python3','-B',str(c/'tools/generate.py'),'--check'])
  run(['cargo','test','--offline','--locked','--jobs','1','--manifest-path',str(c/'Cargo.toml'),'--','--test-threads=1','--nocapture'])
  run(['cargo','clippy','--offline','--locked','--jobs','1','--manifest-path',str(c/'Cargo.toml'),'--all-targets','--','-D','warnings'])
 r['status']='PASS_DEFAULT_FOUNDATION_CANDIDATE;FULL_PARITY_OPEN'
finally:
 r['source_after']=snap();r['source_unchanged']=r['source_before']==r['source_after'];r['artifacts']={str(p.relative_to(o)):sha(p) for p in o.iterdir() if p.is_file() and p.name!='receipt.json'};assert r['source_unchanged'];(o/'receipt.json').write_text(json.dumps(r,indent=2)+'\n')
print(json.dumps({'status':r['status'],'commands':len(r['commands']),'source_files':len(files),'receipt':str(o/'receipt.json')}))
