#!/usr/bin/env python3
"""Check captured typed finite cardinality and snapshot bytes."""
from hashlib import sha256
import importlib.util
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
SOURCE = 'package P { part def W; part def C { part w : W[4]; } }'
PREFIX = 'package P { part def W; attribute def Wrong; part def C { part good : W[4]; } }'
POSITIVE = {
    'finite-four': (SOURCE, 'w', 4, 12),
    'zero': (SOURCE.replace('[4]', '[0]'), 'w', 0, 12),
    'max': (SOURCE.replace('[4]', '[2147483647]'), 'w', 2147483647, 12),
    'unbounded-absent': (SOURCE.replace('[4]', ''), 'w', None, 8),
    'valid-prefix': (PREFIX, 'good', 4, 14),
}
REFUSALS = {
    'arithmetic': ('2+2', 'Unsupported("integer literal with source")', 7),
    'negative': ('-1', 'Unsupported("integer literal with source")', 7),
    'overflow': ('2147483648', 'Unsupported("literal outside nonnegative EInt range")', 7),
    'range': ('1..4', 'UnsupportedInfixOperator(..)', 0),
    'rational': ('1.0', 'Unsupported("finite integer cardinality")', 7),
    'star': ('*', 'UnsupportedModelKind(LiteralInfinity)', 0),
}
COUNTS = {
    'default': 62, 'prior-success': 105, 'prior-refusal': 13, 'null-success': 8,
    'null-refusal': 2, 'nonunique-success': 4, 'nonunique-refusal': 1,
}
MATCH = {
    'reference_nodes': 12, 'rust_nodes': 12, 'structural_match': True, 'structural_diffs': [],
    'wire_diffs': ['import root Namespace wrapper: reference=True rust=False',
                   "cross-references point at: reference=['id'] rust=['elementId']"],
}


def require(condition, reason):
    if not condition:
        raise ValueError(reason)


def digest(data):
    return sha256(data).hexdigest()


def walk(node):
    if isinstance(node, dict):
        if 'eClass' in node:
            yield node
        for value in node.values():
            yield from walk(value)
    elif isinstance(node, list):
        for value in node:
            yield from walk(value)


def named(document, concept, name):
    matches = [node for node in walk(document)
               if node['eClass'] == 'sysml:'+concept and node['data'].get('declaredName') == name]
    require(len(matches) == 1, f'exact {concept} declaration for {name}')
    return matches[0]


def child(node, concept, field):
    require(node['eClass'] == 'sysml:'+concept, f'{concept} containment class')
    children = node['data'][field]
    require(len(children) == 1, f'{concept} containment count')
    return children[0]


def relationship(usage, cardinality):
    require(usage['eClass'] == 'sysml:PartUsage', 'cardinality owner part usage')
    fields = usage['data']
    require(fields['isComposite'] is True and 'isUnique' not in fields, 'unchanged usage flags')
    relations = fields['ownedRelationship']
    if cardinality is None:
        require(len(relations) == 1, 'absent cardinality preserves typing only')
    else:
        require(len(relations) == 2, 'cardinality precedes typing')
        membership = relations[0]
        range_node = child(membership, 'OwningMembership', 'ownedRelatedElement')
        member = child(range_node, 'MultiplicityRange', 'ownedRelationship')
        bound = child(member, 'OwningMembership', 'ownedRelatedElement')
        require(bound['eClass'] == 'sysml:LiteralInteger', 'finite bound classifier')
        require(set(range_node['data']) == {'elementId', 'ownedRelationship'}, 'derived bounds omitted')
        fields = bound['data']
        if cardinality == 0:
            require(set(fields) == {'elementId'}, 'zero uses literal integer default')
        else:
            require(set(fields) == {'elementId', 'value'} and
                    type(fields['value']) is int and fields['value'] == cardinality, 'literal integer bound')
        nodes = (membership, range_node, member, bound)
        ids = {node['id'] for node in nodes}
        element_ids = {node['data']['elementId'] for node in nodes}
        require(len(ids) == len(element_ids) == 4 and ids.isdisjoint(element_ids), 'four paired identities')
    typing = relations[-1]
    require(typing['eClass'] == 'sysml:FeatureTyping', 'owned feature typing')
    return typing['data']


def model(doc, name, value, native=False):
    usage = named(doc, 'PartUsage', name)
    target = named(doc, 'PartDefinition', 'W')
    typing = relationship(usage, value)
    identities = [node['id'] if native else node['data']['elementId'] for node in (usage, target)]
    require([typing['typedFeature'], typing['type']] == identities, 'owner and target references')


def saved(row, source, name, value, ids):
    require((row['source'], row['usage'], row['value'], row['identities_requested']) ==
            (source, name, value, ids), 'positive fixture identity')
    writer, reader = row['writer'], row['reader']
    require(type(writer) is int and type(reader) is int and writer > 0 and reader > 0 and writer != reader,
            'fresh snapshot reopen processes')
    snapshot, reopen = row['snapshot'], row['reopen_stdout']
    require(digest(snapshot.encode()) == row['snapshot_sha256'], 'snapshot digest')
    require(digest(reopen.encode()) == row['reopen_sha256'] and reopen == snapshot+'\n', 'reopen bytes')
    stage = json.loads(row['stage_stdout'])
    doc = json.loads(snapshot)
    require(stage['status'] == 'ok' and stage['document'] == doc and
            stage['identities_requested'] == ids, 'stage persistence and identity count')
    require(stage['targets'] == [[{'raw': 'W', 'target': 'Type'}]], 'stage link target')
    require(len(list(walk(doc))) == ids, 'one model node per requested identity')
    raw = source.encode()
    literals = []
    for lexeme in stage['lexemes']:
        start, end = lexeme['byte']
        require(0 <= start < end <= len(raw), 'source span bounds')
        token = raw[start:end].decode()
        offset = len(raw[:start].decode().encode('utf-16-le'))//2
        count = len(token.encode('utf-16-le'))//2
        require(token == lexeme['raw'] and lexeme['utf16'] == [offset, offset+count], 'byte and UTF-16 source spans')
        if lexeme['feature'] == 'literal':
            literals.append(token)
    require(literals == ([] if value is None else [str(value)]), 'cardinality literal lexeme')
    model(doc, name, value)
    return doc


def refused(row, source, error, ids):
    require(row['snapshot_present'] is False and row['source'] == source, 'atomic refusal source and absence')
    expected = {'status': 'error', 'error': error, 'identities_requested': ids}
    require(json.loads(row['stage_stdout']) == expected, 'exact refusal')
    require(all(row[key] == val for key, val in expected.items()), 'refusal metadata')


def verify(receipt, raw, compare):
    require(receipt['schema'] == 1 and receipt['scope'] == 'off-tree/retained-native/singular-typed-PartUsage', 'packet scope')
    require(receipt['base'] == '05a4be35d4842272d0249ab43b9e49295591ef1e', 'Rust base')
    require(receipt['pinned_source'] == '6ba2ac64e6b16ee2b05752ad55395eebb87d4910', 'baseline source pin')
    require(receipt['reference_release'] == {
        'version': '2026.9.3', 'source': 'b380dada415ce05155150f23cb3b74ae21cc45e4',
        'jar_sha256': '85c031352695d6ac8f059f0215ca493c58a16805a111233998dd443d6990c9e1',
    }, 'native release pin')
    require(receipt['retained_capture_sha256'] == 'd0704ad96ae66f69a7943e7e2fd93dbd00dbe5d07d64b97d9e25f29597353840',
            'retained reduced capture pin')
    require(receipt['regression_counts'] == COUNTS and receipt['default_aggregate_sha256'] ==
            '96e3bd0a85b8850f653192a3c18c835c8010d30461e6fc0434ca65dd9da4674c', 'regression inventory')
    require(receipt['cancellation'] == {'status': 'error', 'error': 'Cancelled', 'identities_requested': 0}, 'cancel receipt')
    require(digest(raw) == receipt['cases_sha256'], 'case manifest digest')
    entries = [json.loads(line) for line in raw.splitlines()]
    rows = {row['case']: row for row in entries}
    require(len(entries) == len(rows) and set(rows) == set(POSITIVE) | set(REFUSALS) | {'late-wrong-type'}, 'case inventory')
    for row in rows.values():
        require(digest(row['source'].encode()) == row['source_sha256'] and
                digest(row['stage_stdout'].encode()) == row['stage_sha256'], 'source and stage digests')

    docs = {name: saved(rows[name], *expected) for name, expected in POSITIVE.items()}
    four = rows['finite-four']
    require({row['case'] for row in rows.values() if 'reference_document' in row} == {'finite-four'}, 'native case scope')
    native = four['reference_document']
    model(native, 'w', 4, native=True)
    require(compare(native, docs['finite-four']) == four['comparison'] == MATCH, 'normalized retained reference')

    bat = receipt['batmobile']
    require(bat['line'] == 73 and bat['source_line'].strip() == 'part wheels : Wheel [4];' and
            bat['has_diagnostics'] is True, 'Batmobile projection scope')
    require(bat['source_sha256'] == 'e7c7a70af9c799a0a5b9e430eec4f09fdc8d44dc623cbc8ddb3c3841e53f9bb7' and
            bat['capture_sha256'] == '2d4b3cc509ed35a41f879c6c3d936da64791ea3459cbd64b0721c518e2d1cc57', 'upstream projection pins')
    require(bat['projection']['data']['declaredName'] == 'wheels', 'upstream PartUsage name')
    relationship(bat['projection'], 4)
    for name, (spelling, error, count) in REFUSALS.items():
        refused(rows[name], SOURCE.replace('[4]', f'[{spelling}]'), error, count)
    wrong = named(docs['valid-prefix'], 'AttributeDefinition', 'Wrong')['data']['elementId']
    refused(rows['late-wrong-type'], PREFIX.replace('W[4]; }', 'W[4]; part bad : Wrong; }'),
            f'Model(WrongType {{ element: "{wrong}", expected: "PartDefinition" }})', 17)


def main():
    spec = importlib.util.spec_from_file_location('compare', HERE.parents[1]/'compare.py')
    comparison = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(comparison)
    verify(json.loads((HERE/'receipt.json').read_bytes()), (HERE/'cases.jsonl').read_bytes(), comparison.compare)
    print('1 retained match, 1 upstream projection, 4 local reopens and 7 refusals')


if __name__ == '__main__':
    main()
