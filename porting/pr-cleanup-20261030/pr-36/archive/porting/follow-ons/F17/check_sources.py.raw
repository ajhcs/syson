#!/usr/bin/env python3
"""Check the inherited F16 source contract; never execute upstream layout code."""
import hashlib
import json
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parents[3]
record = json.loads(Path(__file__).with_name('source-contract.json').read_text())
if record['prerequisite_revision'] != '6c198db81434ad4b1f318a6895fee8933a331d53':
    raise ValueError('Unexpected F16 prerequisite')
for source in record['files']:
    spec = record['prerequisite_revision'] + ':' + source['path']
    data = subprocess.check_output(['git', '-C', str(root), 'show', spec])
    blob = subprocess.check_output(['git', '-C', str(root), 'rev-parse', spec], text=True).strip()
    if blob != source['blob'] or hashlib.sha256(data).hexdigest() != source['sha256']:
        raise ValueError('Pinned dependency source mismatch: ' + source['path'])
    if (root / source['path']).read_bytes() != data:
        raise ValueError('Dependency worktree changed: ' + source['path'])
print(json.dumps({'status': 'PASS', 'pinned_prerequisite_files': len(record['files']),
    'upstream_runtime': 'NOT_RUN', 'algorithm_origin': 'PROPOSED_RUST_NOT_ELK'}))
