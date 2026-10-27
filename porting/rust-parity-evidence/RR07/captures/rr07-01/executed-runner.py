from pathlib import Path
import subprocess,json,hashlib,os,fcntl,tomllib,re,sys
b=Path(__file__).parent;uid,attempt=sys.argv[1:3];mode=sys.argv[3] if len(sys.argv)>3 else 'all';a=json.loads((b/'batch-admission.json').read_text());u=next(v for v in a['units'] if v['id']==uid);w=Path(u['worktree']);out=b/attempt;out.mkdir(exist_ok=False);scratch=out/'scratch';scratch.mkdir();target=b/'build/target';target.mkdir(parents=True,exist_ok=True)
crates={'RR02':['crates/syson-model','crates/syson-json-interchange'],'RR03':['tools/native-library-catalog'],'RR07':['crates/syson-ast-adapter','crates/syson-text-export']}[uid];manifests=[w/'rust'/c/'Cargo.toml' for c in crates]
lock=open(a['compiler_lock'],'a');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB);env=dict(os.environ,CARGO_BUILD_JOBS='1',CARGO_INCREMENTAL='0',RUST_TEST_THREADS='1',CARGO_TARGET_DIR=str(target),TMPDIR=str(scratch),SYSON_LIBRARY_CATALOG_TEST_ROOT=str(scratch));sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();receipt={'unit':uid,'base':u['base'],'status':'FAILED_ATTEMPT','commands':[],'review':'Root reviewed changed source and fixed regressions; only local Rust APIs/owned temporary files. No native/browser/DB execution in these checks.','feature_scope':'Default features; held raw-factory candidate validation is not activated.'};(out/'executed-runner.py').write_bytes(Path(__file__).read_bytes())
def command(args,timeout=240):
 i=len(receipt['commands']);r=subprocess.run(args,cwd=w,env=env,capture_output=True,timeout=timeout);(out/f'{i:02}-stdout.txt').write_bytes(r.stdout);(out/f'{i:02}-stderr.txt').write_bytes(r.stderr);receipt['commands'].append({'argv':args,'exit_code':r.returncode});assert r.returncode==0,(args,r.stderr.decode()[-1600:]);return r
try:
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=w,text=True).strip()==u['base']
 known=tomllib.loads(Path('/workspace/worktrees/builderbob/syson/rp-c03b-project-preparation/rust/crates/syson-project-preparation/Cargo.lock').read_text());tuples=lambda x:{(p['name'],p['version'],p.get('checksum')) for p in x['package'] if p.get('source','').startswith('registry+')}
 for m in manifests:
  command(['cargo','fmt','--manifest-path',str(m)])
  command(['cargo','generate-lockfile','--offline','--manifest-path',str(m)])
  actual=tuples(tomllib.loads(m.with_name('Cargo.lock').read_text()));assert actual<=tuples(known),'New registry tuple'
 files={p for p in (w/'rust').rglob('*') if p.is_file() and 'target' not in p.parts}
 for p in list(files):
  if p.suffix=='.rs':
   for rel in re.findall(r'include(?:_str|_bytes)?!\(\s*"([^"\n]+)"',p.read_text()):
    f=(p.parent/rel).resolve()
    if f.is_file():files.add(f)
 before={str(p.relative_to(w)):sha(p) for p in sorted(files)};receipt['source_before']=before;receipt['registry_tuples']=sorted(set().union(*(tuples(tomllib.loads(m.with_name('Cargo.lock').read_text())) for m in manifests)))
 for m in manifests:
  command(['cargo','fmt','--manifest-path',str(m),'--','--check'])
  if mode=='catalog-remainder':
   assert uid=='RR03'
   previous=json.loads((b/'rr03-02/receipt.json').read_text());assert before==previous['source_after'],'Source changed since prior eight passes'
   receipt['continuation']='rr03-02: eight existing tests passed; CLI test lacked owned-root env, new regression executable never ran. Only failed/unexecuted remainder is run on identical source.'
   command(['cargo','test','--offline','--locked','--manifest-path',str(m),'--test','catalog','actual_cli_exports_real_lookup_rows_and_refuses_filesystem_and_late_failure','--','--exact','--test-threads=1'])
   command(['cargo','test','--offline','--locked','--manifest-path',str(m),'--test','rr03_regression','--','--test-threads=1'])
  else:
   command(['cargo','test','--offline','--locked','--manifest-path',str(m),'--','--test-threads=1'])
  command(['cargo','clippy','--offline','--locked','--manifest-path',str(m),'--all-targets','--','-D','warnings'])
 if uid=='RR03':command(['cargo','build','--offline','--locked','--release','--manifest-path',str(manifests[0])])
 receipt['source_after']={str(p.relative_to(w)):sha(p) for p in sorted(files)};receipt['source_unchanged']=before==receipt['source_after'];assert receipt['source_unchanged']
 paths=set(subprocess.check_output(['git','diff','--name-only'],cwd=w,text=True).splitlines()+subprocess.check_output(['git','ls-files','--others','--exclude-standard'],cwd=w,text=True).splitlines());scopes=u['write_scope']+a['root_coordinator_additional_paths'].get(uid,[])
 for path in paths:assert any(path==s.rstrip('/') or (s.endswith('/') and path.startswith(s)) for s in scopes),('Unleased path',path)
 receipt['changed_paths']=sorted(paths);receipt['status']='PASS_BOUNDED_FINAL_SOURCE_LOCAL_CHECKS;_FEATURE_ACCEPTANCE_OPEN'
finally:
 if 'source_before' in receipt and 'source_after' not in receipt:
  receipt['source_after']={str(p.relative_to(w)):sha(p) for p in sorted(files)};receipt['source_unchanged']=receipt['source_before']==receipt['source_after']
 (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({'unit':uid,'status':receipt['status'],'commands':len(receipt['commands']),'source_files':len(receipt.get('source_before',{})),'source_unchanged':receipt.get('source_unchanged')}))
