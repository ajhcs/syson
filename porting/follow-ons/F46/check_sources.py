#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Verify immutable source receipts; this does not execute Java or native EMF."""
import hashlib
import json
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parents[3]
manifest = json.loads((Path(__file__).parent / 'source-manifest.json').read_text())
rows = manifest['sources']
if len(rows) != 11 or len({r['path'] for r in rows}) != 11:
    raise SystemExit('Unexpected source pin inventory')
for row in rows:
    relative = Path(row['path'])
    if relative.is_absolute() or '..' in relative.parts:
        raise SystemExit('Unsafe source pin path')
    current = (root / relative).read_bytes()
    pinned = subprocess.check_output(['git', '-C', str(root), 'show', row['commit'] + ':' + row['path']])
    if current != pinned or hashlib.sha256(current).hexdigest() != row['sha256']:
        raise SystemExit('Source pin mismatch: ' + row['path'])
print(json.dumps({'status': 'PASS', 'source_pins': len(rows), 'native_runtime': 'NOT_RUN'}))
