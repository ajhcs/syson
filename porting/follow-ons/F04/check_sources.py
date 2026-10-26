#!/usr/bin/env python3
"""Standard-library-only byte pins and native identity examples; no runtime oracle."""
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = json.loads((Path(__file__).parent / 'source-manifest.json').read_text())

def require(condition, message):
    if not condition:
        raise RuntimeError(message)

for row in MANIFEST['files']:
    data = (ROOT / row['path']).read_bytes()
    blob = subprocess.check_output(['git', '-C', str(ROOT), 'show', MANIFEST['upstream_commit'] + ':' + row['path']])
    require(data == blob and hashlib.sha256(data).hexdigest() == row['sha256'], row['path'])

def objects(value):
    if isinstance(value, dict):
        yield value
        for nested in value.values():
            yield from objects(nested)
    elif isinstance(value, list):
        for nested in value:
            yield from objects(nested)

base = ROOT / 'backend/application/syson-application-configuration/src/main/resources'
batmobile = json.loads((base / 'templates/Batmobile.json').read_text())
nodes = {n['id']: n for n in objects(batmobile) if 'eClass' in n}
target = nodes['b966a2dd-cd51-4996-8449-d4e9d5b48588']
require(target['data']['elementId'] == '80078007-9a1f-42eb-9456-1d0b8d63703d' and target['eClass'] == 'sysml:PortDefinition', 'native local identity pair')
require(any(n.get('type') == target['id'] for n in objects(batmobile)), 'native bare node-id type reference')
views = json.loads((base / 'sysml.libraries/StandardViewDefinitions.json').read_text())
require(any(n.get('importedNamespace') == 'sysml:Namespace sysmllibrary:///389615e8-f636-3405-928f-411d874ba837#f8861c88-124d-527d-9946-d707fb209b47' for n in objects(views)), 'native typed external library reference')
require(any(n.get('id') == 'd7955624-057b-45d2-98f9-671776192420' and n['data']['elementId'] == '03904fdf-d6f2-57b1-92d5-95d36b8208dc' and n['data']['declaredName'] == 'GeneralView' for n in objects(views)), 'native GeneralView identity pair')
print(json.dumps({'status': 'PASS', 'source_pins': len(MANIFEST['files']), 'identity_examples': 4, 'upstream_runtime': 'NOT_RUN'}))
