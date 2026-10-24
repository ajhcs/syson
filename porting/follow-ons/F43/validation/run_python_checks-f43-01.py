#!/usr/bin/env python3
"""Capture one owned Python unittest run with workspace-only synthetic state."""
import hashlib
import json
import os
import pathlib
import subprocess
import sys
from datetime import datetime, timezone

root=pathlib.Path(sys.argv[1]).resolve()
assert str(root).startswith('/workspace/worktrees/builderbob/syson/')
run=sys.argv[2]
assert run=='f43-01'
out=root/'target/follow-on-checks'/run
out.mkdir(parents=True,exist_ok=False)
scratch=root/'target/follow-on-tmp'
scratch.mkdir(parents=True,exist_ok=True)
inputs=[root/'porting/distribution-check',root/'porting/follow-ons/F43']
def digest(raw):
    return hashlib.sha256(raw).hexdigest()
def freeze():
    return {str(p.relative_to(root)):digest(p.read_bytes()) for folder in inputs for p in sorted(folder.rglob('*')) if p.is_file() and '__pycache__' not in p.parts}
env=os.environ.copy()
env.update(PYTHONDONTWRITEBYTECODE='1',TMPDIR=str(scratch),SYSON_TEST_DIR=str(scratch),SYSON_REGISTRY_CACHE='/workspace/cache/builderbob/global-tools/cargo/registry/cache')
argv=['python3','-B','-m','unittest','discover','-s','porting/distribution-check','-p','test_check.py','-v']
result={'argv':argv,'started_utc':datetime.now(timezone.utc).isoformat(),'source_before':freeze(),'runner_sha256':digest(pathlib.Path(__file__).read_bytes()),'python_version':sys.version,'upstream_runtime':'NOT_RUN','built_WASM':'NOT_RUN','synthetic_packages':True}
child=subprocess.run(argv,cwd=root,env=env,capture_output=True,timeout=120)
(out/'stdout.txt').write_bytes(child.stdout)
(out/'stderr.txt').write_bytes(child.stderr)
result.update(completed_utc=datetime.now(timezone.utc).isoformat(),exit_code=child.returncode,stdout_sha256=digest(child.stdout),stderr_sha256=digest(child.stderr),source_after=freeze())
result['source_unchanged']=result['source_before']==result['source_after']
result['status']='PASS' if child.returncode==0 and result['source_unchanged'] else 'FAIL'
(out/'receipt.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({'status':result['status'],'receipt':str(out/'receipt.json'),'source_unchanged':result['source_unchanged']}))
raise SystemExit(0 if result['status']=='PASS' else 1)
