#!/usr/bin/env python3
"""Run only this Rust milestone's offline checks; capture UTC and source hashes."""
import argparse,datetime,hashlib,json,os,pathlib,subprocess
ROOT=pathlib.Path(__file__).resolve().parents[2]
p=argparse.ArgumentParser();p.add_argument('--target-dir',type=pathlib.Path,required=True);p.add_argument('--evidence-dir',type=pathlib.Path,required=True);a=p.parse_args()
a.evidence_dir.mkdir(parents=True,exist_ok=False)
def now():return datetime.datetime.now(datetime.timezone.utc).isoformat().replace('+00:00','Z')
def digest(b):return hashlib.sha256(b).hexdigest()
paths=sorted([p for p in (ROOT/'rust/crates/syson-syntax').rglob('*') if p.is_file()]+list((ROOT/'porting/syntax').glob('*.py')))
hashes={str(p.relative_to(ROOT)):digest(p.read_bytes()) for p in paths}
env=dict(os.environ,CARGO_BUILD_JOBS='1',CARGO_TARGET_DIR=str(a.target_dir));manifest='rust/crates/syson-syntax/Cargo.toml'
commands=[['python3','porting/syntax/generate_data.py','--check'],['cargo','fmt','--check','--manifest-path',manifest],['cargo','test','--offline','--locked','--manifest-path',manifest],['cargo','clippy','--offline','--locked','--manifest-path',manifest,'--all-targets','--','-D','warnings'],['cargo','check','--offline','--locked','--target','wasm32-unknown-unknown','--manifest-path',manifest]]
report={'started_at_utc':now(),'source_hashes':hashes,'upstream_runtime_execution':'NOT RUN','build_jobs':1,'target_directory':'${SYSON_BUILD_DIR}/syntax','commands':[]}
failed=False
for i,command in enumerate(commands):
 start=now();result=subprocess.run(command,cwd=ROOT,env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT);end=now();name=f'{i+1:02d}-{command[0]}-{pathlib.Path(command[1]).name}.log';(a.evidence_dir/name).write_bytes(result.stdout)
 report['commands'].append({'argv':command,'started_at_utc':start,'ended_at_utc':end,'exit_code':result.returncode,'output_file':name,'output_sha256':digest(result.stdout),'test_result_lines':[l for l in result.stdout.decode().splitlines() if l.startswith('test result:')]})
 print(command[0],command[1],result.returncode)
 if result.returncode: print(result.stdout.decode());failed=True;break
report['ended_at_utc']=now();report['status']='FAIL' if failed else 'PASS';text=json.dumps(report,indent=2)+'\n';(a.evidence_dir/'receipt.json').write_text(text);(ROOT/'porting/syntax/VALIDATION.json').write_text(text)
print('UTC',report['started_at_utc'],report['ended_at_utc'],report['status'])
raise SystemExit(1 if failed else 0)
