from pathlib import Path
import json,hashlib,subprocess,os,fcntl,tomllib,re,sys
b=Path(__file__).parent;uid,attempt=sys.argv[1:3];a=json.loads((b/'batch-admission.json').read_text());u=next(u for u in a['units'] if u['id']==uid);w=Path(u['worktree']);o=b/attempt;o.mkdir(exist_ok=False);scratch=o/'scratch';scratch.mkdir();target=b/'build/target';target.mkdir(parents=True,exist_ok=True);spec={'RR22':['crates/syson-migrations','crates/syson-migration-pipeline','crates/syson-project-preparation','crates/syson-project-wire'],'RR23a':['tools/native-graphql-contract'],'RR23b':['tools/native-db-contract']};manifests=[w/'rust'/c/'Cargo.toml' for c in spec[uid]]
lock=open(a['compiler_lock'],'a');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB);env=dict(os.environ,CARGO_BUILD_JOBS='1',CARGO_INCREMENTAL='0',RUST_TEST_THREADS='1',CARGO_TARGET_DIR=str(target),TMPDIR=str(scratch),SYSON_COMPOSITION_TEST_ROOT=str(scratch),SYSON_TEST_DIR=str(scratch));sha=lambda f:hashlib.sha256(f.read_bytes()).hexdigest();r={'unit':uid,'base':u['base'],'status':'FAILED_ATTEMPT','commands':[],'source_review':'Root reviews fixed code and regression actions before execution; defaultfeatures only, owned workspace temp files/SQLite/Git/FIFO fixtures. No held raw-factory executor, service or native/browser epoch.'};(o/'executed-runner.py').write_bytes(Path(__file__).read_bytes())
def command(args,timeout=240):
 i=len(r['commands']);p=subprocess.run(args,cwd=w,env=env,capture_output=True,timeout=timeout);(o/f'{i:02}-stdout.txt').write_bytes(p.stdout);(o/f'{i:02}-stderr.txt').write_bytes(p.stderr);r['commands'].append({'argv':args,'exit_code':p.returncode});print(uid, args[0:2], p.returncode, flush=True);assert p.returncode==0,(args,p.stderr.decode()[-1600:]);return p
try:
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=w,text=True).strip()==u['base']
 paths=set(subprocess.check_output(['git','diff','--name-only'],cwd=w,text=True).splitlines()+subprocess.check_output(['git','ls-files','--others','--exclude-standard'],cwd=w,text=True).splitlines());scopes=u['write_scope']+a['root_coordinator_additional_paths'][uid]
 for path in paths:assert any(path==s.rstrip('/') or(s.endswith('/') and path.startswith(s)) for s in scopes),('Out of scope BEFORE formatter',path)
 rust=[str(w/p) for p in sorted(paths) if p.endswith('.rs')];command(['rustfmt','--edition','2024','--config','skip_children=true',*rust])
 known=tomllib.loads(Path('/workspace/worktrees/builderbob/syson/rp-c03b-project-preparation/rust/crates/syson-project-preparation/Cargo.lock').read_text());tuples=lambda x:{(p['name'],p['version'],p.get('checksum')) for p in x['package'] if p.get('source','').startswith('registry+')};registry=set()
 for m in manifests:
  actual=tuples(tomllib.loads(m.with_name('Cargo.lock').read_text()));assert actual<=tuples(known);registry.update(actual)
 files={p for p in (w/'rust').rglob('*') if p.is_file() and 'target' not in p.parts}
 for p in list(files):
  if p.suffix=='.rs':
   for rel in re.findall(r'include(?:_str|_bytes)?!\(\s*"([^"\n]+)"',p.read_text()):
    f=(p.parent/rel).resolve()
    if f.is_file():files.add(f)
 before={str(p.relative_to(w)):sha(p) for p in sorted(files)};r['source_before']=before;r['registry_tuples']=sorted(registry);command(['rustfmt','--edition','2024','--config','skip_children=true','--check',*rust])
 for m in manifests:
  command(['cargo','test','--offline','--locked','--manifest-path',str(m),'--','--test-threads=1','--nocapture'])
  command(['cargo','clippy','--offline','--locked','--manifest-path',str(m),'--all-targets','--','-D','warnings'])

 if uid=='RR22':
  m=w/'rust/crates/syson-migrations/Cargo.toml'
  for case in ['rr22_semantic_preparation_wire_literal_gate','rr22_representation_preparation_wire_literal_gate']:
   command(['cargo','test','--offline','--locked','--manifest-path',str(m),'--test','rr24_literal',case,'--','--ignored','--exact','--test-threads=1','--nocapture'])
 else:
  command(['cargo','build','--offline','--locked','--manifest-path',str(manifests[0])])

 r['source_after']={str(p.relative_to(w)):sha(p) for p in sorted(files)};r['source_unchanged']=before==r['source_after'];assert r['source_unchanged']
 after=set(subprocess.check_output(['git','diff','--name-only'],cwd=w,text=True).splitlines()+subprocess.check_output(['git','ls-files','--others','--exclude-standard'],cwd=w,text=True).splitlines());assert after==paths
 r['changed_paths']=sorted(paths);r['owned_runtime_artifacts']={str(f.relative_to(o)):sha(f) for f in scratch.rglob('*') if f.is_file() and '.git' not in f.parts};r['status']='PASS_BOUNDED_FINAL_SOURCE_LOCAL_CHECKS;_FEATURE_ACCEPTANCE_OPEN'
finally:
 if 'source_before' in r and 'source_after' not in r:r['source_after']={str(p.relative_to(w)):sha(p) for p in sorted(files)};r['source_unchanged']=r['source_before']==r['source_after']
 (o/'receipt.json').write_text(json.dumps(r,indent=2)+'\n')
print(json.dumps({'unit':uid,'status':r['status'],'commands':len(r['commands']),'source_files':len(r.get('source_before',{})),'source_unchanged':r.get('source_unchanged')}))
