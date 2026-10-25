#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0 OR GPL-2.0 WITH Classpath-exception-2.0
"""Check retained candidate source bytes and reviewed anchors; never execute JS."""
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parents[3]
manifest = json.loads(Path(__file__).with_name('source-manifest.json').read_text())
if manifest['source_candidate'] != '1326f234cae23b0e4777d5a48a069b17e9e332f1':
    raise ValueError('Unexpected candidate pin')
source_file = root / manifest['retained_documents']
if hashlib.sha256(source_file.read_bytes()).hexdigest() != manifest['retained_documents_sha256']:
    raise ValueError('Retained source document bytes changed')
expected = {source['path']: source for source in manifest['sources']}
found = set()
for line in source_file.open():
    document = json.loads(line)
    path = document['path']
    if path not in expected:
        continue
    if path in found:
        raise ValueError('Duplicate retained source')
    found.add(path)
    raw = document['source'].encode()
    source = expected[path]
    if hashlib.sha256(raw).hexdigest() != source['sha256'] or document['sha256'] != source['sha256']:
        raise ValueError('Source hash mismatch: ' + path)
    lines = raw.splitlines(keepends=True)
    for anchor in source['ranges']:
        first, last = anchor['first'], anchor['last']
        if not 1 <= first <= last <= len(lines):
            raise ValueError('Source range bounds: ' + path)
        if hashlib.sha256(b''.join(lines[first - 1:last])).hexdigest() != anchor['sha256']:
            raise ValueError('Source range drift: ' + path)
if found != expected.keys():
    raise ValueError('Missing retained source')
print(json.dumps({'status': 'PASS', 'source_documents': len(found), 'upstream_runtime': 'NOT_RUN'}))
