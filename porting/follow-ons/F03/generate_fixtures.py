#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Translate the frozen F02 inputs to typed Rust data; never derive expected results."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]
SOURCE = ROOT / 'porting/area-inventories/DOMAIN-QUERY/imports/fixtures.json'
OUTPUT = ROOT / 'rust/crates/syson-imports/tests/data/f02_cases.rs'
PIN = '9105f54c8e560dd8b31f5a020d58c6eea3b54a51a387439f3bc1fc89c44636c1'
raw = SOURCE.read_bytes()
if hashlib.sha256(raw).hexdigest() != PIN:
    raise ValueError('F02 fixture bytes changed')
x = json.loads(raw)

def text(s):
    # Frozen fixtures use ASCII; refuse silently changing Rust string semantics.
    if not s.isascii():
        raise ValueError('Non-ASCII fixture text needs explicit Rust escaping')
    return json.dumps(s)

def mid(s):
    return 'm(' + text(s) + ')'

def nid(s):
    return 'n(' + text(s) + ')'

def ids(values, fn=mid):
    return 'vec![' + ','.join(map(fn, values)) + ']'

def option(s):
    return 'None' if s is None else 'Some(' + text(s) + '.into())'

members = []
for key, member in x['members'].items():
    element = member['element']
    target = 'Target::Absent' if element is None else ('Target::Namespace(' + nid(element) + ')' if element in x['namespaces'] else 'Target::Element')
    alias = 'Alias::None' if member['member_name'] is None else ('Alias::SameReference' if member['same_name_reference'] else 'Alias::Distinct(' + text(member['member_name']) + '.into())')
    members.append('(' + mid(key) + ', Known(Member { target: Known(' + target + '), alias: Known(' + alias + '), effective_name: Known(' + option(member['effective_name']) + '), effective_short_name: Known(' + option(member['effective_short_name']) + ') }))')
lines = ['// Generated from the immutable F02 JSON by generate_fixtures.py.', '// Fixture SHA256: ' + PIN, 'fn f02_members() -> Vec<(MemberId, Observation<Member>)> { vec![' + ','.join(members) + '] }', 'fn f02_cases() -> Vec<Case> { vec![']
for case in x['cases']:
    operation = case['operation']
    nodes = []
    for key, node in case.get('graph', {}).items():
        owned = 'vec![' + ','.join('(' + mid(o['membership']) + ', Visibility::' + o['visibility'].title() + ')' for o in node['owned']) + ']'
        imports = 'vec![' + ','.join('(Import::Namespace { target: Known(Some(' + nid(i['target']) + ')), recursive: false, include_all: false }, Visibility::' + i['visibility'].title() + ')' for i in node['imports']) + ']'
        nodes.append('(' + nid(key) + ', Known(Namespace { is_type: false, owned: ' + owned + ', imports: ' + imports + ' }))')
    if operation == 'namespace_pipeline':
        calls = 'vec![' + ','.join('Known(Some(' + ids(c['returned']) + '))' for c in case['calls']) + ']'
        query = 'Query::Supplied { owned: ' + ids(case['owned']) + ', calls: ' + calls + ' }'
    elif operation == 'filter_trace':
        query = 'Query::Filter { seed: ' + ids(case['seed_names'], lambda s: text(s) + '.into()') + ', members: ' + ids([s['membership'] for s in case['filter_steps']]) + ' }'
    elif operation == 'graph_trace':
        q = case['request']
        flags = ', recursive: ' + str(q['recursive']).lower() + ', include_all: ' + str(q['include_all']).lower()
        if q['entrypoint'] == 'Namespace.visibleMemberships':
            query = 'Query::Visible { namespace: ' + nid(q['namespace']) + flags + ' }'
        else:
            kind = q['kind'].title()
            target = nid(q['imported_namespace']) if kind == 'Namespace' else mid(q['membership'])
            query = 'Query::Import(Import::' + kind + ' { target: Known(Some(' + target + '))' + flags + ' })'
    elif operation == 'unresolved':
        nodes = [('(' + nid('A') + ', Observation::Unresolved)')]
        query = 'Query::Visible { namespace: n("A"), recursive: false, include_all: false }'
    else:
        query = 'Query::MembershipsOfVisibility'
        if case['id'] == 'base-import-null-then-flatten':
            nodes = ['(n("A"), Known(Namespace { is_type: false, owned: vec![], imports: vec![(Import::BaseNull, Visibility::Public)] }))']
            query = 'Query::Imported(n("A"))'
    expected = case['expected']
    if case['id'] == 'visibility-include-all':
        result = 'Err(Error::UnsupportedIncludeAll)'
    elif expected['status'] == 'MEMBERSHIPS':
        result = 'Ok(Outcome::Memberships(' + ids(expected['memberships']) + '))'
    elif expected['status'] == 'NULL_UNIMPLEMENTED':
        result = 'Ok(Outcome::NullUnimplemented)'
    else:
        result = 'Err(Error::' + ('Unresolved' if operation == 'unresolved' else 'NullImport') + ')'
    lines.append('Case { label: ' + text(case['id']) + ', snapshot: Snapshot::new(vec![' + ','.join(nodes) + '], f02_members()).unwrap(), query: ' + query + ', excluded: ' + ids(case.get('excluded', case.get('request', {}).get('excluded', [])), nid) + ', expected: ' + result + ' },')
lines.append('] }')
# rustfmt is formatting only; no Cargo build, macro execution or native oracle.
child = subprocess.run(['rustfmt', '--edition', '2024', '--emit', 'stdout'], input=('\n'.join(lines) + '\n').encode(), capture_output=True, check=True)
if '--check' in sys.argv:
    if OUTPUT.read_bytes() != child.stdout:
        raise ValueError('Generated Rust fixture drift')
else:
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_bytes(child.stdout)
print(json.dumps({'status': 'PASS', 'translated_f02_cases': len(x['cases']), 'include_all_disposition': 'EXPLICITLY_UNSUPPORTED', 'upstream_runtime': 'NOT_RUN'}))
