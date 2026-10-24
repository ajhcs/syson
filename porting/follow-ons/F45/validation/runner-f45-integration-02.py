#!/usr/bin/env python3
"""Capture F45's actual selected-model file roundtrip without changing dependencies."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
from datetime import datetime, timezone
import tomllib

root=Path('/workspace/worktrees/builderbob/syson/f45-identities')
harness=root/'porting/domain/identity-context/integration'
manifest=harness/'Cargo.toml'
data=tomllib.loads(manifest.read_text())
expected={name:{'path':'../../../../rust/crates/'+name} for name in ['syson-domain','syson-model','syson-workflow']}
if data['dependencies']!=expected or data.get('build-dependencies') or (harness/'build.rs').exists():
    raise SystemExit('Unreviewed integration dependency/build input')
def git(*args):
    return subprocess.check_output(['git','-C',str(root),*args])
old=tomllib.loads(git('show','ad814c232:rust/crates/syson-workflow/Cargo.lock').decode())['package']
new=tomllib.loads((harness/'Cargo.lock').read_text())['package']
expected_old=[p.copy() for p in old if p['name'] not in {'syson-initializer','syson-initializer-graph'}]
for record in expected_old:
    if record['name']=='syson-workflow':
        record['dependencies']=['syson-editor','syson-model','syson-persistence']
if [p for p in new if p['name']!='syson-identity-context-integration']!=expected_old:
    raise SystemExit('Existing dependency lock records changed')
directories=[harness]+[root/'rust/crates'/name for name in ['syson-domain','syson-model','syson-metamodel','syson-workflow','syson-editor','syson-persistence']]
def sha(raw):
    return hashlib.sha256(raw).hexdigest()
def freeze():
    return {str(p.relative_to(root)):sha(p.read_bytes()) for directory in directories for p in sorted(directory.rglob('*')) if p.is_file() and 'target' not in p.relative_to(directory).parts}
run=sys.argv[1]
if run!='f45-integration-02':
    raise SystemExit('Unexpected integration capture ID')
out=root/'target/follow-on-checks'/run
out.mkdir(parents=True,exist_ok=False)
scratch=root/'target/follow-on-tmp'
scratch.mkdir(parents=True,exist_ok=True)
env=os.environ.copy()
env.update(CARGO_BUILD_JOBS='1',RUST_TEST_THREADS='1',CARGO_TARGET_DIR=str(root/'target/follow-on-build'),TMPDIR=str(scratch),SYSON_TEST_DIR=str(scratch))
result={'source_base':git('rev-parse','HEAD').decode().strip(),'source_before':freeze(),'runner_sha256':sha(Path(__file__).read_bytes()),'commands':[],'started_utc':datetime.now(timezone.utc).isoformat(),'upstream_runtime':'NOT_RUN','test_kind':'actual selected JSON Document/Editor/SnapshotStore save and fresh-store file reload'}
commands=[['cargo','fmt','--manifest-path',str(manifest),'--','--check'],['cargo','test','--offline','--locked','--manifest-path',str(manifest),'--','--test-threads=1'],['cargo','clippy','--offline','--locked','--manifest-path',str(manifest),'--all-targets','--','-D','warnings']]
for i,argv in enumerate(commands):
    entry={'argv':argv,'started_utc':datetime.now(timezone.utc).isoformat()}
    child=subprocess.run(argv,cwd=root,env=env,capture_output=True,timeout=120)
    (out/f'{i:02d}-stdout.txt').write_bytes(child.stdout)
    (out/f'{i:02d}-stderr.txt').write_bytes(child.stderr)
    entry.update(completed_utc=datetime.now(timezone.utc).isoformat(),exit_code=child.returncode,stdout_sha256=sha(child.stdout),stderr_sha256=sha(child.stderr))
    result['commands'].append(entry)
    if child.returncode:
        result['status']='FAIL'
        break
else:
    result['status']='PASS'
result.update(source_after=freeze(),completed_utc=datetime.now(timezone.utc).isoformat())
result['source_unchanged']=result['source_before']==result['source_after']
if not result['source_unchanged']:
    result['status']='SOURCE_DRIFT'
(out/'receipt.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({'status':result['status'],'receipt':str(out/'receipt.json'),'commands':[r['exit_code'] for r in result['commands']],'source_unchanged':result['source_unchanged']}))
raise SystemExit(0 if result['status']=='PASS' else 1)
