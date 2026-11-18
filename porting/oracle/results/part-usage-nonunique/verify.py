#!/usr/bin/env python3
"""Verify captured PartUsage uniqueness models without launching either importer."""
from hashlib import sha256
import importlib.util
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
REFERENCE = {
    'nonunique': (False, 'c6311065059390072888f44031281e7394bdaffc02d138ae1f9a11be1bbe562a'),
    'isUniqueFeature-model': (False, '802aec2a6543609c549b298113899d8aa74f999fae121b1739c9f72d93b89348'),
    'p01-basic-typed-part': (True, 'cba148e46f0c9e6afd9be8fb35563bd99387ace6e7473cef83370e5f06b7597d'),
}
COUNTS = {'default': 62, 'null-refusal': 2, 'null-success': 8, 'prior-refusal': 13, 'prior-success': 105}
WIRE = [
    'import root Namespace wrapper: reference=True rust=False',
    "cross-references point at: reference=['id'] rust=['elementId']",
]


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(data):
    return sha256(data).hexdigest()


def walk(value):
    if isinstance(value, dict):
        if 'eClass' in value:
            yield value
        for child in value.values():
            yield from walk(child)
    elif isinstance(value, list):
        for child in value:
            yield from walk(child)


def model(document, unique, id_field):
    usage, = (node for node in walk(document) if node['eClass'] == 'sysml:PartUsage')
    definition, = (node for node in walk(document) if node['eClass'] == 'sysml:PartDefinition')
    fields = usage['data']
    require(fields['isComposite'] is True, 'composition flag')
    if unique:
        require('isUnique' not in fields, 'true default must be omitted')
    else:
        require(fields.get('isUnique') is False, 'nonunique must be explicitly false')
    typing, = fields['ownedRelationship']
    require(typing['eClass'] == 'sysml:FeatureTyping', 'owned typing class')
    if id_field == 'id':
        type_id, feature_id = definition['id'], usage['id']
    else:
        type_id, feature_id = definition['data']['elementId'], fields['elementId']
    require(typing['data']['type'] == type_id and typing['data']['typedFeature'] == feature_id, 'typing links')
    return usage


def stage(row, unique, identity_count):
    source = row['source'].encode()
    raw = row['snapshot']
    require(digest(raw.encode()) == row['snapshot_sha256'], 'snapshot digest')
    require(row['reopen_stdout'] == raw + '\n', 'reopen bytes')
    writer, reader = row['snapshot_processes']
    require(type(writer) is int and type(reader) is int and writer > 0 and reader > 0 and writer != reader, 'fresh reopen processes')
    staged = json.loads(row['stage_stdout'])
    require(staged['status'] == 'ok' and staged['identities_requested'] == identity_count, 'stage status and identities')
    document = json.loads(raw)
    require(staged['document'] == document, 'stage and persisted model')
    observed = []
    for lexeme in staged['lexemes']:
        start, end = lexeme['byte']
        require(0 <= start < end <= len(source), 'source span bounds')
        literal = source[start:end].decode()
        offset = len(source[:start].decode().encode('utf-16-le')) // 2
        end_offset = offset + len(literal.encode('utf-16-le')) // 2
        require(literal == lexeme['raw'] and lexeme['utf16'] == [offset, end_offset], 'source span bytes')
        if lexeme['feature'] == 'isNonunique':
            observed.append(literal)
    require(observed == ([] if unique else ['nonunique']), 'nonunique lexical slot')
    model(document, unique, 'elementId')
    return document


def verify(receipt, raw, compare):
    require(receipt['schema'] == 1 and receipt['scope'] == 'retained-reference/off-tree-rust/no-default-activation', 'packet scope')
    require(receipt['base'] == '05a4be35d4842272d0249ab43b9e49295591ef1e', 'Rust base')
    require(receipt['pinned_source'] == '6ba2ac64e6b16ee2b05752ad55395eebb87d4910', 'baseline source identity')
    release = receipt['reference_release']
    require(release == {
        'version': '2026.9.3', 'source': 'b380dada415ce05155150f23cb3b74ae21cc45e4',
        'jar_sha256': '85c031352695d6ac8f059f0215ca493c58a16805a111233998dd443d6990c9e1',
        'captured_previously': True,
    }, 'reference release identity')
    require(receipt['regression_counts'] == COUNTS, 'regression inventory')
    require(receipt['default_aggregate_sha256'] == '96e3bd0a85b8850f653192a3c18c835c8010d30461e6fc0434ca65dd9da4674c', 'default output digest')
    require([run['terminal'] for run in receipt['build_attempts']] == [88, 0], 'failed and successful build history')
    require(digest(raw) == receipt['cases_sha256'], 'case manifest digest')
    entries = [json.loads(line) for line in raw.splitlines()]
    rows = {row['case']: row for row in entries}
    require(len(entries) == len(rows) and set(rows) == set(REFERENCE) | {'valid-prefix', 'late-wrong-type'}, 'case inventory')
    for row in rows.values():
        require(digest(row['source'].encode()) == row['source_sha256'], 'source bytes')

    for name, (unique, capture) in REFERENCE.items():
        row = rows[name]
        require(row['role'] == 'reference' and row['native_capture_sha256'] == capture, 'retained capture pin')
        native = row['reference_document']
        model(native, unique, 'id')
        candidate = stage(row, unique, 6)
        result = compare(native, candidate)
        require(result == row['comparison'] == {
            'reference_nodes': 6, 'rust_nodes': 6, 'structural_match': True,
            'structural_diffs': [], 'wire_diffs': WIRE,
        }, 'normalized reference comparison')

    upstream = receipt['upstream_fixture']
    require(upstream['source_sha256'] == rows['isUniqueFeature-model']['source_sha256'], 'pinned upstream fixture digest')
    prefix = rows['valid-prefix']
    require(prefix['role'] == 'control' and prefix['source'].endswith('part good nonunique : T; }'), 'valid prefix')
    document = stage(prefix, False, 8)
    wrong, = (node for node in walk(document) if node['eClass'] == 'sysml:AttributeDefinition')
    require(wrong['data']['declaredName'] == 'Wrong', 'wrong target present as unused prefix declaration')
    refusal = rows['late-wrong-type']
    require(refusal['role'] == 'refusal' and refusal['snapshot_present'] is False, 'atomic refusal and snapshot absence')
    require(refusal['source'] == prefix['source'][:-1] + 'part bad nonunique : Wrong; }', 'late invalid addition')
    require(json.loads(refusal['stage_stdout']) == {
        'status': 'error', 'identities_requested': 11,
        'error': f'Model(WrongType {{ element: "{wrong["data"]["elementId"]}", expected: "PartDefinition" }})',
    }, 'late typing error')


def main():
    spec = importlib.util.spec_from_file_location('compare', HERE.parents[1] / 'compare.py')
    comparison = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(comparison)
    verify(json.loads((HERE/'receipt.json').read_bytes()), (HERE/'cases.jsonl').read_bytes(), comparison.compare)
    print('3 retained reference matches, 1 valid prefix, 1 atomic refusal')


if __name__ == '__main__':
    main()
