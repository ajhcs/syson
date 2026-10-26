#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Check pinned construction/encoding evidence; no native runtime oracle."""
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
packet = json.loads(Path(__file__).with_name('source-manifest.json').read_text())
for row in packet['sources']:
    path = row['path']
    actual = (ROOT / path).read_bytes()
    pinned = subprocess.check_output(['git', '-C', str(ROOT), 'show', row['commit'] + ':' + path])
    if actual != pinned or hashlib.sha256(actual).hexdigest() != row['sha256']:
        raise SystemExit('Source drift: ' + path)
p = ROOT / 'backend/application/syson-application-configuration/src/main/resources/templates/Batmobile.json'
found = []
def visit(value, parent=None):
    if isinstance(value, dict):
        if value.get('id') == 'b38b489f-9052-47f7-a417-0c1e137d0dca':
            found.append((value, parent))
        for child in value.values():
            visit(child, value if 'eClass' in value else parent)
    elif isinstance(value, list):
        for child in value:
            visit(child, parent)
visit(json.loads(p.read_text()))
if len(found) != 1:
    raise SystemExit('Native ViewUsage fixture identity drift')
view, parent = found[0]
typing = next(n for n in view['data']['ownedRelationship'] if n['eClass'] == 'sysml:FeatureTyping')
if not (parent['eClass'] == 'sysml:OwningMembership' and view['eClass'] == 'sysml:ViewUsage'
        and view['id'] != view['data']['elementId'] and typing['data']['typedFeature'] == view['id']
        and typing['data']['type'] == 'sysml:ViewDefinition sysmllibrary:///faf517ae-dbcd-30a4-b3b9-3d9cb3bbf5c1#03904fdf-d6f2-57b1-92d5-95d36b8208dc'):
    raise SystemExit('Native membership/typing wire shape drift')
print(json.dumps({'status':'PASS','source_pins':len(packet['sources']), 'native_fixture_identity_example':1,
                  'native_runtime':'NOT_RUN'},sort_keys=True))
