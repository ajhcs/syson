#!/usr/bin/env python3
"""Record the actual bounded composition CLI against its committed snapshot."""
import datetime
import hashlib
import json
import subprocess
from pathlib import Path

base = Path('/workspace/build/builderbob/syson-rust-parity-batch3-20261026')
out = base/'composition-runtime-02'
out.mkdir()
(out/'executed-runner.py').write_bytes(Path(__file__).read_bytes())
tool = Path('/workspace/worktrees/builderbob/syson/rp-c03a-composition')
receipt = json.loads((base/'captures/c03a-02/receipt.json').read_text())
assert receipt['status']=='PASS' and receipt['source_unchanged']
def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()
def freeze():
    return {path:sha(tool/path) for path in receipt['source_before']}
before = freeze()
assert before==receipt['source_after']
program = Path('/workspace/build/builderbob/syson-audit-20261026/build/target/debug/syson-verify-composition')
manifest = base/'composition-manifest.json'
repo = Path('/workspace/worktrees/builderbob/syson/rp-migrations-composed-base')
argv = [str(program),str(repo),str(manifest)]
result = subprocess.run(argv, capture_output=True, timeout=40)
(out/'stdout.json').write_bytes(result.stdout)
(out/'stderr.txt').write_bytes(result.stderr)
after = freeze()
record = dict(argv=argv,exit_code=result.returncode,
              source_before=before,source_after=after,source_unchanged=before==after,
              program_sha256=sha(program),manifest_sha256=sha(manifest),
              captured_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
              status='PASS' if result.returncode==0 and before==after else 'FAIL',
              scope='Exact committed and selected on-disk composition only. All compiler inputs separately checked against candidate Git before compose-01; whole-program acceptance remains open.')
(out/'receipt.json').write_text(json.dumps(record,indent=2)+'\n')
print(json.dumps(record))
print(result.stdout.decode())
raise SystemExit(0 if record['status']=='PASS' else 1)
