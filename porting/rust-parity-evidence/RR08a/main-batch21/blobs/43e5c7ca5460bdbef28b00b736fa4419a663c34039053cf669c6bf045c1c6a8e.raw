#!/usr/bin/env python3
"""Verify pinned retained source, extract declaration tables and static fixtures.

Never executes JavaScript, parser, compiler, remote source or an upstream script.
Fixture sort uses CPython's stable sort with the source partial comparator; it
is a source-derived candidate ordering, not a JavaScript-runtime observation.
"""
import functools
import hashlib
import json
import pathlib
import sys

CRATE = pathlib.Path(__file__).resolve().parents[1]
ROOT = CRATE.parents[2]
PREP = ROOT / 'porting/ast/reflection-preparation'
EXPECTED = {
    'packages/syside-languageserver/src/generated/ast.ts': '982a5b2415abbc4ad8f29cab3facbf5fbe29a9dc7cc7bde42a58ad0849de9c02',
    'packages/syside-languageserver/src/model/types.ts': '61c221ebe23bee6f61757a72f515ce2aeafb111a8bbf3cc6fce9c3a9415c4f3c',
    'packages/syside-languageserver/src/services/sysml-ast-reflection.ts': '9f43aba6051f1ef68b36730212e5ebdbb906816fac746a644be7cd6e89c9607b',
    'packages/syside-languageserver/src/model/__tests__/type-index.test.ts': 'd2e1acbb3888d0d9d6f95170e6975ef9b7ce2c8338fed76467d1e86d51698a15',
}
INPUT_HASHES = {
    'generated-type-declarations.jsonl': '42c01266563bc83917f4040ce8ec41397f420b58a663a62ec1283394da131fb0',
    'summary.json': '29e49aeb680d84783a94f472e48cbe125bfd95d9af0da23fbc567deb6c53d1d2',
}

def main():
    sources = {d['path']: d for d in map(json.loads, (PREP / 'candidate-source-documents.jsonl').read_text().splitlines())}
    for path, expected in EXPECTED.items():
        doc = sources[path]
        assert hashlib.sha256(doc['source'].encode()).hexdigest() == expected == doc['sha256'], path
    for path, expected in INPUT_HASHES.items():
        assert hashlib.sha256((PREP / path).read_bytes()).hexdigest() == expected, path
    rows = list(map(json.loads, (PREP / 'generated-type-declarations.jsonl').read_text().splitlines()))
    summary = json.loads((PREP / 'summary.json').read_text())
    names = [r['name'] for r in rows]
    assert len(names) == len(set(names)) == 189 and names == summary['generated_type_order']
    direct = {r['name']: r['generated_direct_supertypes'] for r in rows}
    assert set(p for parents in direct.values() for p in parents) <= set(names)
    def ancestors(name, stack=()):
        assert name not in stack, ('cyclic source inheritance', name)
        found = set()
        for parent in direct[name]:
            found.add(parent)
            found.update(ancestors(parent, stack + (name,)))
        return found
    closure = {n: ancestors(n) for n in names}
    non_types = {r['name'] for r in rows if r['filtered_by_NON_TYPES']}
    assert non_types == set(summary['NON_TYPES']) and len(non_types) == 5
    def subtype(a, b):
        return a == b or b in closure[a]
    def compare(a, b):
        return -1 if subtype(a, b) else 1 if subtype(b, a) else 0
    cmp_key = functools.cmp_to_key(compare)
    orders, partitions = {}, []
    for name in names:
        groups = [[], [], []]
        for parent in names:
            if parent in closure[name]:
                groups[0 if parent in non_types else 1 if subtype(parent, 'Namespace') else 2].append(parent)
        partitions.append({'type': name, 'unions': groups[0], 'namespace_types': groups[1], 'relationships': groups[2]})
        unions, types, relationships = [sorted(group, key=cmp_key) for group in groups]
        for other in [relationships, unions]:
            positions = [(next((i for i, existing in enumerate(types) if not subtype(existing, item)), len(types)), item) for item in other]
            for inserted, (position, item) in enumerate(sorted(positions, key=lambda p: p[0])):
                types.insert(position + inserted, item)
        assert set(types) == closure[name] and len(types) == len(closure[name])
        orders[name] = types
    strict = {
        'Element': [],
        'Comment': ['TextualAnnotatingElement', 'AnnotatingElement', 'Element'],
        'Feature': ['Type', 'Namespace', 'Element'],
        'OperatorExpression': ['InvocationExpression', 'InlineExpression', 'Expression', 'Step', 'Feature', 'Type', 'Namespace', 'Element'],
    }
    for name, expected in strict.items():
        assert orders[name] == expected, name
    for row in rows:
        local = row['generated_local_mandatory']
        assert len({p['name'] for p in local}) == len(local)
        assert all(p['type'] in ['array', 'boolean'] for p in local)
    def metadata(name):
        result = []
        for owner in [name] + orders[name]:
            for prop in rows[names.index(owner)]['generated_local_mandatory']:
                if not any(p[0] == prop['name'] for p in result):
                    result.append((prop['name'], prop['type'], owner))
        result.append(('$children', 'array', '-'))
        return result
    generated = ['// Copyright (c) 2022-2025 Sensmetry UAB and others.',
                 '// SPDX-License-Identifier: EPL-2.0 OR GPL-2.0 WITH Classpath-exception-2.0',
                 '// Generated from hash-verified retained declarations by tools/generate.py.',
                 'use super::{MandatoryKind, Row};', '#[rustfmt::skip]', 'pub(super) static ROWS: &[Row] = &[']
    for r in rows:
        parents = ', '.join(str(names.index(p)) for p in r['generated_direct_supertypes'])
        props = ', '.join('(' + json.dumps(p['name']) + ', MandatoryKind::' + p['type'].title() + ')' for p in r['generated_local_mandatory'])
        generated.append('    Row { name: ' + json.dumps(r['name']) + ', parents: &[' + parents + '], non_type: ' + str(r['filtered_by_NON_TYPES']).lower() + ', mandatory: &[' + props + '] },')
    generated += ['];', '#[rustfmt::skip]', 'pub(super) static REFERENCE_IDS: &[&str] = &[']
    generated += ['    ' + json.dumps(s) + ',' for s in summary['generated_reference_ids']]
    generated += ['];', '']
    fixture = []
    for n in names:
        subtypes = [candidate for candidate in names if n in closure[candidate]]
        props = [':'.join(p) for p in metadata(n)]
        fixture.append('\t'.join([n, ','.join(orders[n]), ','.join(subtypes), ','.join(props)]))
    maximum = max(len(p[k]) for p in partitions for k in ['unions', 'namespace_types', 'relationships'])
    assert maximum == 13
    report = {
        'source_baseline': summary['source_baseline'], 'source_candidate': summary['source_candidate'],
        'retained_source_sha256': EXPECTED, 'prepared_input_sha256': INPUT_HASHES,
        'generated_names': 189, 'filtered_ast_names': 184, 'maximum_sort_partition': maximum,
        'ordering_policy': 'SOURCE_DERIVED_STABLE_SHORT_SORT_CANDIDATE',
        'ordering_limits': 'Partial subtype comparator is not a strict weak order. CPython fixture sort and Rust explicit stable short-sort agreement is static cross-check evidence, not proof of original JavaScript order.',
        'strict_source_chain_expectations': strict, 'input_partitions': partitions,
        'upstream_runtime': 'NOT_RUN', 'upstream_order_parity_verified': False,
    }
    outputs = {CRATE / 'src/generated.rs': '\n'.join(generated), CRATE / 'tests/source_tables.tsv': '\n'.join(fixture) + '\n', ROOT / 'porting/follow-ons/F08/source-mapping.json': json.dumps(report, indent=2) + '\n'}
    check = '--check' in sys.argv
    for path, text in outputs.items():
        if check:
            assert path.read_text() == text, ('generated output drift', str(path))
        else:
            path.write_text(text)
    print(json.dumps({'mode': 'check' if check else 'write', 'types': len(names), 'max_partition': maximum, 'strict_source_expectations': len(strict), 'upstream_runtime': 'NOT_RUN'}))

if __name__ == '__main__':
    main()
