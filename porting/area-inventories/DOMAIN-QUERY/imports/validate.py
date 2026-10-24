#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Check frozen source and synthetic trace consistency; no graph import evaluator."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

PIN = '6ba2ac64e6b16ee2b05752ad55395eebb87d4910'
HERE = Path(__file__).resolve().parent


def require(ok, message):
    if not ok:
        raise ValueError(message)


def read_json(raw):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            require(key not in result, 'duplicate JSON key: ' + key)
            result[key] = value
        return result
    return json.loads(raw, object_pairs_hook=unique)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def check_sources(contract, root):
    require(contract['upstream_commit'] == PIN, 'wrong upstream pin')
    for key, source in contract['sources'].items():
        path = source['path']
        require(not Path(path).is_absolute() and '..' not in Path(path).parts, 'source path')
        data = subprocess.check_output(['git', '-C', str(root), 'show', PIN + ':' + path])
        blob = subprocess.check_output(['git', '-C', str(root), 'rev-parse', PIN + ':' + path], text=True).strip()
        require(blob == source['blob'] and digest(data) == source['sha256'], key + ' source drift')
        lines = data.splitlines(keepends=True)
        for anchor in source['anchors']:
            first, last = anchor['first'], anchor['last']
            require(1 <= first <= last <= len(lines), key + ' line range')
            require(digest(b''.join(lines[first - 1:last])) == anchor['sha256'], key + ' anchor drift')


def names(member):
    """Caller-observed getters; alias uses Java reference identity, not equality."""
    if member['element'] is None:
        return []
    if member['member_name'] is not None and not member['same_name_reference']:
        return [member['member_name']]
    return [member['effective_name'], member['effective_short_name']]


def check_filter_steps(case, members):
    # Only prove local NameConflictingFilter state transitions. Never traverse
    # namespaces, resolve references, compute visibility, or call upstream code.
    used = set(case['seed_names'])
    accepted = []
    for step in case['filter_steps']:
        member_id = step['membership']
        require(member_id in members, 'unknown filter member')
        require(set(step['used_before']) == used, 'filter state continuity')
        result = True
        for name in names(members[member_id]):
            if name is not None:
                if name in used:
                    result = False
                    break  # Java && does not evaluate the short name.
                used.add(name)
        require(type(step['accepted']) is bool and step['accepted'] == result, 'filter acceptance')
        require(set(step['used_after']) == used, 'filter state effect')
        if result:
            accepted.append(member_id)
    require(case['expected']['memberships'] == accepted, 'accepted order')


def check_fixtures(contract, fixtures):
    require(fixtures['schema'] == 1, 'fixture schema')
    require(fixtures['evidence'] == 'SOURCE_DERIVED_NOT_EXECUTED', 'evidence label')
    members, namespaces = fixtures['members'], fixtures['namespaces']
    require(len(namespaces) == len(set(namespaces)), 'duplicate namespace identity')
    for member in members.values():
        require(set(member) == {'element', 'element_name', 'member_name', 'same_name_reference', 'effective_name', 'effective_short_name'}, 'member shape')
        require(type(member['same_name_reference']) is bool, 'reference observation required')
        require(not member['same_name_reference'] or member['member_name'] == member['element_name'], 'reference identity requires equal text')
        for name in ['element_name', 'member_name', 'effective_name', 'effective_short_name']:
            require(member[name] is None or isinstance(member[name], str), 'name observation type')
    seen, obligations, statuses = set(), set(), set()
    for case in fixtures['cases']:
        require(case['id'] not in seen, 'duplicate case identity')
        seen.add(case['id'])
        require(case['source_refs'] and set(case['source_refs']) <= contract['sources'].keys(), 'source references')
        require(set(case['obligations']) <= contract['obligations'].keys(), 'unknown obligation')
        obligations.update(case['obligations'])
        status = case['expected']['status']
        statuses.add(status)
        require(status in contract['required_statuses'], 'unknown outcome')
        result = case['expected']['memberships']
        require(result is None or isinstance(result, list) and all(m in members for m in result), 'result references')
        if status != 'MEMBERSHIPS':
            require(result is None, 'unsupported/null outcome has no fabricated list')
        else:
            require(isinstance(result, list), 'membership outcome requires list')
        operation = case['operation']
        require(operation in {'namespace_pipeline', 'filter_trace', 'graph_trace', 'stub', 'unresolved'}, 'operation')
        if operation in {'namespace_pipeline', 'filter_trace'}:
            check_filter_steps(case, members)
        if operation == 'namespace_pipeline':
            request, calls = case['request'], case['calls']
            require(request['namespace'] in namespaces and all(n in namespaces for n in request['excluded']), 'namespace references')
            require([c['import'] for c in calls] == case['import_order'], 'import call order')
            for call in calls:
                require(call['excluded'] == request['excluded'] + [request['namespace']], 'exclusion append/order')
            candidates = [m for call in calls for m in call['returned']]
            require(candidates == [s['membership'] for s in case['filter_steps']], 'candidate encounter order')
            seed = {n for m in case['owned'] for n in [members[m]['effective_name'], members[m]['effective_short_name']] if members[m]['element'] is not None and n is not None}
            require(set(case['seed_names']) == seed, 'owned effective-name seed')
        if operation == 'graph_trace':
            request = case['request']
            require(type(request['recursive']) is bool and type(request['include_all']) is bool, 'query flags')
            require(request['kind'] in {'namespace', 'membership'}, 'import kind')
            require(request['namespace'] in namespaces, 'request namespace')
            require(request['entrypoint'] in {'Namespace.visibleMemberships', 'NamespaceImport.importedMemberships', 'MembershipImport.importedMemberships'}, 'graph entrypoint')
            for node, graph in case['graph'].items():
                require(node in namespaces, 'graph namespace identity')
                require(all(m['membership'] in members and m['visibility'] in {'public', 'private', 'protected'} for m in graph['owned']), 'graph owned membership')
                for imp in graph['imports']:
                    require(imp['target'] in namespaces, 'graph import target')
                    require(imp['visibility'] in {'public', 'private', 'protected'}, 'import visibility')
            visited = set(case['excluded'])
            require(visited <= set(namespaces), 'unknown excluded identity')
            for event in case['visits']:
                require(event['namespace'] in namespaces, 'unknown visited identity')
                already = event['namespace'] in visited
                require(event['action'] == ('skip' if already else 'enter'), 'visited/exclusion trace')
                visited.add(event['namespace'])
            require(set(case['visited_after']) == visited, 'visited trace end')
    require(obligations == contract['obligations'].keys(), 'missing obligation coverage')
    require(statuses == set(contract['required_statuses']), 'missing unsupported/null outcomes')
    return len(seen)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=HERE.parents[3])
    args = parser.parse_args()
    contract = read_json((HERE / 'contract.json').read_text())
    raw = (HERE / 'fixtures.json').read_bytes()
    require(digest(raw) == contract['fixtures_sha256'], 'frozen fixture drift')
    check_sources(contract, args.root)
    count = check_fixtures(contract, read_json(raw))
    print(json.dumps({'status': 'PASS', 'sources': len(contract['sources']), 'fixture_cases': count,
                      'upstream_execution': 'NOT_RUN', 'graph_evaluator': 'NOT_IMPLEMENTED'}))


if __name__ == '__main__':
    main()
