#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Checks exact source pins, without executing a parser/native importer."""
import hashlib
import json
from pathlib import Path
import subprocess
ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
rows = json.loads((HERE / 'source-manifest.json').read_text())['sources']
for row in rows:
    current = (ROOT / row['path']).read_bytes()
    pinned = subprocess.check_output(['git', '-C', str(ROOT), 'show', row['revision'] + ':' + row['path']])
    if current != pinned or hashlib.sha256(current).hexdigest() != row['sha256']:
        raise ValueError('source drift: ' + row['path'])
    if not all(anchor in current.decode() for anchor in row['anchors']):
        raise ValueError('source anchor drift: ' + row['path'])
print(json.dumps({'status':'PASS','source_pins':len(rows),'native_runtime':'NOT_RUN'}))
