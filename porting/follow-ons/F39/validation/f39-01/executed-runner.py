#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Capture only the source checker and Python tests; no proposed command executes."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
from datetime import datetime, timezone

ROOT = Path(__file__).resolve().parents[3]
SOURCE = ROOT / 'porting/reference-preflight'
run = sys.argv[1]
if run != 'f39-01':
    raise ValueError('Unexpected capture ID')
OUT = Path(__file__).parent / 'validation' / run
OUT.mkdir(parents=True, exist_ok=False)
sha = lambda data: hashlib.sha256(data).hexdigest()
now = lambda: datetime.now(timezone.utc).isoformat()
def freeze():
    return {str(p.relative_to(ROOT)): sha(p.read_bytes()) for p in sorted(SOURCE.iterdir())
            if p.is_file() and p.name != 'README.md'}
commands = [[sys.executable, '-B', str(SOURCE / 'check.py')],
            [sys.executable, '-B', '-m', 'unittest', 'discover', '-s', str(SOURCE), '-v']]
r = {'started_utc': now(), 'source_base': subprocess.check_output(['git', '-C', str(ROOT), 'rev-parse', 'HEAD'], text=True).strip(),
     'source_before': freeze(), 'runner_sha256': sha(Path(__file__).read_bytes()), 'commands': [],
     'upstream_runtime': 'NOT_RUN', 'lifecycle': 'NOT_RUN', 'network_probe': 'NOT_RUN'}
for index, argv in enumerate(commands):
    started = now()
    result = subprocess.run(argv, cwd=ROOT, capture_output=True, timeout=30,
                            env={**os.environ, 'PYTHONDONTWRITEBYTECODE': '1'})
    (OUT / f'{index:02d}-stdout.txt').write_bytes(result.stdout)
    (OUT / f'{index:02d}-stderr.txt').write_bytes(result.stderr)
    r['commands'].append({'argv': argv, 'started_utc': started, 'completed_utc': now(),
        'exit_code': result.returncode, 'stdout_sha256': sha(result.stdout), 'stderr_sha256': sha(result.stderr)})
    if result.returncode:
        r['status'] = 'FAIL'
        break
else:
    r['status'] = 'PASS'
r.update(completed_utc=now(), source_after=freeze())
r['source_unchanged'] = r['source_before'] == r['source_after']
if not r['source_unchanged']:
    r['status'] = 'SOURCE_DRIFT'
(OUT / 'receipt.json').write_text(json.dumps(r, indent=2) + '\n')
print(json.dumps({'status': r['status'], 'source_unchanged': r['source_unchanged'], 'receipt': str(OUT / 'receipt.json')}))
sys.exit(0 if r['status'] == 'PASS' else 1)
