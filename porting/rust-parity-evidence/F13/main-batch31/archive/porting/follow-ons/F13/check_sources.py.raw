#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Read-only source pins; never executes a parser, linker or native application."""
import hashlib
import json
from pathlib import Path
import subprocess
ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
manifest = json.loads((HERE / 'source-manifest.json').read_text())
def require(condition, reason):
    if not condition:
        raise ValueError(reason)
for relative, digest in manifest['retained_inputs'].items():
    data = (ROOT / relative).read_bytes()
    require(hashlib.sha256(data).hexdigest() == digest, 'retained source drift')
    require(data == subprocess.check_output(['git', '-C', str(ROOT), 'show', manifest['dependency_snapshot'] + ':' + relative]), 'source differs from dependency snapshot')
documents = {row['path']: row for row in map(json.loads, (ROOT / manifest['semantic_documents']).read_text().splitlines())}
for pin in manifest['sources']:
    row = documents[pin['path']]
    source = row['source']
    require(hashlib.sha256(source.encode()).hexdigest() == row['sha256'] == pin['sha256'], 'candidate source digest mismatch')
    for anchor in pin['anchors']:
        require(anchor in source, 'candidate source anchor missing')
for pin in manifest['rust_prerequisites']:
    require(hashlib.sha256((ROOT / pin['path']).read_bytes()).hexdigest() == pin['sha256'], 'Rust prerequisite drift')
print(json.dumps({'status':'PASS', 'source_pins':len(manifest['sources']) + len(manifest['rust_prerequisites']), 'source_candidate':manifest['source_candidate'], 'native_runtime':'NOT_RUN'}))
