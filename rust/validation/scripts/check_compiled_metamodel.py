#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Compare compiled registry observations with independently parsed pinned XML."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import subprocess
import xml.etree.ElementTree as ET

PIN = '6ba2ac64e6b16ee2b05752ad55395eebb87d4910'
PATHS = [('Domain', 'backend/metamodel/syson-sysml-metamodel/src/main/resources/model/sysml.ecore'),
         ('CustomNodes', 'backend/metamodel/syson-siriusweb-customnodes-metamodel/src/main/resources/model/sysml-customnodes.ecore')]
XSI = '{http://www.w3.org/2001/XMLSchema-instance}type'


def check(root, observed_path):
    if not __debug__:
        raise RuntimeError("Run evidence checks without Python optimization")
    observed = json.loads(observed_path.read_text())
    trees = {}
    source_records = []
    for package, path in PATHS:
        raw = subprocess.check_output(['git', '-C', str(root), 'show', f'{PIN}:{path}'])
        tree = ET.fromstring(raw)
        source_records.append({'package': package, 'path': path, 'revision': PIN,
                               'sha256': hashlib.sha256(raw).hexdigest(), 'attributes': tree.attrib})
        for child in tree:
            assert child.tag == 'eClassifiers'
            key = package + '::' + child.attrib['name']
            assert key not in trees
            trees[key] = child
    assert observed['sources'] == source_records, 'compiled source provenance/package attributes differ'
    actual = {record['key']: record for record in observed['classifiers']}
    assert len(actual) == len(observed['classifiers']) and set(actual) == set(trees), 'classifier identities differ'
    counts = Counter()

    def metadata(node):
        # Registry deliberately omits formatting, never semantic XML text.
        assert not (node.text or '').strip() and not (node.tail or '').strip(), 'non-whitespace XML text omitted'
        counts[node.tag] += 1
        return {'tag': node.tag, 'attributes': node.attrib, 'children': [metadata(child) for child in node]}

    def reference(package, raw):
        if raw is None:
            return None
        local = package + '::' + raw[3:] if raw.startswith('#//') else None
        return {'raw': raw, 'local': local if local in trees else None,
                'resolution': 'local' if local in trees else ('unresolved_local' if local else 'external')}

    def typed(package, node):
        return {'name': node.attrib['name'], 'type': reference(package, node.get('eType')),
                'lower': int(node.get('lowerBound', '0')), 'upper': int(node.get('upperBound', '1')),
                'ordered': node.get('ordered', 'true') == 'true', 'unique': node.get('unique', 'true') == 'true'}

    def super_references(key):
        package = key.split('::')[0]
        return [reference(package, raw) for raw in trees[key].get('eSuperTypes', '').split()]

    def ancestors(key, path=()):
        if key in path:
            return set(), set()
        local = {key}
        external = set()
        for parent in super_references(key):
            if parent['local']:
                parent_local, parent_external = ancestors(parent['local'], path + (key,))
                local |= parent_local
                external |= parent_external
            else:
                external.add(parent['raw'])
        return local, external

    def feature_lookup(key, name, path=()):
        if key in path:
            return set(), set()
        own = {key + '.' + name for feature in trees[key].findall('eStructuralFeatures') if feature.get('name') == name}
        if own:
            return own, set()
        found, external = set(), set()
        # Enumerate independent paths then deduplicate by declaring identity;
        # this does not reuse the registry's global visited-set algorithm.
        for parent in super_references(key):
            if parent['local']:
                candidates, unresolved = feature_lookup(parent['local'], name, path + (key,))
                found |= candidates
                external |= unresolved
            else:
                external.add(parent['raw'])
        return found, external

    names = {feature.get('name') for tree in trees.values() for feature in tree.findall('eStructuralFeatures')}
    feature_lookups = 0
    for key, tree in trees.items():
        record = actual[key]
        package = key.split('::')[0]
        kind = 'Class' if tree.get(XSI) == 'ecore:EClass' else 'Enum'
        assert record['kind'] == kind, key + ': kind'
        assert record['abstract'] == (tree.get('abstract', 'false') == 'true'), key + ': abstract'
        assert record['interface'] == (tree.get('interface', 'false') == 'true'), key + ': interface'
        assert record['node'] == metadata(tree), key + ': recursive metadata'
        assert record['supertypes'] == super_references(key), key + ': direct supertypes'
        features = []
        for feature in tree.findall('eStructuralFeatures'):
            result = {'typed': typed(package, feature), 'kind': 'Reference' if feature.get(XSI) == 'ecore:EReference' else 'Attribute',
                      'opposite': feature.get('eOpposite'), 'default': feature.get('defaultValueLiteral')}
            for output_name, source_name, default in [('containment','containment',False), ('resolve_proxies','resolveProxies',True),
                                                      ('derived','derived',False), ('volatile','volatile',False), ('transient','transient',False),
                                                      ('changeable','changeable',True), ('unsettable','unsettable',False), ('is_id','iD',False)]:
                result[output_name] = feature.get(source_name, str(default).lower()) == 'true'
            features.append(result)
        assert record['features'] == features, key + ': typed feature metadata/defaults'
        operations = [{'typed': typed(package, operation), 'parameters': [typed(package, param) for param in operation.findall('eParameters')]}
                      for operation in tree.findall('eOperations')]
        assert record['operations'] == operations, key + ': operation/parameter metadata'
        literals = [{'name': literal.get('name'), 'value': int(literal.get('value', '0')),
                     'literal': literal.get('literal', literal.get('name'))} for literal in tree.findall('eLiterals')]
        assert record['enum_literals'] == literals, key + ': enum spelling/value/order'
        local, external = ancestors(key)
        if kind == 'Enum':
            local = set()
        else:
            local = {name for name in local if trees[name].get(XSI) == 'ecore:EClass'}
        assert set(record['local_ancestors']) == local, key + ': transitive local inheritance'
        assert set(record['unresolved_ancestors']) == external, key + ': external ancestry'
        assert set(record['inherited_features']) == names, key + ': lookup name coverage'
        for name in names:
            found, unresolved = feature_lookup(key, name)
            result = record['inherited_features'][name]
            assert set(result['candidates']) == found, key + '.' + name + ': inherited candidates'
            assert set(result['unresolved']) == unresolved, key + '.' + name + ': unresolved lookup ancestry'
            assert result['unique'] == (next(iter(found)) if len(found) == 1 else None), key + '.' + name + ': local uniqueness'
            feature_lookups += 1
    return {'classifiers': len(actual), 'local_subtype_pairs': len(actual)**2,
            'feature_name_lookups': feature_lookups,
            'metadata_counts': dict(sorted(counts.items())),
            'result': 'compiled metadata and local structural queries match pinned declarations; semantic runtime unverified'}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parents[3])
    parser.add_argument('observations', type=Path)
    args = parser.parse_args()
    print(json.dumps(check(args.root, args.observations), sort_keys=True))
