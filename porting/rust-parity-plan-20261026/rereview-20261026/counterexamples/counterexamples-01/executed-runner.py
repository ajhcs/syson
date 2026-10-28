from pathlib import Path
import json,hashlib,subprocess,os,fcntl,resource,signal,tomllib
b=Path(__file__).parent;out=b/'counterexamples-01';out.mkdir(exist_ok=False)
lock=open('/workspace/build/builderbob/syson-audit-20261026/build/compiler.lock','a');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
known=tomllib.loads(Path('/workspace/worktrees/builderbob/syson/rp-c03b-project-preparation/rust/crates/syson-project-preparation/Cargo.lock').read_text());tuples=lambda d:{(x['name'],x['version'],x.get('checksum')) for x in d['package'] if x.get('source','').startswith('registry+')};known=tuples(known)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();sources=[p for d in [b/'exports',b/'parser-review/probes'] for p in d.rglob('*') if p.is_file() and p.suffix in ['.rs','.toml','.lock']];before={str(p):sha(p) for p in sources}
env=dict(os.environ,CARGO_BUILD_JOBS='1',CARGO_INCREMENTAL='0',RUST_TEST_THREADS='1',CARGO_TARGET_DIR=str(b/'probe-target'),TMPDIR=str(out));receipt={'status':'FAIL','source_before':before,'cases':[]}
(out/'executed-runner.py').write_bytes(Path(__file__).read_bytes())
def limits():
 resource.setrlimit(resource.RLIMIT_AS,(512*1024*1024,512*1024*1024));resource.setrlimit(resource.RLIMIT_CPU,(20,20))
try:
 for name,binname in [('f33','rereview-f33-pretty-probe'),('f34','rereview-f34-name-probe'),('model-marker','rereview-model-marker-probe')]:
  manifest=b/'parser-review/probes'/name/'Cargo.toml';case={'name':name,'commands':[]}
  for i,args in enumerate([['cargo','generate-lockfile','--offline','--manifest-path',str(manifest)],['cargo','build','--offline','--locked','--manifest-path',str(manifest)]]):
   r=subprocess.run(args,env=env,cwd=manifest.parent,capture_output=True,timeout=180);(out/f'{name}-{i}-stdout.txt').write_bytes(r.stdout);(out/f'{name}-{i}-stderr.txt').write_bytes(r.stderr);case['commands'].append({'argv':args,'exit_code':r.returncode});assert r.returncode==0,(name,r.stderr.decode()[-2000:])
   if i==0:assert tuples(tomllib.loads(manifest.with_name('Cargo.lock').read_text()))<=known,'new registry tuple'
  argv=[str(b/'probe-target/debug'/binname)];child=subprocess.Popen(argv,stdout=subprocess.PIPE,stderr=subprocess.PIPE,env=env,start_new_session=True,preexec_fn=limits)
  try:stdout,stderr=child.communicate(timeout=30)
  except subprocess.TimeoutExpired:os.killpg(child.pid,signal.SIGKILL);stdout,stderr=child.communicate(timeout=5)
  (out/f'{name}-runtime-stdout.json').write_bytes(stdout);(out/f'{name}-runtime-stderr.txt').write_bytes(stderr);case['commands'].append({'argv':argv,'pid':child.pid,'exit_code':child.returncode});assert child.returncode==0,(name,stderr)
  case['observation']=json.loads(stdout);receipt['cases'].append(case)
 receipt['status']='PASS_NEW_COUNTEREXAMPLE_EXECUTION;_DEFECTS_CONFIRMED_AS_RECORDED'
finally:
 receipt['source_after']={str(p):sha(p) for p in sources};receipt['source_unchanged']=before==receipt['source_after'];(out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({'status':receipt['status'],'observations':[c['observation'] for c in receipt['cases']],'source_unchanged':receipt['source_unchanged']}))
