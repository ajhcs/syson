"""Coordinator-only same-input Rust observation of the B25d module."""
from pathlib import Path
import hashlib,json,subprocess,os,fcntl,tomllib
root=Path('/workspace/build/builderbob/syson-rust-parity-batch3-20261026');out=root/'action-probe';(out/'src').mkdir(parents=True);(out/'tmp').mkdir()
source=Path(__file__).with_name('action_observe.rs');(out/'src/main.rs').write_bytes(source.read_bytes())
(out/'Cargo.toml').write_text('''[package]
name="syson-action-observe"
version="0.1.0"
edition="2024"
publish=false
[workspace]
[dependencies]
serde_json={version="=1.0.151",features=["arbitrary_precision"]}
syson-migrations={path="/workspace/worktrees/builderbob/syson/rp-b25d/rust/crates/syson-migrations"}
''')
(out/'executed-runner.py').write_bytes(Path(__file__).read_bytes())
env=os.environ.copy();env.update(CARGO_BUILD_JOBS='1',CARGO_INCREMENTAL='0',TMPDIR=str(out/'tmp'),CARGO_TARGET_DIR='/workspace/build/builderbob/syson-audit-20261026/build/target')
lock=Path('/workspace/build/builderbob/syson-audit-20261026/build/compiler.lock').open('a');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
setup=subprocess.run(['cargo','generate-lockfile','--offline','--manifest-path',str(out/'Cargo.toml')],cwd=out,env=env,capture_output=True,timeout=30)
(out/'setup.stdout.txt').write_bytes(setup.stdout);(out/'setup.stderr.txt').write_bytes(setup.stderr);assert setup.returncode==0
allowed={(p['name'],p['version'],p.get('checksum')) for p in tomllib.loads(Path('/workspace/worktrees/builderbob/syson/rp-b25d/rust/crates/syson-migrations/Cargo.lock').read_text())['package'] if 'source' in p}
actual={(p['name'],p['version'],p.get('checksum')) for p in tomllib.loads((out/'Cargo.lock').read_text())['package'] if 'source' in p};assert actual<=allowed
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
wt=Path('/workspace/worktrees/builderbob/syson/rp-b25d');inputs={str(wt/f):h for f,h in json.loads((root/'captures/b25d-01/receipt.json').read_text())['source_after'].items()}
for p in [out/'src/main.rs',out/'Cargo.toml',out/'Cargo.lock']:inputs[str(p)]=sha(p)
assert all(sha(Path(f))==h for f,h in inputs.items())
packet=wt/'porting/rust-parity-evidence/B25d/native-fixtures.json';argv=['cargo','run','--offline','--locked','--manifest-path',str(out/'Cargo.toml'),'--',str(packet)]
run=subprocess.run(argv,cwd=out,env=env,capture_output=True,timeout=180);(out/'stdout.json').write_bytes(run.stdout);(out/'stderr.txt').write_bytes(run.stderr)
receipt={'argv':argv,'exit_code':run.returncode,'source_before':inputs,'source_unchanged':all(sha(Path(f))==h for f,h in inputs.items()),'runner_sha256':sha(Path(__file__)),'stdout_sha256':hashlib.sha256(run.stdout).hexdigest(),'stderr_sha256':hashlib.sha256(run.stderr).hexdigest()}
(out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n');assert run.returncode==0 and receipt['source_unchanged'],run.stderr
native=json.loads((root/'native/run-01/stdout.json').read_text());rust=json.loads(run.stdout);assert len(native['results'])==len(rust['results'])==19
rows=[]
for a,b in zip(native['results'],rust['results']):
 for key in ['id','version','kind','status']:assert a[key]==b[key],(key,a,b)
 if a['status']=='PREPARED':assert a['json']==b['json'] and a['changed']==b['changed'],(a,b)
 rows.append({'id':a['id'],'match':True,'status':a['status'],'native_partial_tree_retained':a['status']=='REFUSED' and 'native_partial_tree'in a})
(out/'comparison.json').write_text(json.dumps({'status':'PASS_19_SELECTED_CALLBACK_TRAVERSAL_CASES','rows':rows,'scope':'Refused outcomes compared; Java partial tree retained, Rust atomic publication deliberate; no wholeartifact/framework/app/DB acceptance'},indent=2)+'\n')
print('PASS19native/Rustcallback-traversalcases')
