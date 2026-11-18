# SPDX-License-Identifier: EPL-2.0
"""Verify the scoped type-recovery observation without executing Rust or SysON."""
from hashlib import sha256
import json
from pathlib import Path
import re

PIN = '8cb7dfaaa10693326f0edb87a86d182109b97518f12b132615bbd018cde61b2c'
CASES = {
    'local-private': (['A'], 'private', 'A::T', None, 6),
    'missing': (['A', 'B'], None, None, 'A::Missing', 7),
    'peer-private': (['A', 'B'], 'private', None, 'A::T', 7),
    'peer-public': (['A', 'B'], None, 'A::T', None, 7),
}
FATAL = {
    'duplicate-root': ('Link', 6), 'late-wrong-type': ('Model', 12),
    'malformed': ('Unexpected', 0), 'missing-prefix': ('Link', 6),
    'multiplicity': ('Link', 10), 'nested-packages': ('Link', 9),
    'part-definition-owner': ('Link', 8), 'quoted-terminal': ('Link', 6),
    'same-package-missing': ('Link', 5),
}


def require(ok, detail):
    if not ok:
        raise ValueError(detail)


def hash_text(value):
    return isinstance(value, str) and re.fullmatch('[0-9a-f]{64}', value) is not None


def walk(node):
    yield node
    for key in ('ownedRelationship', 'ownedRelatedElement'):
        for child in node['data'].get(key, []):
            yield from walk(child)


def ref(node):
    return 'ref(' + node['#'] + ')'


def check(data):
    require(data['reference_release'] == '2026.9.3'
            and data['native_comparison_commit'] == 'c99ea8c912bcd8563605b4b5b6f30754cd9a8f25'
            and data['default_adopted'] is False and data['opt_in_controls'] == 236,
            'release and opt-in scope')
    require(data['early_cancellation'] == {'status':'error','error':'Cancelled','identities_requested':0},
            'cancellation before allocation')
    require(set(data['cases']) == set(CASES) and set(data['fatal']) == set(FATAL),
            'fixed positive and fatal cases')
    for name, (owners, visibility, target, unresolved, count) in CASES.items():
        row = data['cases'][name]
        require(sha256(row['source'].encode()).hexdigest() == row['source_sha256']
                and row['fresh_reopen'] is True and row['identities_requested'] == count, name+' source and fresh reopen')
        native, rust = row['native_projection'], row['rust_projection']
        roots = rust['roots']
        require(native['roots'] == roots and native['nodes'] == rust['nodes'] == count
                and native['wrapped'] is False and rust['wrapped'] is False
                and row['native_total_nodes'] == count + 1 + len(owners)
                and native['reference_encoding'] == ['id']
                and rust['reference_encoding'] == ['elementId'], name+' exact compared topology and wire')
        require(all(root['eClass']=='sysml:Package' for root in roots)
                and [root['data']['declaredName'] for root in roots] == owners, name+' package order')
        nodes = [node for root in roots for node in walk(root)]
        require(len(nodes) == count and {node['#'] for node in nodes} == {f'n{i}' for i in range(count)},
                name+' unique positional identities')
        member, = [node for node in roots[0]['data']['ownedRelationship']
                   if 'T' in [child['data'].get('declaredName') for child in node['data']['ownedRelatedElement']]]
        definition, = member['data']['ownedRelatedElement']
        require(member['eClass']=='sysml:OwningMembership' and definition['eClass']=='sysml:PartDefinition'
                and 'visibility' not in definition['data']
                and ('visibility' in member['data']) == (visibility is not None)
                and member['data'].get('visibility') == visibility, name+' owning membership visibility')
        usage, = [node for node in walk(roots[-1]) if node['eClass']=='sysml:PartUsage']
        edge, = [node for node in nodes if node['eClass']=='sysml:FeatureTyping']
        roles = {'typedFeature':ref(usage)}
        if target is not None:
            require(target == 'A::'+definition['data']['declaredName'], name+' resolved target name')
            roles['type'] = ref(definition)
        require(usage['data']['declaredName']=='x' and edge in usage['data']['ownedRelationship']
                and edge['data'] == roles, name+' exact typing edges')
        warning = [] if unresolved is None else [{
            'severity':'WARNING',
            'message':f"Unable to resolve name '{unresolved}' for reference 'type' on element '[FeatureTyping] B::x::<FeatureTyping>'",
        }]
        require(row['native_warning'] == row['rust_warning'] == warning, name+' exact separate warning')
        require(all(hash_text(row[key]) for key in ('native_raw_sha256','snapshot_sha256','reopen_sha256',
                'legacy_stdout_sha256','default_stdout_sha256')), name+' captured output digests')
    for name, (family, count) in FATAL.items():
        row = data['fatal'][name]
        require(row['error_family']==family and row['identities_requested']==count
                and row['warning_emitted'] is False and row['snapshot_published'] is False
                and hash_text(row['source_sha256']) and hash_text(row['stdout_sha256']), name+' fatal rollback')
    rows = data['incumbents']
    require(len(rows)==305 and sum(item['default'] is True for item in rows.values())==65
            and all(type(item['default']) is bool and hash_text(item['source_sha256'])
                    and hash_text(item['stdout_sha256']) for item in rows.values()), 'exact incumbent inventory')


def main():
    raw = Path(__file__).with_name('receipt.json').read_bytes()
    require(sha256(raw).hexdigest() == PIN, 'receipt digest')
    check(json.loads(raw))
    print('PASS: four fresh reopens, nine atomic refusals, 305 prior outputs and 236 opt-in controls')


if __name__ == '__main__':
    main()
