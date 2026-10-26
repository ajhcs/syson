#!/usr/bin/env python3
"""Inspect retained reference inputs without running an oracle or application."""
import datetime
import hashlib
import json
import subprocess
import os
import signal
import time
from pathlib import Path

base = Path('/workspace/build/builderbob/syson-rust-parity-batch3-20261026')
out = base/'reference-runtime-01'
out.mkdir()
(out/'executed-runner.py').write_bytes(Path(__file__).read_bytes())
tree = Path('/workspace/worktrees/builderbob/syson/rp-c01b-reference-inspector')
capture = json.loads((base/'captures/c01b-02/receipt.json').read_text())
assert capture['status']=='PASS' and capture['source_unchanged']
def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream,'sha256').hexdigest()
def freeze():
    return {path:sha(tree/path) for path in capture['source_before']}
before = freeze()
assert before==capture['source_after']
program = Path('/workspace/build/builderbob/syson-audit-20261026/build/target/debug/syson-reference-binding')
historical = tree/'rust/tools/reference-binding/examples/selected-request.json'
current = base/'C01b/selected-request-final-review.json'
original_review = Path('/workspace/build/builderbob/syson-rust-parity-batch2-20261026/native/review.json')
old_hash = sha(original_review)
prefix = [str(program),'/workspace','/workspace/repos/builderbob/syson']
tools = ['/usr/bin/git','/usr/bin/sha256sum']
cases = [('historical',prefix+[str(historical),*tools],1),
         ('current',prefix+[str(current),*tools],0),
         ('usage',[str(program)],2)]
records=[]
errors=[]
for name, argv, expected in cases:
    started=time.monotonic()
    child=subprocess.Popen(argv,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
    try:
        stdout,stderr=child.communicate(timeout=50)
        code=child.returncode
    except subprocess.TimeoutExpired:
        os.killpg(child.pid,signal.SIGKILL)
        stdout,stderr=child.communicate(timeout=3)
        code=124
    (out/f'{name}-stdout.txt').write_bytes(stdout)
    (out/f'{name}-stderr.txt').write_bytes(stderr)
    row=dict(case=name,argv=argv,exit_code=code,expected_exit=expected,
             wall_ms=(time.monotonic()-started)*1000,
             stdout_sha256=hashlib.sha256(stdout).hexdigest(),
             stderr_sha256=hashlib.sha256(stderr).hexdigest())
    records.append(row)
    if code!=expected:
        errors.append(f'{name}: unexpected exit {code}, expected {expected}')
    if name=='historical':
        if b'stale review harness' not in stderr:errors.append('historical: missing harness refusal')
    elif name=='current':
        try:
            report=json.loads(stdout)
            assert report['C01']=='OPEN' and report['whole_artifact_equivalence'] is False
            assert report['file_inspection_performed'] is True
            assert report['current_classpath_inventory']['jars']==9
            assert report['current_classpath_inventory']['classes']==5
        except (ValueError,KeyError,AssertionError):errors.append('current: failed inspection or claim boundary')
    else:
        if b'usage:' not in stderr:errors.append('usage: missing usage instruction')
after=freeze()
receipt=dict(status='PASS' if not errors and before==after and sha(original_review)==old_hash else 'FAIL',errors=errors,
             source_before=before,source_after=after,source_unchanged=before==after,
             program_sha256=sha(program),metadata_primitives={p:sha(Path(p)) for p in tools},
             original_review_sha256=old_hash,original_review_unchanged=sha(original_review)==old_hash,
             request_sha256={'historical':sha(historical),'current':sha(current)},
             captured_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),commands=records,
             scope='Current local file/Git/classpath consistency and explicit historical refusal. No native rerun, atomic snapshot, historical dependency immutability, publisher authentication, whole artifact or UI acceptance.')
(out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({k:receipt[k] for k in ['status','source_unchanged','original_review_unchanged','program_sha256']}))
print(json.dumps([{'case':r['case'],'exit_code':r['exit_code']} for r in records]))
assert receipt['status']=='PASS'
