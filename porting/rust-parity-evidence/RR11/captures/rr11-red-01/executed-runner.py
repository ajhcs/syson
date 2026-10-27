from pathlib import Path
import subprocess,tarfile,io,json,hashlib,os,fcntl,tomllib,resource,signal,shutil
b=Path(__file__).parent;o=b/'rr11-red-01';o.mkdir(exist_ok=False);root=Path('/workspace/repos/builderbob/syson');export=o/'source';export.mkdir()
archive=subprocess.check_output(['git','archive','6dc96e2cb4ef548656f4c9ac3a24f6e37ff9c194','rust'],cwd=root)
with tarfile.open(fileobj=io.BytesIO(archive)) as t:t.extractall(export,filter='data')
test=Path('/workspace/worktrees/builderbob/syson/rr11-db-generation/rust/crates/syson-storage/tests/rr11_regression.rs');assert hashlib.sha256(test.read_bytes()).hexdigest()=='541fbe6d05c63313453940f26170926f2270f34227f77d15cd76d781afba85f8';shutil.copy2(test,export/'rust/crates/syson-storage/tests/rr11_regression.rs')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();files=[f for f in export.rglob('*') if f.is_file()];before={str(f.relative_to(export)):sha(f) for f in files}
known=tomllib.loads(Path('/workspace/worktrees/builderbob/syson/rp-c03b-project-preparation/rust/crates/syson-project-preparation/Cargo.lock').read_text());tuples=lambda x:{(p['name'],p['version'],p.get('checksum')) for p in x['package'] if p.get('source','').startswith('registry+')};assert tuples(tomllib.loads((export/'rust/crates/syson-storage/Cargo.lock').read_text()))<=tuples(known)
lock=open('/workspace/build/builderbob/syson-audit-20261026/build/compiler.lock','a');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
scratch=o/'scratch';scratch.mkdir();target=b/'build/target';target.mkdir(parents=True,exist_ok=True);env=dict(os.environ,CARGO_BUILD_JOBS='1',CARGO_INCREMENTAL='0',RUST_TEST_THREADS='1',CARGO_TARGET_DIR=str(target),TMPDIR=str(scratch))
receipt={'status':'FAILED_ATTEMPT','scope':'Frozen #64 baseline plus source-reviewed new regression; fresh owned SQLite files only','source_before':before,'commands':[],'review':'Root read both Rust regressions and immutable bridge.py; only fixed sqlite operations/owned renames, no packages/network/service/production state'}
(o/'executed-runner.py').write_bytes(Path(__file__).read_bytes())
try:
 args=['cargo','test','--offline','--locked','--manifest-path',str(export/'rust/crates/syson-storage/Cargo.toml'),'--test','rr11_regression','--no-run','--message-format=json'];r=subprocess.run(args,capture_output=True,env=env,cwd=export,timeout=180);(o/'build-stdout.txt').write_bytes(r.stdout);(o/'build-stderr.txt').write_bytes(r.stderr);receipt['commands'].append({'argv':args,'exit_code':r.returncode});assert r.returncode==0,r.stderr.decode()[-1200:]
 messages=[json.loads(s) for s in r.stdout.splitlines() if s.startswith(b'{')];exe=next(x['executable'] for x in messages if x.get('reason')=='compiler-artifact' and x.get('target',{}).get('name')=='rr11_regression' and x.get('executable'))
 def limits():resource.setrlimit(resource.RLIMIT_AS,(512*1024*1024,512*1024*1024));resource.setrlimit(resource.RLIMIT_CPU,(30,30))
 args=[exe,'--test-threads=1','--nocapture'];r=subprocess.run(args,capture_output=True,env=env,cwd=export,timeout=90,preexec_fn=limits);(o/'runtime-stdout.txt').write_bytes(r.stdout);(o/'runtime-stderr.txt').write_bytes(r.stderr);receipt['commands'].append({'argv':args,'executable_sha256':sha(Path(exe)),'exit_code':r.returncode});assert r.returncode==101 and r.stdout.count(b'FAILED')>=2 and b'an observation from A authorized mutation of replacement B' in r.stderr,'Unexpected red result';receipt['status']='EXPECTED_RED_BOTH_REPLACEMENT_MUTATIONS_REPRODUCED'
finally:
 receipt['source_after']={str(f.relative_to(export)):sha(f) for f in files};receipt['source_unchanged']=receipt['source_after']==before;receipt['owned_db_artifacts']={str(f.relative_to(o)):sha(f) for f in scratch.rglob('*') if f.is_file()};(o/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({'status':receipt['status'],'source_unchanged':receipt['source_unchanged'],'owned_db_artifact_count':len(receipt['owned_db_artifacts'])}))
