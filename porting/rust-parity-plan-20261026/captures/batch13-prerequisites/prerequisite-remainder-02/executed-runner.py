from pathlib import Path
import subprocess,os,json,time,hashlib,fcntl,signal,re
b=Path(__file__).parent;a=json.loads((b/'prerequisite-candidate.json').read_text());w=Path(a['worktree']);o=b/'prerequisite-remainder-02';o.mkdir();s=o/'scratch';s.mkdir();target=b/'prerequisite-target';target.mkdir(exist_ok=True);sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();(o/'executed-runner.py').write_bytes(Path(__file__).read_bytes());lock=open('/workspace/build/builderbob/syson-audit-20261026/build/compiler.lock','r');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
env=dict(os.environ,CARGO_BUILD_JOBS='1',CARGO_INCREMENTAL='0',RUST_TEST_THREADS='1',CARGO_TARGET_DIR=str(target),TMPDIR=str(s),PYTHONDONTWRITEBYTECODE='1');paths=subprocess.check_output(['git','ls-files','-z'],cwd=w).decode().split('\0');files=[p for p in paths if p and (w/p).is_file()];snap=lambda:{p:sha(w/p) for p in files};r={'status':'FAILED_ATTEMPT','candidate':a,'source_before':snap(),'commands':[],'held_features':'Pure syntax/AST/declarative planning only; no model/raw/graph/factory/browser','format_policy':'Record existing donor formatting debt separately; do not modify donor bytes or relabel check result'}
def run(argv,allow_format_failure=False):
 t=time.monotonic();p=subprocess.Popen(argv,cwd=w,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
 try:out,err=p.communicate(timeout=300);timed=False
 except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);out,err=p.communicate();timed=True
 i=len(r['commands']);(o/f'{i:02}-stdout.txt').write_bytes(out);(o/f'{i:02}-stderr.txt').write_bytes(err);r['commands'].append({'argv':argv,'exit_code':p.returncode,'timed_out':timed,'wall_seconds':time.monotonic()-t,'existing_format_failure':allow_format_failure and p.returncode!=0});(o/'receipt.json').write_text(json.dumps(r,indent=2)+'\n')
 if not allow_format_failure:assert not timed and p.returncode==0,(argv,err.decode()[-1500:])
 return out
try:
 count=0
 for rel in ['rust/crates/syson-ast','rust/validation/ast','rust/crates/syson-initializer']:
  m=str(w/rel/'Cargo.toml');out=run(['cargo','test','--offline','--locked','--jobs','1','--manifest-path',m,'--','--test-threads=1','--nocapture']);count+=sum(map(int,re.findall(rb'test result: ok\. (\d+) passed;',out)));run(['cargo','clippy','--offline','--locked','--jobs','1','--manifest-path',m,'--all-targets','--','-D','warnings'],True)
 r['prior_attempt']='prerequisitechecks01formatdebt/wronggeneratorpath and remainder01independentsyntaxClippyfailure retained; onlyunexecuted AST/init remainder here';r['tests_passed']=count;assert count==64,count;r['format_failures']=[i for i,x in enumerate(r['commands']) if x['existing_format_failure']];r['status']='PASS_DEFAULT_FOUNDATION_SEMANTIC_CHECKS;FORMAT_DEBT_RETAINED' if r['format_failures'] else 'PASS_DEFAULT_FOUNDATION_CANDIDATE;FULL_PARITY_OPEN'
finally:
 r['source_after']=snap();r['source_unchanged']=r['source_before']==r['source_after'];r['artifacts']={str(p.relative_to(o)):sha(p) for p in o.iterdir() if p.is_file() and p.name!='receipt.json'};assert r['source_unchanged'];(o/'receipt.json').write_text(json.dumps(r,indent=2)+'\n')
print(json.dumps({'status':r['status'],'commands':len(r['commands']),'tests':r.get('tests_passed'),'format_failures':r.get('format_failures'),'receipt':str(o/'receipt.json')}))
