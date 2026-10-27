from pathlib import Path
import subprocess,tarfile,io,json,hashlib,os,fcntl,tomllib,shutil
b=Path(__file__).parent;root=Path('/workspace/repos/builderbob/syson');o=b/'red-01';o.mkdir(exist_ok=False);lock=open('/workspace/build/builderbob/syson-audit-20261026/build/compiler.lock','a');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB);target=b/'build/target';target.mkdir(parents=True,exist_ok=True)
sha=lambda f:hashlib.sha256(f.read_bytes()).hexdigest();receipt={'status':'FAILED_ATTEMPT','review':'Root read fixed regression sources: normal Rust model/catalogue/importer API only, independent manual/raw oracles, no external side effects','cases':[]};(o/'executed-runner.py').write_bytes(Path(__file__).read_bytes())
specs=[('RR02','f4e951a74c2a1b2d32db4d397ad4bddae3bbd173','crates/syson-model',2),('RR03','df588dc53ca55e3d04464c06dc798e594fdd7d66','tools/native-library-catalog',1),('RR07','0b1ac2d72f131322939227876ab561c5dbef073d','crates/syson-text-export',1)]
try:
 for uid,rev,crate,fails in specs:
  case=o/uid;case.mkdir();export=case/'source';export.mkdir();archive=subprocess.check_output(['git','archive',rev,'rust'],cwd=root)
  with tarfile.open(fileobj=io.BytesIO(archive)) as t:t.extractall(export,filter='data')
  shutil.copy2(b/'red-tests'/(uid.lower()+'_regression.rs'),export/'rust'/crate/'tests'/(uid.lower()+'_regression.rs'))
  files=[f for f in export.rglob('*') if f.is_file()];before={str(f.relative_to(export)):sha(f) for f in files};scratch=case/'scratch';scratch.mkdir();env=dict(os.environ,CARGO_BUILD_JOBS='1',CARGO_INCREMENTAL='0',RUST_TEST_THREADS='1',CARGO_TARGET_DIR=str(target),TMPDIR=str(scratch))
  args=['cargo','test','--offline','--locked','--manifest-path',str(export/'rust'/crate/'Cargo.toml'),'--test',uid.lower()+'_regression','--','--test-threads=1'];r=subprocess.run(args,cwd=export,env=env,capture_output=True,timeout=180);(case/'stdout.txt').write_bytes(r.stdout);(case/'stderr.txt').write_bytes(r.stderr);after={str(f.relative_to(export)):sha(f) for f in files};row={'unit':uid,'baseline':rev,'argv':args,'exit_code':r.returncode,'source_before':before,'source_after':after,'source_unchanged':before==after,'expected_failed_test_count':fails};receipt['cases'].append(row)
  assert before==after;assert r.returncode==101 and f'{fails} failed;'.encode() in r.stdout and b'test result: FAILED' in r.stdout,(uid,r.stderr.decode()[-800:]);row['status']='EXPECTED_RED_REAL_COUNTEREXAMPLE'
 receipt['status']='EXPECTED_RED_ALL_FOUR_COUNTEREXAMPLES_REPRODUCED'
finally:(o/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({'status':receipt['status'],'cases':[{'unit':r['unit'],'status':r.get('status')} for r in receipt['cases']]}))
