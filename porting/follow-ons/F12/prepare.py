#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Reproduce fixed source/obligation pins and excerpts; never run source handlers."""
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
BASE = ROOT / 'porting/source-map/syside-source'

def sha(raw):
    return hashlib.sha256(raw).hexdigest()

def records(name):
    return [json.loads(line) for line in (BASE / name).read_text().splitlines()]

ranges = {
    'packages/syside-languageserver/src/services/parser/parser.ts': [(57, 87), (229, 247)],
    'packages/syside-languageserver/src/services/sysml-ast-reflection.ts': [(129, 202)],
    'packages/syside-languageserver/src/generated/ast.ts': [(456, 470), (1241, 1244), (1570, 1586), (1878, 1882)],
}
sources, excerpts = [], []
for document in records('semantic-source-documents.jsonl'):
    path = document['path']
    if path not in ranges:
        continue
    raw = document['source'].encode()
    if sha(raw) != document['sha256']:
        raise ValueError('Retained source hash drift: ' + path)
    lines = document['source'].splitlines(keepends=True)
    anchors = []
    for first, last in ranges[path]:
        if not 1 <= first <= last <= len(lines):
            raise ValueError('Source line bounds')
        text = ''.join(lines[first - 1:last])
        anchors.append({'first': first, 'last': last, 'sha256': sha(text.encode())})
        excerpts.append({'path': path, 'first': first, 'last': last, 'source': text})
    sources.append({'path': path, 'sha256': sha(raw), 'anchors': anchors})
if len(sources) != 3:
    raise ValueError('Source closure changed')
obligations = records('ast-construction-obligations.jsonl')
if len(obligations) != 848:
    raise ValueError('Obligation inventory changed')
selected = [{'inventory_line': index + 1, 'identity': [r['grammar'], r['rule'], r['kind'], r['relative_path']],
             'row_sha256': sha(json.dumps(r, sort_keys=True).encode())}
            for index, r in enumerate(obligations)
            if r['grammar'] == 'SysML' and r['rule'] == 'ClassificationExpression']
if len(selected) != 19:
    raise ValueError('Classification obligation closure changed')
manifest = {
    'source_candidate': '1326f234cae23b0e4777d5a48a069b17e9e332f1',
    'evidence': 'STATIC_SOURCE_DERIVED_NOT_RUNTIME_GOLDENS',
    'inputs_sha256': {name: sha((BASE / name).read_bytes()) for name in
                     ['semantic-source-documents.jsonl', 'ast-construction-obligations.jsonl']},
    'sources': sources, 'associated_obligations': selected,
    'classification_test_operators': ['istype', 'hastype', '@', 'as'],
    'limits': ['Only structural postprocessor writes are implemented',
               'Classification grammar parsing is not implemented by this unit',
               'Native child caches and immediate parent fields are not represented',
               'Whole-handler and upstream-runtime equivalence remain unverified'],
}
for name, value in [('source-manifest.json', manifest), ('source-excerpts.json', excerpts)]:
    raw = json.dumps(value, indent=2, ensure_ascii=False) + '\n'
    path = HERE / name
    if '--check' in sys.argv:
        if path.read_text() != raw:
            raise ValueError('Prepared source artifact drift: ' + name)
    else:
        path.write_text(raw)
print(json.dumps({'status': 'PASS', 'source_documents': len(sources), 'associated_obligations': len(selected),
                  'upstream_runtime': 'NOT_RUN'}))
