#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Verify exact background source blobs; no upstream/browser/assistive execution."""
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
packet = json.loads((Path(__file__).parent / 'source-manifest.json').read_text())
if packet['schema'] != 'F36-SOURCE-PINS/1' or len(packet['evidence']) != 5:
    raise SystemExit('Unexpected source packet')
for row in packet['evidence']:
    path = Path(row['path'])
    if path.is_absolute() or '..' in path.parts:
        raise SystemExit('Unexpected source path')
    data = subprocess.check_output(['git', '-C', str(ROOT), 'show', row['commit'] + ':' + row['path']])
    if hashlib.sha256(data).hexdigest() != row['sha256']:
        raise SystemExit('Pinned source differs: ' + row['path'])
print(json.dumps({'source_pins':'PASS','count':5,'native_inspector_runtime':'NOT_RUN','screen_reader_speech':'NOT_RUN'}))
