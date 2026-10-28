from pathlib import Path
import subprocess,json,hashlib,os,fcntl,tomllib,re,sys
b=Path(__file__).parent;uid,attempt=sys.argv[1:3];mode=sys.argv[3] if len(sys.argv)>3 else 'all';admission=json.loads((b/'batch-admission.json').read_text());unit=next(u for u in admission['units'] if u['id']==uid);wt=Path(unit['worktree']);crate={'RR01':'syson-json-value','RR06':'syson-text-export','RR11':'syson-storage'}[uid];manifest=wt/'rust/crates'/crate/'Cargo.toml';out=b/attempt;out.mkdir(exist_ok=False);scratch=out/'scratch';scratch.mkdir();target=b/'build/target';target.mkdir(parents=True,exist_ok=True)
lock=open(admission['compiler_lock'],'a');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
env=dict(os.environ,CARGO_BUILD_JOBS='1',CARGO_INCREMENTAL='0',RUST_TEST_THREADS='1',CARGO_TARGET_DIR=str(target),TMPDIR=str(scratch));sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();receipt={'unit':uid,'status':'FAILED_ATTEMPT','base':unit['base'],'commands':[],'resource_policy':'One offline compiler job; tests serial, owned workspace TMPDIR; no native/browser/production DB/network execution','root_source_review':'Frozen author source read; dev allocator or fixed Python bridge reviewed separately where applicable'}
(out/'executed-runner.py').write_bytes(Path(__file__).read_bytes())
def command(argv,timeout=180):
 index=len(receipt['commands']);r=subprocess.run(argv,cwd=wt,env=env,capture_output=True,timeout=timeout);(out/f'{index:02}-stdout.txt').write_bytes(r.stdout);(out/f'{index:02}-stderr.txt').write_bytes(r.stderr);receipt['commands'].append({'argv':argv,'exit_code':r.returncode});assert r.returncode==0,(argv,r.stderr.decode()[-1800:]);return r
try:
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=wt,text=True).strip()==unit['base']
 command(['cargo','fmt','--manifest-path',str(manifest)])
 if uid=='RR01':command(['cargo','generate-lockfile','--offline','--manifest-path',str(manifest)])
 known=tomllib.loads(Path('/workspace/worktrees/builderbob/syson/rp-c03b-project-preparation/rust/crates/syson-project-preparation/Cargo.lock').read_text());tuples=lambda x:{(p['name'],p['version'],p.get('checksum')) for p in x['package'] if p.get('source','').startswith('registry+')};actual=tuples(tomllib.loads(manifest.with_name('Cargo.lock').read_text()));assert actual<=tuples(known),'Unreviewed registry tuples';receipt['registry_tuples']=sorted(actual)
 files={p for p in (wt/'rust').rglob('*') if p.is_file() and 'target' not in p.parts}
 for p in list(files):
  if p.suffix=='.rs':
   for rel in re.findall(r'include(?:_str|_bytes)?!\(\s*"([^"\n]+)"',p.read_text()):
    f=(p.parent/rel).resolve()
    if f.is_file():files.add(f)
 before={str(p.relative_to(wt)):sha(p) for p in sorted(files)};receipt['source_before']=before
 command(['cargo','fmt','--manifest-path',str(manifest),'--','--check'])
 if mode=='regression-only':
  assert uid=='RR06'
  previous=json.loads((b/'rr06-01/receipt.json').read_text())
  changed=[path for path,digest in before.items() if previous['source_after'].get(path)!=digest]
  assert changed==['rust/crates/syson-text-export/tests/rr06_regression.rs'],changed
  receipt['continuation']={'original_attempt':'rr06-01','only_changed_source':changed,'unchanged_production_and_existing_tests':'Original 13 existing tests passed on identical production/test bytes; only the changed regression executable is rerun.'}
  command(['cargo','test','--offline','--locked','--manifest-path',str(manifest),'--test','rr06_regression','--','--test-threads=1'],240)
 else:
  command(['cargo','test','--offline','--locked','--manifest-path',str(manifest),'--','--test-threads=1'],240)
 command(['cargo','clippy','--offline','--locked','--manifest-path',str(manifest),'--all-targets','--','-D','warnings'],180)
 receipt['source_after']={str(p.relative_to(wt)):sha(p) for p in sorted(files)};receipt['source_unchanged']=receipt['source_after']==before;assert receipt['source_unchanged'],'Source changed during checks'
 paths=set(subprocess.check_output(['git','diff','--name-only'],cwd=wt,text=True).splitlines()+subprocess.check_output(['git','ls-files','--others','--exclude-standard'],cwd=wt,text=True).splitlines())
 for path in paths:
  assert any(path==scope.rstrip('/') or (scope.endswith('/') and path.startswith(scope)) for scope in unit['write_scope']),('Unleased changed path',path)
 receipt['changed_paths']=sorted(paths);receipt['owned_runtime_artifacts']={str(f.relative_to(out)):sha(f) for f in scratch.rglob('*') if f.is_file()};receipt['status']='PASS_BOUNDED_LOCAL_FINAL_SOURCE_CHECKS;_FEATURE_ACCEPTANCE_OPEN'
finally:
 if 'source_before' in receipt and 'source_after' not in receipt:
  receipt['source_after']={str(p.relative_to(wt)):sha(p) for p in sorted(files)};receipt['source_unchanged']=receipt['source_after']==receipt['source_before']
 (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({'unit':uid,'status':receipt['status'],'source_unchanged':receipt.get('source_unchanged'),'commands':len(receipt['commands']),'source_files':len(receipt.get('source_before',{}))}))
