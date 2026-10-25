#!/usr/bin/env python3
"""Read-only exact source pin check; no runtime or package lifecycle."""
import hashlib
import json
from pathlib import Path
import subprocess
root = Path(__file__).resolve().parents[3]
manifest = json.loads(Path(__file__).with_name('source-manifest.json').read_text())
for entry in manifest['sources']:
    data = (root / entry['path']).read_bytes()
    pinned = subprocess.check_output(['git', '-C', str(root), 'show', entry['git_ref'] + ':' + entry['path']])
    if data != pinned or hashlib.sha256(data).hexdigest() != entry['sha256']:
        raise SystemExit('Source pin mismatch: ' + entry['path'])
print(json.dumps({'status': 'PASS', 'source_pins': len(manifest['sources']), 'upstream_runtime': 'NOT_RUN'}))
