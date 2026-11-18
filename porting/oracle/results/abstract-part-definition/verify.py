# SPDX-License-Identifier: EPL-2.0
"""Verify the archived abstract-definition evidence without running either importer."""

from hashlib import sha256
import json
from pathlib import Path
import re

HERE = Path(__file__).resolve().parent
SEAL = 'a54c92b25d476e28133bb3dc13395baf90e44db847637e19c8743165070cfb72'
SOURCE = 'package P { abstract part def A; }'
REFUSALS = {
    'anonymous', 'attribute', 'body', 'duplicate', 'individual', 'item', 'missing-terminator',
    'namespace-root', 'nested-nonpackage', 'part-usage', 'private', 'protected', 'public',
    'short-name', 'specialized', 'uppercase',
}


def require(condition, detail):
    if not condition:
        raise ValueError(detail)


def digest(value):
    return sha256(value if isinstance(value, bytes) else value.encode()).hexdigest()


def checked(row, field, checksum):
    value = row[field]
    require(digest(value) == row[checksum], field)
    return value


def nodes(node):
    yield node
    for key in ('ownedRelationship', 'ownedRelatedElement'):
        for child in node['data'].get(key, []):
            yield from nodes(child)


def check_ids(root, count):
    tree = tuple(nodes(root))
    ids = {node['id'] for node in tree}
    elements = {node['data']['elementId'] for node in tree}
    require(len(tree) == len(ids) == len(elements) == count and ids.isdisjoint(elements), 'identity domains')


def part(package, expected):
    require(package['eClass'] == 'sysml:Package' and package['data']['declaredName'] == 'P', 'Package P')
    membership, = package['data']['ownedRelationship']
    definition, = membership['data']['ownedRelatedElement']
    require(membership['eClass'] == 'sysml:OwningMembership'
            and definition['eClass'] == 'sysml:PartDefinition'
            and definition['data']['declaredName'] == 'A', 'owning membership and definition A')
    require('isAbstract' not in package['data'] and 'isAbstract' not in membership['data'], 'property owner')
    data = definition['data']
    require(data.get('isAbstract') is expected and ('isAbstract' in data) == (expected is True), 'field value and presence')
    return data


def no_ids(node):
    data = {field: value for field, value in node['data'].items() if field != 'elementId'}
    for containment in ('ownedRelationship', 'ownedRelatedElement'):
        if containment in data:
            data[containment] = [no_ids(child) for child in data[containment]]
    return {'eClass': node['eClass'], 'data': data}


def saved(row, count):
    source = checked(row, 'source', 'source_sha256')
    stage = json.loads(checked(row, 'stage', 'stage_sha256'))
    snapshot = checked(row, 'snapshot', 'snapshot_sha256')
    doc = json.loads(snapshot)
    package, = doc['content']
    check_ids(package, count)
    require(row['fresh_reader'] is True and digest(snapshot+'\n') == row['reopen_sha256']
            and stage['status'] == 'ok' and stage['document'] == doc
            and stage['identities_requested'] == count, 'archived staged document and reopen')
    for token in stage['lexemes']:
        start, end = token['byte']
        require(token['byte'] == token['utf16'] and source[start:end] == token['raw'], 'exact source span')
    abstract = [token for token in stage['lexemes'] if token['feature'] == 'isAbstract']
    if source == SOURCE:
        require(abstract == [dict(feature='isAbstract', kind='primitive', raw='abstract',
                                  byte=[12, 20], utf16=[12, 20])], 'exact abstract token')
    return doc


def verify_models(receipt):
    capture = receipt['native']
    require(checked(capture, 'source', 'source_sha256') == SOURCE, 'exact native source')
    golden = json.loads(checked(capture, 'golden', 'golden_sha256'))
    require(golden['id'] == 'abstract-def' and golden['reference'] == receipt['release']
            and golden['source_sha256'] == capture['source_sha256']
            and golden['result']['status'] == 'ok' and golden['result']['report'] == '', 'exact native fixture')
    native = golden['result']['document']
    namespace, = native['content']
    check_ids(namespace, 5)
    wrapped, = namespace['data']['ownedRelationship']
    package, = wrapped['data']['ownedRelatedElement']
    require(namespace['eClass'] == 'sysml:Namespace'
            and namespace['data']['eAnnotations'] == [{'source':'org.eclipse.syson.sysml.imported'}]
            and wrapped['eClass'] == 'sysml:OwningMembership', 'native import wrapper')
    part(package, True)
    positives = receipt['positives']
    require(positives['abstract-def']['source'] == SOURCE, 'exact Rust fixture')
    rust = saved(positives['abstract-def'], 3)
    part(rust['content'][0], True)
    require(native['ns'] == rust['ns'] and native['json'] == rust['json']
            and no_ids(package) == no_ids(rust['content'][0]), 'native/Rust Package structure')
    expected = dict(structural_match=True, structural_diffs=[], reference_nodes=3, rust_nodes=3,
                    wire_diffs=['import root Namespace wrapper: reference=True rust=False'])
    require(receipt['comparison'] == expected, 'recorded comparison')
    implicit = saved(positives['implicit-concrete'], 3)
    part(implicit['content'][0], None)
    del part(rust['content'][0], True)['isAbstract']
    require(rust == implicit, 'only the stored flag changes')
    prefix = saved(positives['valid-prefix'], 5)
    targets = [node for node in nodes(prefix['content'][0]) if node['eClass'] == 'sysml:AttributeDefinition']
    wrong, = targets
    require(wrong['data']['declaredName'] == 'Wrong', 'exact late wrong-type target')
    return wrong['data']['elementId']


def verify_controls(receipt, wrong):
    rows = receipt['refusals']
    require(set(rows) == REFUSALS | {'late-wrong-type'}, 'refusal scope')
    for name, row in rows.items():
        checked(row, 'source', 'source_sha256')
        stage = json.loads(checked(row, 'stage', 'stage_sha256'))
        require(row['snapshot_present'] is False and stage['status'] == 'error', name)
        if name == 'late-wrong-type':
            expected = f'Model(WrongType {{ element: "{wrong}", expected: "PartDefinition" }})'
            require(stage == dict(status='error', identities_requested=8, error=expected), 'late refusal')
        else:
            require(stage['identities_requested'] == 0 and row['predecessor_sha256'] == row['stage_sha256'], name)
    for key in ('prior_probe', 'first_attempt'):
        row = receipt[key]
        require(json.loads(checked(row,'stdout','sha256')) == dict(
            status='error', error='UnsupportedModelFeature(PartDefinition.isAbstract)', identities_requested=0), key)
    default = receipt['prior_default']
    require(json.loads(checked(default,'stdout','sha256')) ==
            dict(status='error',error='Parse(Unsupported { offset: 12 })'), 'exact default refusal')
    require(receipt['cancellation'] == dict(status='error',error='Cancelled',identities_requested=0), 'cancel before allocation')
    require(receipt['scope'] == dict(archived_runtime_receipt=True,default_adopted=False,
        syntax_validation_qualified=False,abstract_instantiation_qualified=False,application_integration_qualified=False),
        'limited acceptance scope')


def verify():
    data = (HERE/'receipt.json').read_bytes()
    require(digest(data) == SEAL, 'receipt digest')
    receipt = json.loads(data)
    replay = (HERE/'replay.jsonl').read_bytes()
    require(digest(replay) == receipt['replay']['sha256'], 'replay digest')
    rows = [json.loads(line) for line in replay.splitlines()]
    require(len(rows) == len({row['case'] for row in rows}) == receipt['replay']['count'] == 283, 'incumbent count')
    require(sum(row['family'] == 'default' for row in rows) == receipt['replay']['default_count'] == 64,
            'default corpus count')
    for row in rows:
        require(row['family'] in ('default','prototype') and all(re.fullmatch('[0-9a-f]{64}',row[key])
                for key in ('source_sha256','stdout_sha256')), 'incumbent row shape')
    verify_controls(receipt, verify_models(receipt))
    print('PASS: three-node Package, 3 archived fresh reopens, 283 incumbents and 16 unchanged refusals')


if __name__ == '__main__':
    verify()
