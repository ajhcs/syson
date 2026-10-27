from pathlib import Path
import json,subprocess,os,fcntl,hashlib,tarfile,io,sys,tomllib
b=Path(__file__).parent;uid,stage=sys.argv[1:3];a=json.loads((b/'batch-admission.json').read_text());u=next(u for u in a['units'] if u['id']==uid);w=Path(u['worktree']);o=b/(uid.lower()+'-red-'+stage);o.mkdir();x=o/'source';x.mkdir();lock=open(a['compiler_lock'],'a');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
with tarfile.open(fileobj=io.BytesIO(subprocess.check_output(['git','archive',u['base']],cwd=w))) as tar:tar.extractall(x,filter='data')
folder={'RR22':'crates/syson-project-preparation','RR23a':'tools/native-graphql-contract','RR23b':'tools/native-db-contract'}[uid];test={'RR22':'rr22_regression','RR23a':'rr23a_literal','RR23b':'rr23b_literal'}[uid];f='rust/'+folder+'/tests/'+test+'.rs';(x/f).write_bytes((w/f).read_bytes());scratch=o/'scratch';scratch.mkdir();env=dict(os.environ,CARGO_BUILD_JOBS='1',CARGO_INCREMENTAL='0',RUST_TEST_THREADS='1',CARGO_TARGET_DIR=str(b/'build/target'),TMPDIR=str(scratch),SYSON_TEST_DIR=str(scratch));r={'unit':uid,'base':u['base'],'configuration':stage,'commands':[],'status':'FAILED_ATTEMPT'}
def run(args,expect=None):
 p=subprocess.run(args,cwd=x,env=env,capture_output=True,timeout=240);i=len(r['commands']);(o/f'{i:02}-stdout.txt').write_bytes(p.stdout);(o/f'{i:02}-stderr.txt').write_bytes(p.stderr);r['commands'].append(dict(argv=args,exit_code=p.returncode));assert expect is None or p.returncode==expect;return p.returncode
try:
 m=x/'rust'/folder/'Cargo.toml'
 if stage=='unified' or uid=='RR22':
  with m.open('a') as h:
   h.write('\nsyson-json-value = { path = "'+('../syson-json-value' if uid=='RR22' else '../../crates/syson-json-value')+'" }\n')
   if uid=='RR22':h.write('\n[dev-dependencies]\nsyson-project-wire = { path = "../syson-project-wire" }\n')
  run(['cargo','generate-lockfile','--offline','--manifest-path',str(m)],0)
 sha=lambda q:hashlib.sha256(q.read_bytes()).hexdigest();files={q for q in (x/'rust').rglob('*') if q.is_file()};r['source_sha256']={str(q.relative_to(x)):sha(q) for q in sorted(files)};r['test_sha256']=sha(x/f)
 code=run(['cargo','test','--offline','--locked','--manifest-path',str(m),'--test',test,'--','--test-threads=1','--nocapture']);expected=0 if uid=='RR23b' and stage=='default' else 101;assert code==expected,'Unexpected baseline outcome; inspect actual output';out=''.join(q.read_text() for q in o.glob('*stdout.txt'));err=''.join(q.read_text() for q in o.glob('*stderr.txt'));assert ('test result: FAILED' in out and 'panicked at' in err) if expected==101 else 'test result: ok' in out
 r['status']='EXPECTED_REACHED_REGRESSION_FAILURE' if expected==101 else 'ORIGINAL_DEFAULT_CONTROLS_PASS;_FEATURE_UNIFICATION_COUNTEREXAMPLE_SEPARATE';assert all(sha(x/rel)==h for rel,h in r['source_sha256'].items())
finally:
 (o/'executed-runner.py').write_bytes(Path(__file__).read_bytes());(o/'receipt.json').write_text(json.dumps(r,indent=2)+'\n')
print(json.dumps({'unit':uid,'configuration':stage,'status':r['status']}))
