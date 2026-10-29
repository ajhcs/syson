#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Reproduce finite source pins and conservative keyword admission; no oracle runs."""
import hashlib
import json
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
BASE = ROOT / 'porting/source-map/syside-source'
SELECTED = [398, 397, 400, 416, 439, 443, 444, 448, 803, 823, 824, 825, 826, 828]

def sha(raw):
    return hashlib.sha256(raw).hexdigest()

def records(name):
    return [json.loads(line) for line in (BASE / name).read_text().splitlines()]

words = set()
def walk(value):
    if isinstance(value, dict):
        if value.get('$type') == 'Keyword' and re.fullmatch(r'[A-Za-z_][A-Za-z_0-9]*', value.get('value', '')):
            words.add(value['value'])
        for child in value.values():
            walk(child)
    elif isinstance(value, list):
        for child in value:
            walk(child)

for document in records('generated-grammar-documents.jsonl'):
    walk(document['grammar'])
if not {'package', 'part', 'def', 'private', 'in'} <= words or len(words) < 100:
    raise ValueError('Missing keyword admission closure')
reserved = '// Generated conservative keyword-union admission exclusions; see F11 source manifest.\nconst RESERVED: &[&str] = &[\n' + ''.join('    ' + json.dumps(word) + ',\n' for word in sorted(words)) + '];\n'
obligations = records('ast-construction-obligations.jsonl')
if len(obligations) != 848:
    raise ValueError('Obligation inventory changed')
selected = [{'inventory_line': line, **obligations[line - 1]} for line in SELECTED]
if any(item['grammar'] != 'SysML' for item in selected):
    raise ValueError('Whitelist grammar changed')
inputs = ['generated-grammar-documents.jsonl', 'grammar-source-documents.jsonl', 'semantic-source-documents.jsonl', 'ast-construction-obligations.jsonl']
source_paths = {
    'packages/syside-languageserver/src/grammar/SysML.langium',
    'packages/syside-languageserver/src/grammar/KerML.expressions.langium',
    'packages/syside-languageserver/src/generated/ast.ts',
    'packages/syside-languageserver/src/services/sysml-ast-reflection.ts',
}
sources = []
for name in ['grammar-source-documents.jsonl', 'semantic-source-documents.jsonl']:
    for document in records(name):
        if document['path'] in source_paths:
            actual = sha(document['source'].encode())
            if actual != document['sha256']:
                raise ValueError('Retained document hash mismatch')
            sources.append({'path': document['path'], 'sha256': actual})
if len(sources) != 4:
    raise ValueError('Missing selected source documents')
manifest = {
    'source_candidate': '1326f234cae23b0e4777d5a48a069b17e9e332f1',
    'evidence': 'STATIC_SOURCE_DERIVED_NOT_RUNTIME_GOLDENS',
    'input_sha256': {name: sha((BASE / name).read_bytes()) for name in inputs},
    'sources': sources,
    'source_obligation_count': len(obligations),
    'selected_obligations': selected,
    'selected_count': len(selected),
    'keyword_count': len(words),
    'keyword_policy': 'Conservative union of retained grammar keywords; exclusion is Rust admission, not proof of native lexical rejection',
    'reserved_sha256': sha(reserved.encode()),
    'unsupported': ['all other construction obligations', 'native parser recovery', 'semantic import/linking', 'postprocessor execution', 'child caches'],
}
outputs = {ROOT / 'rust/crates/syson-ast-adapter/src/reserved.rs': reserved,
           HERE / 'source-manifest.json': json.dumps(manifest, indent=2) + '\n'}
for path, text in outputs.items():
    if '--check' in sys.argv:
        if path.read_text() != text:
            raise ValueError('Prepared source artifact drift: ' + str(path))
    else:
        path.write_text(text)
print(json.dumps({'status': 'PASS', 'source_documents': len(sources), 'selected_obligations': len(selected), 'keywords': len(words), 'upstream_runtime': 'NOT_RUN'}))
