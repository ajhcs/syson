#!/usr/bin/env python3
"""Verify the upper-unbounded cardinality evidence without executing either importer."""
from hashlib import sha256
import importlib.util
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
SOURCE = 'package P { part def W; part def C { part w : W[1..*]; } }'
PREFIX = 'package P { part def W; attribute def Wrong; part def C { part good : W[1..*]; } }'
INFIX_REFUSAL = 'UnsupportedInfixOperator(..)'
POSITIVES = {
    'upper-unbounded': (SOURCE, 'w', 'W', [1, '*'], 14),
    'nested-zero': (SOURCE.replace('1..*', '0..*'), 'w', 'W', [0, '*'], 14),
    'top-level-zero': ('package P { part def System; part systems : System [0..*]; }',
                       'systems', 'System', [0, '*'], 12),
    'valid-prefix': (PREFIX, 'good', 'W', [1, '*'], 16),
}
REFUSALS = {
    name: SOURCE.replace('1..*', form) for name, form in (
        ('mixed-lower', '*..4'), ('both-stars', '*..*'), ('open-upper', '1..'), ('open-lower', '..*'),
        ('signed', '-1..*'), ('rational', '1.0..*'), ('expression', '1+1..*'), ('overflow', '2147483648..*'),
    )
}
REFUSALS['feature-range'] = 'package P { attribute def N; part def C { attribute n : N = 1..*; } }'
REFUSALS['late-wrong-type'] = PREFIX.replace('[1..*]; }', '[1..*]; part bad : Wrong; }')
COUNTS = {
    'cardinality-refusal': 5, 'cardinality-success': 5, 'default': 62, 'nonunique-refusal': 1,
    'nonunique-success': 4, 'null-refusal': 2, 'null-success': 8, 'prior-refusal': 13,
    'prior-success': 105, 'unlimited-refusal': 3, 'unlimited-success': 3,
    'finite-range-success': 4, 'finite-range-refusal': 9,
}


def require(ok, reason):
    if not ok:
        raise ValueError(reason)


def digest(data):
    return sha256(data).hexdigest()


def walk(value, classifier='eClass'):
    if isinstance(value, dict):
        if classifier in value:
            yield value
        for child in value.values():
            yield from walk(child, classifier)
    elif isinstance(value, list):
        for child in value:
            yield from walk(child, classifier)


def named(doc, kind, name):
    matches = [node for node in walk(doc) if node['eClass'] == 'sysml:'+kind and
               node['data'].get('declaredName') == name]
    require(len(matches) == 1, f'exact {kind} {name}')
    return matches[0]


def child(node, kind, field):
    require(node['eClass'] == 'sysml:'+kind, f'{kind} containment class')
    children = node['data'][field]
    require(len(children) == 1, f'{kind} one owned child')
    return children[0]


def shape(doc, usage, values, target="W", native=False):
    part = named(doc, 'PartUsage', usage)
    typ = named(doc, 'PartDefinition', target)
    fields = part['data']
    require(fields['isComposite'] is True and 'isUnique' not in fields, 'untouched usage flags')
    relations = fields['ownedRelationship']
    require(len(relations) == 2, 'range precedes typing')
    outer, typing = relations
    range_node = child(outer, 'OwningMembership', 'ownedRelatedElement')
    require(range_node['eClass'] == 'sysml:MultiplicityRange' and
            set(range_node['data']) == {'elementId', 'ownedRelationship'}, 'range without derived fields')
    members = range_node['data']['ownedRelationship']
    require(len(members) == 2, 'two bound memberships')
    bounds = [child(item, 'OwningMembership', 'ownedRelatedElement') for item in members]
    lower, upper = bounds
    data = lower['data']
    require(lower['eClass'] == 'sysml:LiteralInteger' and
            set(data) == ({'elementId', 'value'} if values[0] else {'elementId'}) and
            type(data.get('value', 0)) is int and data.get('value', 0) == values[0], 'stored integer lower bound')
    require(values[1] == '*' and upper['eClass'] == 'sysml:LiteralInfinity' and
            set(upper['data']) == {'elementId'}, 'stored Infinity without numeric sentinel')
    nodes = [outer, range_node, *members, *bounds]
    resources = {node['id'] for node in nodes}
    semantics = {node['data']['elementId'] for node in nodes}
    require(len(resources) == len(semantics) == 6 and resources.isdisjoint(semantics), 'six paired identities')
    identifiers = [node['id'] if native else node['data']['elementId'] for node in (part, typ)]
    require(typing['eClass'] == 'sysml:FeatureTyping' and
            [typing['data']['typedFeature'], typing['data']['type']] == identifiers, 'resolved feature typing')
    require(all(n['eClass'] != 'sysml:OperatorExpression' for n in walk(doc)), 'range operator not persisted')
    return part, typ, range_node, bounds


def reopened(row, expected):
    source, usage, target, values, count = expected
    require((row['source'], row['usage'], row['target'], row['values'], row['identities_requested']) == expected,
            'positive source and model inventory')
    writer, reader = row['writer'], row['reader']
    require(type(writer) is int and type(reader) is int and writer > 0 and reader > 0 and writer != reader,
            'separate save and reopen processes')
    snapshot, reopen = row['snapshot'], row['reopen_stdout']
    require(digest(snapshot.encode()) == row['snapshot_sha256'] and digest(reopen.encode()) == row['reopen_sha256']
            and reopen == snapshot+'\n', 'stored and reopened bytes')
    doc, stage = json.loads(snapshot), json.loads(row['stage_stdout'])
    require(stage['status'] == 'ok' and stage['document'] == doc and stage['identities_requested'] == count
            and len(list(walk(doc))) == count, 'stage and model identities')
    require(stage['infix_actions'] == 1 and stage['targets'] == [[{'raw': target, 'target': 'Type'}]], 'infix and typing')
    encoded = source.encode()
    tokens = []
    for lex in stage['lexemes']:
        first, last = lex['byte']
        require(0 <= first < last <= len(encoded), 'byte span bounds')
        text = encoded[first:last].decode()
        offset = len(encoded[:first].decode().encode('utf-16-le'))//2
        require(lex['raw'] == text and lex['utf16'] == [offset, offset+len(text.encode('utf-16-le'))//2],
                'exact byte and UTF-16 spans')
        if lex['feature'] in ('operator', 'literal'):
            tokens.append(text)
    require(tokens == [str(values[0]), '..', '*'], 'left/operator/right source order')
    star, = [lex for lex in stage['lexemes'] if lex['raw'] == '*' and lex['feature'] == 'literal']
    require(star['kind'] == 'expression' and star['byte'] == [encoded.index(b'*'),encoded.index(b'*')+1],
            'exact Infinity span and role')
    shape(doc, usage, values, target)
    return doc


def verify(receipt, raw, compare):
    require(receipt['schema'] == 1 and receipt['scope'] == 'off-tree/Rust-and-exact-release-converter', 'packet scope')
    require(receipt['base'] == '05a4be35d4842272d0249ab43b9e49295591ef1e' and
            receipt['source_pin'] == '6ba2ac64e6b16ee2b05752ad55395eebb87d4910', 'source identity')
    require(receipt['release'] == {
        'version': '2026.9.3', 'source': 'b380dada415ce05155150f23cb3b74ae21cc45e4',
        'jar_sha256': '85c031352695d6ac8f059f0215ca493c58a16805a111233998dd443d6990c9e1',
    }, 'pinned release')
    app = receipt['app_http_model']
    require(app == dict(status='not run',case_captured=False),
            'no HTTP capture claim')
    require(receipt['regression_counts'] == COUNTS and receipt['default_aggregate_sha256'] ==
            '96e3bd0a85b8850f653192a3c18c835c8010d30461e6fc0434ca65dd9da4674c', 'default and prior regression inventory')
    require(receipt['cases_sha256'] == digest(raw), 'case manifest digest')
    entries = [json.loads(line) for line in raw.splitlines()]
    rows = {row['case']: row for row in entries}
    require(len(entries) == len(rows) and set(rows) == set(POSITIVES) | set(REFUSALS), 'case inventory')
    for row in rows.values():
        require(digest(row['source'].encode()) == row['source_sha256'] and
                digest(row['stage_stdout'].encode()) == row['stage_sha256'], 'source and stage bytes')
    docs = {name: reopened(rows[name], expected) for name, expected in POSITIVES.items()}
    historical = receipt['historical_zero_changes']
    require(set(historical) == {'nested-zero', 'top-level-zero'}, 'exact two historical changes')
    for name, delta in historical.items():
        row = rows[name]
        old = row['predecessor_stdout']
        require(row['source_sha256'] == delta['source_sha256'] and
                digest(old.encode()) == delta['previous_stdout_sha256'] and
                json.loads(old) == dict(status='error',error=INFIX_REFUSAL,identities_requested=0),
                'stored predecessor refusal and local zero-bound acceptance')

    for name, source in REFUSALS.items():
        row = rows[name]
        require(row['source'] == source and row['snapshot_present'] is False, 'atomic refusal source')
        stage = json.loads(row['stage_stdout'])
        require(stage == {field: row[field] for field in ['status','error','identities_requested']} and
                stage['status'] == 'error', 'refusal result')
        if name == 'late-wrong-type':
            require(row['comparison'] == 'local-only' and 'predecessor_stdout' not in row,
                    'late atomic control qualification scope')
            wrong = named(docs['valid-prefix'], 'AttributeDefinition', 'Wrong')['data']['elementId']
            require(stage['identities_requested'] == 19 and
                    stage['error'] == f'Model(WrongType {{ element: "{wrong}", expected: "PartDefinition" }})',
                    'late typing error')
        else:
            prior = row['predecessor_stdout']
            require(row['comparison'] == 'byte-exact' and digest(prior.encode()) == row['predecessor_sha256'] and
                    prior == row['stage_stdout'] and stage['identities_requested'] == 0, 'preserved refusal bytes')
            if name.startswith('open'):
                require(stage['error'].startswith('Unexpected { span:'), 'recognizer rejects open range')
            else:
                require(stage['error'] == INFIX_REFUSAL, 'unsupported range scope')
    require(receipt['cancellation'] == dict(status='error',error='Cancelled',identities_requested=0), 'cancel result')

    ast = receipt['upstream_ast']
    require(digest(ast['java_input'].encode()) == ast['sha256'], 'exact Java AST bytes')
    tree = json.loads(ast['java_input'])
    require(tree['$cstNode']['text'] == SOURCE, 'AST source bytes')
    part, = (node for node in walk(tree, '$type') if node['$type'] == 'PartUsage' and node['declaredName'] == 'w')
    outer = part['multiplicity']
    rng = outer['target']
    member = rng['range']
    operator = member['target']
    require([n['$type'] for n in (outer,rng,member,operator)] ==
            ['OwningMembership','MultiplicityRange','OwningMembership','OperatorExpression'], 'AST range path')
    require(rng['$cstNode']['text'] == '[1..*]' and operator['$cstNode']['text'] == '1..*' and
            operator['operator'] == '..', 'AST range source')
    require([(n['$type'],n.get('literal'),n['$cstNode']['text']) for n in operator['operands']] ==
            [('LiteralNumber',1,'1'),('LiteralInfinity',None,'*')], 'ordered AST operands')

    converter = receipt['converter']
    require(converter['scope'] == 'one release AST and ASTTransformer.convertResource without database' and converter['terminal'] == 0 and
            converter['classpath_entries'] == 312 and converter['loaded_standard_libraries'] == 94,
            'exact converter scope and inputs')
    for name, key in [('model_emf','model_sha256'),('facts_tsv','facts_sha256'),('messages_tsv','messages_sha256')]:
        require(digest(converter[name].encode()) == converter[key], 'converter artifact bytes')
    require(converter['messages_tsv'] == '', 'converter emitted no messages')
    native = json.loads(converter['model_emf'])
    part, typ, rng, bounds = shape(native, 'w', [1,'*'], native=True)
    facts = {row[0]: row[1:] for row in (line.split('\t') for line in converter['facts_tsv'].splitlines())}
    require(facts['library_resources'] == ['94'] and facts['imported_nodes'] == ['16'] and
            facts['messages'] == ['0'], 'native converter counts')
    for key,node,labels in [('part_usage',part,['w']),('type',typ,['W']),('range',rng,[]),
                            ('lower',bounds[0],['1']),('upper',bounds[1],['*'])]:
        require(facts[key] == labels+[node['id'],node['data']['elementId']], 'converter object identities')

    namespace, = native['content']
    require(namespace['eClass'] == 'sysml:Namespace' and set(namespace['data']) == {'elementId','ownedRelationship'},
            'exact unannotated converter Namespace')
    outer, = namespace['data']['ownedRelationship']
    require(set(outer['data']) == {'elementId','ownedRelatedElement'}, 'projection preserves wrapper fields')
    package = child(outer,'OwningMembership','ownedRelatedElement')
    require(package['eClass'] == 'sysml:Package' and package['data']['declaredName'] == 'P', 'projected Package')
    wire = ["cross-references point at: reference=['id'] rust=['elementId']"]
    reference = docs['upper-unbounded']
    comparison = receipt['comparison']
    require(compare(native,reference) == comparison['raw'] == dict(structural_match=False,
        structural_diffs=[dict(path='$[0]',reference='sysml:Namespace',rust='sysml:Package')],
        wire_diffs=wire,reference_nodes=16,rust_nodes=14), 'explicit raw Namespace difference')
    require(compare(dict(native,content=[package]),reference) == comparison['projected'] ==
        dict(structural_match=True,structural_diffs=[],wire_diffs=wire,reference_nodes=14,rust_nodes=14),
        '14-node checked Package projection')


def main():
    spec = importlib.util.spec_from_file_location('compare', HERE.parents[1]/'compare.py')
    comparison = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(comparison)
    verify(json.loads((HERE/'receipt.json').read_bytes()), (HERE/'cases.jsonl').read_bytes(), comparison.compare)
    print('1 exact converter projection, 4 Rust reopens, 2 historical deltas, 10 atomic refusals')


if __name__ == '__main__':
    main()
