from pathlib import Path
import subprocess,hashlib,json,fcntl,os,signal,time
b=Path(__file__).parent;o=b/'transport-baseline-red-01';o.mkdir();c=o/'consumer';(c/'tests').mkdir(parents=True);src=Path('/workspace/worktrees/builderbob/syson/rr18b1-canvas-transport/rust/crates/syson-web/tests/rr18b1_regression.rs');data=src.read_bytes();assert hashlib.sha256(data).hexdigest()=='db87547c764eac83629ad6b67dd271bf959320dcc9e4a8a8fd6cce89eacc8e08';(c/'tests/transport.rs').write_bytes(data);base=Path('/workspace/worktrees/builderbob/syson/rr18b-baseline');(c/'Cargo.toml').write_text(f'[package]\nname="rr18b1-baseline-red"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[dependencies]\nsyson-web={{path="{base}/rust/crates/syson-web"}}\nsyson-model={{path="{base}/rust/crates/syson-model"}}\nserde_json="=1.0.151"\n');(o/'executed-runner.py').write_bytes(Path(__file__).read_bytes());r={'status':'FAILED_SETUP','test_sha256':hashlib.sha256(data).hexdigest(),'commands':[]};lock=open('/workspace/build/builderbob/syson-audit-20261026/build/compiler.lock','a');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB);env=dict(os.environ,CARGO_BUILD_JOBS='1',CARGO_INCREMENTAL='0',CARGO_TARGET_DIR=str(b/'baseline-red-target'),TMPDIR=str(o));
try:
 for n,argv in enumerate([['cargo','generate-lockfile','--offline','--manifest-path',str(c/'Cargo.toml')],['cargo','test','--offline','--locked','--jobs','1','--manifest-path',str(c/'Cargo.toml'),'--','--test-threads=1','--nocapture']]):
  p=subprocess.Popen(argv,cwd=c,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
  try:out,err=p.communicate(timeout=240)
  except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);out,err=p.communicate();raise RuntimeError('Owned baseline group timed out')
  (o/f'{n:02}-stdout.txt').write_bytes(out);(o/f'{n:02}-stderr.txt').write_bytes(err);r['commands'].append({'argv':argv,'exit_code':p.returncode});assert p.returncode==(0 if n==0 else 101),(out,err)
  if n:assert b'panicked at' in err and b'alloc' in err.lower(),err
 r['status']='EXPECTED_REACHED_ALLOCATION_ASSERTION_FAILURE';r['scope']='New regression against immutable #112; not a parity or full-suite result'
finally:(o/'receipt.json').write_text(json.dumps(r,indent=2)+'\n')
print(json.dumps(r,indent=2))
