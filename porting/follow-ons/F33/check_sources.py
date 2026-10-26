#!/usr/bin/env python3
"""Read-only stdlib verification of immutable source/fixture observations."""
import hashlib
import json
from pathlib import Path
import subprocess
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = Path(__file__).with_name('source-manifest.json')

def git(*args):
    return subprocess.check_output(['git', '-C', str(ROOT), *args])

manifest = json.loads(MANIFEST.read_text())
for item in manifest['files']:
    data = git('show', item['revision'] + ':' + item['path'])
    assert hashlib.sha256(data).hexdigest() == item['sha256'], item['path']
    assert git('rev-parse', item['revision'] + ':' + item['path']).decode().strip() == item['git_blob']

upstream = manifest['upstream_commit']
model = ET.fromstring(git('show', upstream + ':' + manifest['ecore']))
classes = {node.attrib['name']: node for node in model if node.tag == 'eClassifiers'}
typing = classes['FeatureTyping']
refs = {node.attrib['name']: node for node in typing if node.tag == 'eStructuralFeatures'}
assert refs['type'].attrib['eType'] == '#//Type'
assert refs['typedFeature'].attrib['eType'] == '#//Feature'
assert refs['type'].attrib['lowerBound'] == refs['typedFeature'].attrib['lowerBound'] == '1'
assert classes['Element'].find("eStructuralFeatures[@name='ownedRelationship']").attrib['containment'] == 'true'
assert classes['Relationship'].find("eStructuralFeatures[@name='ownedRelatedElement']").attrib['containment'] == 'true'
fixture = json.loads(git('show', upstream + ':' + manifest['fixture']))
nodes = []
def walk(value):
    if isinstance(value, dict):
        if 'eClass' in value and 'id' in value and 'data' in value:
            nodes.append(value)
        for child in value.values():
            walk(child)
    elif isinstance(value, list):
        for child in value:
            walk(child)
walk(fixture)
required = {'Package', 'PartDefinition', 'PartUsage', 'PortDefinition', 'PortUsage', 'ViewDefinition', 'ViewUsage', 'FeatureTyping'}
assert required <= {node['eClass'].removeprefix('sysml:') for node in nodes}
by_node = {node['id']: node for node in nodes}
local_typing = next(node for node in nodes if node['eClass'] == 'sysml:FeatureTyping' and node['data'].get('type') in by_node and node['data'].get('typedFeature') in by_node)
assert by_node[local_typing['data']['type']]['data']['elementId'] != local_typing['data']['type']
assert by_node[local_typing['data']['typedFeature']]['data']['elementId'] != local_typing['data']['typedFeature']
external = next(node for node in nodes if node['eClass'] == 'sysml:FeatureTyping' and str(node['data'].get('type')).startswith('sysml:ViewDefinition sysmllibrary:///'))
print(json.dumps({'status':'PASS','pins':len(manifest['files']),'fixture_classes':sorted(required),'local_typing_node':local_typing['id'],'external_view_wire':external['data']['type'],'native_runtime':'NOT_RUN'},indent=2))
