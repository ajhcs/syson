#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Capture one local source-pin check without running a compiler or native handler."""
import hashlib
import json
from pathlib import Path
import subprocess
from datetime import datetime, timezone

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
OUT = HERE / 'validation/source-check-01'
OUT.mkdir(parents=True, exist_ok=False)
INPUTS = [HERE / n for n in ['prepare.py', 'source-manifest.json', 'source-excerpts.json']]
INPUTS += [ROOT / 'porting/source-map/syside-source' / n for n in
           ['semantic-source-documents.jsonl', 'ast-construction-obligations.jsonl']]
def sha(raw):
    return hashlib.sha256(raw).hexdigest()
def freeze():
    return {str(p.relative_to(ROOT)): sha(p.read_bytes()) for p in INPUTS}
def now():
    return datetime.now(timezone.utc).isoformat()
before = freeze()
argv = ['/usr/bin/python3', '-B', str(HERE / 'prepare.py'), '--check']
started = now()
result = subprocess.run(argv, cwd=ROOT, capture_output=True, timeout=60)
(OUT / 'stdout.txt').write_bytes(result.stdout)
(OUT / 'stderr.txt').write_bytes(result.stderr)
after = freeze()
receipt = dict(started_utc=started, completed_utc=now(), argv=argv,
    exit_code=result.returncode, source_before=before, source_after=after,
    source_unchanged=before == after, stdout_sha256=sha(result.stdout),
    stderr_sha256=sha(result.stderr), runner_sha256=sha(Path(__file__).read_bytes()),
    upstream_runtime='NOT_RUN',
    status='PASS' if result.returncode == 0 and before == after else 'FAIL')
(OUT / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
print(json.dumps({'status': receipt['status'], 'source_unchanged': receipt['source_unchanged']}))
raise SystemExit(0 if receipt['status'] == 'PASS' else 1)
