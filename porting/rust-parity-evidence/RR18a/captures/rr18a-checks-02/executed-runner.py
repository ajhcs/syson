from pathlib import Path
import json,sys,os,subprocess,hashlib,fcntl,time,signal,tomllib
b=Path('/workspace/build/builderbob/syson-rereview-batch9-20261027');uid,attempt=sys.argv[1:3];a=json.loads((b/'batch-admission.json').read_text());u=next((x for x in a['units'] if x['id']==uid),{'id':'BASE','worktree':'/workspace/worktrees/builderbob/syson/rr-batch9-base','base':a['units'][0]['base'],'write_scope':[]});w=Path(u['worktree']);o=b/attempt;o.mkdir();scratch=o/'scratch';scratch.mkdir();target=b/(uid.lower()+'-target');target.mkdir(exist_ok=True);r={'retained_prior_library_passes':'rr18a-checks-01 library48cases;librarysourceunchanged;onlytesthelperlintfix;affected5integrationrequalifiedhere','unit':uid,'status':'FAILED_ATTEMPT','commands':[],'base':u['base'],'held_features':'raw-factory-candidate never selected; no graph/factory execution'};(o/'executed-runner.py').write_bytes(Path(__file__).read_bytes());lock=open(a['compiler_lock'],'a');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
env=dict(os.environ,CARGO_BUILD_JOBS='1',CARGO_INCREMENTAL='0',RUST_TEST_THREADS='1',CARGO_TARGET_DIR=str(target),TMPDIR=str(scratch));sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def cmd(argv,timeout=300):
 t=time.monotonic();p=subprocess.Popen(argv,cwd=w,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True);timed=False
 try:out,err=p.communicate(timeout=timeout)
 except subprocess.TimeoutExpired:
  timed=True;os.killpg(p.pid,signal.SIGKILL);out,err=p.communicate()
 n=len(r['commands']);(o/f'{n:02}-stdout.txt').write_bytes(out);(o/f'{n:02}-stderr.txt').write_bytes(err);r['commands'].append(dict(argv=argv,exit_code=p.returncode,timed_out=timed,wall_seconds=time.monotonic()-t));assert p.returncode==0 and not timed,(argv,err.decode()[-2500:])
try:
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=w,text=True).strip()==u['base']
 changed=set(subprocess.check_output(['git','diff','--name-only'],cwd=w,text=True).splitlines()+subprocess.check_output(['git','ls-files','--others','--exclude-standard'],cwd=w,text=True).splitlines());scopes=u['write_scope']+a['root_coordinator_additional_paths'].get(uid,[])
 for p in changed:assert any(p==s.rstrip('/') or (s.endswith('/') and p.startswith(s)) for s in scopes),('unleased',p)
 rust=[str(w/p) for p in sorted(changed) if p.endswith('.rs')]
 if rust:cmd(['rustfmt','--edition','2024','--config','skip_children=true',*rust])
 paths=set(subprocess.check_output(['git','ls-files'],cwd=w,text=True).splitlines())|changed;files=[w/p for p in sorted(paths) if (w/p).is_file() and not p.startswith('porting/rust-parity-evidence/'+uid+'/captures/')];before={str(p.relative_to(w)):sha(p) for p in files};r['source_before']=before
 if rust:cmd(['rustfmt','--edition','2024','--config','skip_children=true','--check',*rust])
 if uid!='BASE':
  crates=['syson-model','syson-web'] if uid=='RR20' else ['syson-web']
  for c in crates:
   m=str(w/'rust/crates'/c/'Cargo.toml');cmd(['cargo','test','--offline','--locked','--jobs','1','--manifest-path',m,'--test','rr18_regression','--','--test-threads=1','--nocapture']);cmd(['cargo','clippy','--offline','--locked','--jobs','1','--manifest-path',m,'--all-targets','--','-D','warnings'])
 cmd(['cargo','build','--release','--offline','--locked','--jobs','1','--target','wasm32-unknown-unknown','--manifest-path',str(w/'rust/crates/syson-web/Cargo.toml')])
 wasm=target/'wasm32-unknown-unknown/release/syson_web.wasm';(o/'syson_web.wasm').write_bytes(wasm.read_bytes());r['wasm_sha256']=sha(wasm);r['source_after']={str(p.relative_to(w)):sha(p) for p in files};assert before==r['source_after'];r['status']='PASS_BOUNDED_DEFAULT_SOURCE_TESTS_BUILD;FULL_ACCEPTANCE_OPEN';r['target']=str(target)
finally:
 r['source_after']={str(p.relative_to(w)):sha(p) for p in files} if 'files' in locals() else {};r['source_unchanged']=r.get('source_before')==r['source_after'];(o/'receipt.json').write_text(json.dumps(r,indent=2)+'\n')
print(json.dumps({k:v for k,v in r.items() if k not in ['source_before','source_after']},indent=2))
