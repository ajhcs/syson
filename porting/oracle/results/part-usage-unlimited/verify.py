#!/usr/bin/env python3
"""Check captured PartUsage Infinity projections and Rust snapshot bytes."""
from hashlib import sha256
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
PINS = ('6ba2ac64e6b16ee2b05752ad55395eebb87d4910', 'b380dada415ce05155150f23cb3b74ae21cc45e4')
SOURCE = 'package P { part def System; part systems : System [*]; }'
PREFIX = 'package P { part def System; attribute def Wrong; part def C { part good : System[*]; } }'
OLD_STAR = 'package P { part def W; part def C { part w : W[*]; } }'
FEATURE_STAR = 'package P { attribute def N; part def C { attribute n : N = *; } }'
DENIED_INFINITY = 'UnsupportedModelKind(LiteralInfinity)'
POSITIVE = {
    'unlimited': (SOURCE, 'systems', 'System', 10),
    'previously-refused': (OLD_STAR, 'w', 'W', 12),
    'valid-prefix': (PREFIX, 'good', 'System', 14),
}
NEGATIVE = {
    'feature-star': (FEATURE_STAR, DENIED_INFINITY, 0),
    'mixed-range': (SOURCE.replace('[*]', '[0..*]'), 'UnsupportedInfixOperator(..)', 0),
    'signed-star': (SOURCE.replace('[*]', '[-*]'), DENIED_INFINITY, 0),
}
COUNTS = {
    'cardinality-refusal': 6, 'cardinality-success': 5, 'default': 62,
    'nonunique-refusal': 1, 'nonunique-success': 4, 'null-refusal': 2,
    'null-success': 8, 'prior-refusal': 13, 'prior-success': 105,
}


def require(condition, reason):
    if not condition:
        raise ValueError(reason)


def digest(data):
    return sha256(data).hexdigest()


def walk(item):
    if isinstance(item, dict):
        if 'eClass' in item:
            yield item
        for value in item.values():
            yield from walk(value)
    elif isinstance(item, list):
        for value in item:
            yield from walk(value)


def named(doc, role, name):
    nodes = [n for n in walk(doc)
             if n['eClass'] == 'sysml:'+role and n['data'].get('declaredName') == name]
    require(len(nodes) == 1, f'exact {role} {name}')
    return nodes[0]


def child(node, kind, slot):
    require(node['eClass'] == 'sysml:'+kind, f'{kind} containment class')
    items = node['data'][slot]
    require(len(items) == 1, f'{kind} one child')
    return items[0]


def project(usage, target, id_field):
    require(usage['eClass'] == 'sysml:PartUsage', 'bounded usage kind')
    data = usage['data']
    require(data['isComposite'] is True and 'isUnique' not in data, 'unlimited usage flags')
    relationships = data['ownedRelationship']
    require(len(relationships) == 2, 'one multiplicity membership before typing')
    outer, typing = relationships
    multiplicity = child(outer, 'OwningMembership', 'ownedRelatedElement')
    bound = child(multiplicity, 'MultiplicityRange', 'ownedRelationship')
    literal = child(bound, 'OwningMembership', 'ownedRelatedElement')
    require(set(multiplicity['data']) == {'elementId', 'ownedRelationship'}, 'derived bounds absent')
    require(literal['eClass'] == 'sysml:LiteralInfinity' and set(literal['data']) == {'elementId'},
            'stored Infinity without numeric sentinel or derived zero')
    nodes = (outer, multiplicity, bound, literal)
    resources = {n['id'] for n in nodes}
    semantics = {n['data']['elementId'] for n in nodes}
    require(len(resources) == len(semantics) == 4 and resources.isdisjoint(semantics), 'four paired identities')
    owner = usage['id'] if id_field == 'id' else data['elementId']
    require(typing['eClass'] == 'sysml:FeatureTyping' and
            [typing['data']['typedFeature'], typing['data']['type']] == [owner, target], 'typing references')
    return [n['eClass'] for n in (*nodes, typing)]


def saved(row, expected):
    source, usage, target, count = expected
    require((row['source'], row['usage'], row['target'], row['identities_requested']) == expected, 'positive case identity')
    writer, reader = row['writer'], row['reader']
    require(type(writer) is int and type(reader) is int and writer > 0 and reader > 0 and writer != reader,
            'fresh snapshot reopen processes')
    raw, reopened = row['snapshot'], row['reopen_stdout']
    require(row['snapshot_sha256'] == digest(raw.encode()) and row['reopen_sha256'] == digest(reopened.encode()) and
            reopened == raw+'\n', 'snapshot and reopen bytes')
    doc, stage = json.loads(raw), json.loads(row['stage_stdout'])
    require(stage['status'] == 'ok' and stage['document'] == doc and stage['identities_requested'] == count,
            'stage document and identities')
    require(stage['targets'] == [[{'raw': target, 'target': 'Type'}]] and len(list(walk(doc))) == count, 'typed stage target')
    encoded = source.encode()
    for lex in stage['lexemes']:
        start, end = lex['byte']
        require(0 <= start < end <= len(encoded), 'valid source span')
        literal = encoded[start:end].decode()
        offset = len(encoded[:start].decode().encode('utf-16-le'))//2
        width = len(literal.encode('utf-16-le'))//2
        require(lex['raw'] == literal and lex['utf16'] == [offset, offset+width], 'source byte and UTF-16 span')
    literals = [lex for lex in stage['lexemes'] if lex['feature'] == 'literal']
    star_offset = encoded.index(b'*')
    require(len(literals) == 1 and literals[0]['raw'] == '*' and literals[0]['kind'] == 'expression' and
            literals[0]['byte'] == [star_offset, star_offset+1], 'exact Infinity source token')
    target_id = named(doc, 'PartDefinition', target)['data']['elementId']
    path = project(named(doc, 'PartUsage', usage), target_id, 'elementId')
    return doc, path


def refused(row, source, error, count, predecessor_exact):
    require(row['source'] == source and row['snapshot_present'] is False and
            row['predecessor_exact'] is predecessor_exact, 'refusal source/atomicity/old result')
    expected = dict(error=error, status='error', identities_requested=count)
    require(json.loads(row['stage_stdout']) == expected and
            all(row[key] == value for key, value in expected.items()), 'exact refusal details')


def verify(receipt, raw):
    require(receipt['schema'] == 1 and receipt['scope'] == 'off-tree/Rust-snapshot/retained-native-projection',
            'packet scope')
    require(receipt['base'] == '05a4be35d4842272d0249ab43b9e49295591ef1e' and tuple(receipt['reference_pins']) == PINS,
            'source identities')
    require(receipt['native_release'] == '2026.9.3' and receipt['native_jar_sha256'] ==
            '85c031352695d6ac8f059f0215ca493c58a16805a111233998dd443d6990c9e1', 'reference release identity')
    attempts = receipt['native_attempts']
    require(receipt['native_fresh_case'] == 'open' and len(attempts) == 2 and
            [r['terminal'] for r in attempts] == [1, 88] and all(r['case_captured'] is False for r in attempts),
            'both unsuccessful native attempts and open case')
    require(receipt['cleanup']['inactive_scopes'] == 6 and receipt['cleanup']['case_captured'] is False,
            'isolated cleanup')
    require(receipt['regression_counts'] == COUNTS and receipt['default_aggregate_sha256'] ==
            '96e3bd0a85b8850f653192a3c18c835c8010d30461e6fc0434ca65dd9da4674c', 'regression inventory')
    require(receipt['cancellation'] == dict(error='Cancelled',status='error',identities_requested=0), 'zero-ID cancel')
    require(receipt['case_manifest_sha256'] == digest(raw), 'case manifest digest')
    entries = [json.loads(line) for line in raw.splitlines()]
    rows = {r['case']: r for r in entries}
    require(len(entries) == len(rows) and set(rows) == set(POSITIVE) | set(NEGATIVE) | {'late-wrong-type'},
            'case inventory')
    for row in rows.values():
        require(row['source_sha256'] == digest(row['source'].encode()) and
                row['stage_sha256'] == digest(row['stage_stdout'].encode()), 'source and stage digests')
    docs = {name: saved(rows[name], expected) for name, expected in POSITIVE.items()}
    delta = receipt['delta']
    previous = delta['previous_stdout']
    require(delta['source_sha256'] == rows['previously-refused']['source_sha256'] and
            delta['previous_stdout_sha256'] == digest(previous.encode()) and
            json.loads(previous) == dict(error=DENIED_INFINITY,status='error',identities_requested=0),
            'single intended prior result change')
    for name, expected in NEGATIVE.items():
        refused(rows[name], *expected, True)
    wrong = named(docs['valid-prefix'][0], 'AttributeDefinition', 'Wrong')['data']['elementId']
    refused(rows['late-wrong-type'], PREFIX.replace('[*]; }', '[*]; part bad : Wrong; }'),
            f'Model(WrongType {{ element: "{wrong}", expected: "PartDefinition" }})', 17, False)

    bat = receipt['retained_native']
    require(bat['source_line'] == 221 and bat['source_text'].strip() == 'part systems : System [*];' and
            bat['has_diagnostics'] is True, 'Batmobile projection only')
    require(bat['source_sha256'] == 'e7c7a70af9c799a0a5b9e430eec4f09fdc8d44dc623cbc8ddb3c3841e53f9bb7'
            and bat['release_capture_sha256'] == '2d4b3cc509ed35a41f879c6c3d936da64791ea3459cbd64b0721c518e2d1cc57',
            'Batmobile native provenance')
    usage, target = bat['usage'], bat['target']
    require(usage['data']['declaredName'] == 'systems' and target['declaredName'] == 'System'
            and target['eClass'] == 'sysml:PartDefinition', 'native projection usage and typing identity')
    require(project(usage, target['id'], 'id') == docs['unlimited'][1], 'Rust/native Infinity projection')
    getter = receipt['derived_bound_source']
    require(set(getter['file_sha256_by_pin']) == set(PINS) and
            len(set(getter['file_sha256_by_pin'].values())) == 1, 'derived bound source identity')


def main():
    verify(json.loads((HERE/'receipt.json').read_bytes()), (HERE/'cases.jsonl').read_bytes())
    print('1 retained native projection, 3 Rust snapshot reopens, 4 atomic refusals, fresh native open')


if __name__ == '__main__':
    main()
