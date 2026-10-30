"""Coordinator-only new four-case version-ordered migration observations."""
from pathlib import Path
import fcntl, hashlib, json, os, subprocess, tomllib
root=Path('/workspace/build/builderbob/syson-rust-parity-batch2-20261026')
out=root/'sequence-01';out.mkdir(exist_ok=False);tmp=out/'tmp';tmp.mkdir();classes=out/'classes';classes.mkdir()
probe=root/'probe';support=Path('/workspace/worktrees/builderbob/syson/rp-b25-support')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
lock=Path('/workspace/build/builderbob/syson-audit-20261026/build/compiler.lock').open('a');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
env=os.environ.copy();env.update(CARGO_BUILD_JOBS='1',CARGO_INCREMENTAL='0',TMPDIR=str(tmp),CARGO_TARGET_DIR='/workspace/build/builderbob/syson-audit-20261026/build/target')
subprocess.run(['cargo','fmt','--manifest-path',str(probe/'Cargo.toml')],env=env,check=True,capture_output=True)
before=json.loads((root/'captures/support-04/receipt.json').read_text())['source_after']
inputs={str(support/f):h for f,h in before.items()}
extra=[probe/'Cargo.toml',probe/'Cargo.lock',probe/'src/main.rs',root/'sequence-fixtures.json',*(root/'native/src-03').glob('*.java')]
for unit in ['B25a','B25b','B25c']:extra.append(Path('/workspace/worktrees/builderbob/syson/rp-'+unit.lower()+'/rust/crates/syson-migrations/'+unit.lower()+'.rs'))
for f in extra:inputs[str(f)]=sha(f)
for f,h in inputs.items():assert sha(Path(f))==h,f
for f in extra:
 target=out/'inputs'/f.relative_to('/');target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(f.read_bytes())
(out/'executed-runner.py').write_bytes(Path(__file__).read_bytes())
allowed={(x['name'],x['version'],x.get('checksum')) for x in tomllib.loads((support/'rust/crates/syson-migrations/Cargo.lock').read_text())['package'] if 'source'in x}
actual={(x['name'],x['version'],x.get('checksum')) for x in tomllib.loads((probe/'Cargo.lock').read_text())['package'] if 'source'in x};assert actual<=allowed
javac='/workspace/cache/builderbob/global-tools/releases/jdk/26.0.2.1+1/bin/javac';java='/workspace/build/builderbob/ajhcs-sysml-java21-20261004/jdk-21.0.12.1+1-jre/bin/java'
commands=[['cargo','build','--offline','--locked','--manifest-path',str(probe/'Cargo.toml')],
 [javac,'-J-XX:ActiveProcessorCount=2','-J-Xmx256m','-proc:none','--release','21','-cp',str(root/'native/lib/*'),'-d',str(classes),*[str(x) for x in sorted((root/'native/src-03').glob('*.java'))]],
 ['/workspace/build/builderbob/syson-audit-20261026/build/target/debug/syson-migration-observation-probe','sequence',str(root/'sequence-fixtures.json')],
 [java,'-Xmx256m','-XX:ActiveProcessorCount=2','-Djava.io.tmpdir='+str(tmp),'-cp',str(classes)+':'+str(root/'native/lib/*'),'MigrationOracle','sequence',str(root/'sequence-fixtures.json')]]
receipt={'status':'RUNNING','source_before':inputs,'commands':[],'scope':'Fournewpureparticipantcompositioncases; noDB/apphost/nativewholeartifactclosure'}
try:
 for i,argv in enumerate(commands):
  run=subprocess.run(argv,cwd=root,env=env,capture_output=True,timeout=180 if i==0 else 60)
  (out/f'{i:02}-stdout.txt').write_bytes(run.stdout);(out/f'{i:02}-stderr.txt').write_bytes(run.stderr)
  receipt['commands'].append({'argv':argv,'exit_code':run.returncode,'stdout_sha256':hashlib.sha256(run.stdout).hexdigest(),'stderr_sha256':hashlib.sha256(run.stderr).hexdigest()})
  assert run.returncode==0,run.stderr.decode()[-1500:]
 rust=json.loads((out/'02-stdout.txt').read_text());native=json.loads((out/'03-stdout.txt').read_text())
 assert len(rust['results'])==len(native['results'])==4
 for a,b in zip(rust['results'],native['results']):
  for key in ['id','json','matched_resources','versions']:assert a[key]==b[key],(key,a,b)
 receipt['status']='PASS_FOUR_MATCHED_VERSION_ORDERED_COMPOSITIONS'
finally:
 receipt['source_unchanged']=all(sha(Path(f))==h for f,h in inputs.items())
 receipt['class_sha256']={str(p.relative_to(out)):sha(p) for p in classes.rglob('*.class')}
 (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(receipt['status'],receipt['source_unchanged'])
