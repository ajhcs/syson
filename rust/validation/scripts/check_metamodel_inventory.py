#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Compare every inventoried Ecore declaration with its pinned XML tree."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import subprocess
import xml.etree.ElementTree as ET

PIN = '6ba2ac64e6b16ee2b05752ad55395eebb87d4910'


def compare(node, record, location):
    assert record['tag'] == node.tag, location + ': tag'
    assert record['attributes'] == node.attrib, location + ': attributes'
    assert record['text'] == node.text and record['tail'] == node.tail, location + ': text/tail'
    assert record['source_location'] == location, location + ': location'
    assert len(record['children']) == len(node), location + ': child count'
    ordinals = Counter()
    for child, saved in zip(node, record['children']):
        position = ordinals[child.tag]
        ordinals[child.tag] += 1
        compare(child, saved, f'{location}/@{child.tag}.{position}')


def check(root, inventory):
    if not __debug__:
        raise RuntimeError("Run evidence checks without Python optimization")
    def git(*args):
        return subprocess.check_output(['git', '-C', str(root), *args])
    paths = [p for p in git('ls-tree', '-r', '--name-only', PIN).decode().splitlines() if p.endswith('.ecore')]
    records = [json.loads(line) for line in inventory.read_text().splitlines()]
    expected = []
    counts = Counter()
    for path in paths:
        raw = git('show', f'{PIN}:{path}')
        tree = ET.fromstring(raw)
        for ordinal, node in enumerate(tree.findall('eClassifiers')):
            expected.append((path, node, hashlib.sha256(raw).hexdigest(), ordinal))
            counts.update(item.tag for item in node.iter())
    assert len(records) == len(expected), 'classifier record count differs'
    for record, (path, node, digest, ordinal) in zip(records, expected):
        assert record['path'] == path and record['name'] == node.get('name'), 'classifier order/name differs'
        assert record['source_sha256'] == digest, 'source blob hash differs'
        compare(node, record['declaration'], f'{path}#//@eClassifiers.{ordinal}')
    for required in ('eParameters', 'eAnnotations', 'eLiterals'):
        assert counts[required] > 0, f'no {required} independently checked'
    return {'classifiers': len(records), 'ecore_files': len(paths),
            'nested_nodes': {key: counts[key] for key in ('eParameters', 'eAnnotations', 'eLiterals')},
            'result': 'pinned parsed XML declarations match; semantic closure not established'}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parents[3])
    parser.add_argument('--inventory', type=Path)
    args = parser.parse_args()
    inventory = args.inventory or args.root / 'porting/source-map/metamodel.jsonl'
    print(json.dumps(check(args.root, inventory), sort_keys=True))
