"""Static team overlay checks; actual ownership, readiness and acceptance stay external."""
import hashlib
import json
from pathlib import PurePosixPath, Path
from source_next_splits import legacy_contract, legacy_plan, validate_preparation_splits

TEAMS = [
    'reference', 'semantic-model', 'language-interchange', 'services-persistence',
    'geometry-diagrams', 'interaction-accessibility', 'verification-release',
]
EXECUTION = {
    'source_authors_max': 3, 'coordinators_max': 1, 'concurrent_agents_max': 4,
    'independent_review_required': True, 'self_acceptance_allowed': False,
    'shared_wiring': 'coordinator_only',
    'held_scope_release': 'explicit_owner_handoff_required',
    'runtime_execution': 'coordinator_serialized',
    'assignment_grants_write_lease': False, 'dependency_or_acceptance_waiver': False,
}
PRESERVATION = {
    'unit_count': 790,
    'unit_records_sha256': '6abb573cf979ef117e39ecc73df4f61fd23cc7c11d691052ebfaf0bc2c4872b8',
    'readiness_graph_sha256': '48fd99ff59ee6528bea1aadb7a9206ad54a42dd0a2a129a76ca07b5dfa086045',
    'assignment_sha256': 'ea69a87f1cd5d2c5c92d8135651e825c97a7523de4ee6a4922fe2d8a9cc17e76',
    'omitted_unit_fields': ['completion_contract', 'source_fragment_sha256'],
}


def require(ok, message):
    if not ok:
        raise ValueError(message)


def canonical_hash(value):
    return hashlib.sha256(json.dumps(
        value, sort_keys=True, separators=(',', ':'), ensure_ascii=False,
        allow_nan=False,
    ).encode()).hexdigest()


def validate_teams_legacy(policy, plan):
    """Return a deterministic projection without changing any plan/unit record."""
    require(set(policy) == {'schema', 'team_ids', 'integration_coordinator',
                           'execution_policy', 'preservation', 'assignments'},
            'team policy fields changed')
    require(type(policy['schema']) is int and policy['schema'] == 1,
            'team schema changed')
    require(policy['team_ids'] == TEAMS, 'seven team identities changed')
    require(canonical_hash(policy['integration_coordinator']) == canonical_hash({
        'role': 'integration-coordinator', 'is_team': False}), 'coordinator became a team')
    require(canonical_hash(policy['execution_policy']) == canonical_hash(EXECUTION),
            'team execution caps/review/ownership policy changed')
    require(canonical_hash(policy['preservation']) == canonical_hash(PRESERVATION),
            'team preservation policy changed')
    units = plan['units']
    ids = {unit['id'] for unit in units}
    require(len(ids) == len(units) == plan['unit_count'] == 790,
            'team baseline unit coverage changed')
    normalized = [{key: value for key, value in unit.items()
                   if key not in PRESERVATION['omitted_unit_fields']}
                  for unit in sorted(units, key=lambda item: item['id'])]
    require(canonical_hash(normalized) == PRESERVATION['unit_records_sha256'],
            'team baseline unit obligation changed')
    graph = plan['readiness_graph']
    require(canonical_hash(graph) == PRESERVATION['readiness_graph_sha256'],
            'team baseline typed readiness graph changed')
    assignment = {}
    for row in policy['assignments']:
        require(type(row) is dict and set(row) == {'unit', 'primary_team'},
                'team assignment fields changed')
        uid, team = row['unit'], row['primary_team']
        require(uid in ids, 'unknown team unit')
        require(uid not in assignment, 'duplicate team assignment')
        require(team in TEAMS, 'unknown primary team')
        assignment[uid] = team
    require(set(assignment) == ids, 'missing team assignment')
    require(canonical_hash(policy['assignments']) == PRESERVATION['assignment_sha256'],
            'reviewed primary assignment changed')
    expected_nodes = {stage + uid for stage in ['implementation:', 'accepted:'] for uid in ids}
    require(set(graph) == expected_nodes, 'typed team graph node coverage changed')
    edges = []
    for source in sorted(graph):
        for target in sorted(graph[source]):
            require(target in expected_nodes, 'unknown typed team dependency')
            source_team = assignment[source.split(':', 1)[1]]
            target_team = assignment[target.split(':', 1)[1]]
            if source_team != target_team:
                edges.append({'from': source, 'to': target,
                              'from_team': source_team, 'to_team': target_team})
    return {'assignment': assignment, 'cross_team_edges': edges,
            'team_counts': {team: list(assignment.values()).count(team) for team in TEAMS},
            'policy': dict(EXECUTION)}


def validate_teams(policy, plan):
    contract = json.loads((Path(__file__).resolve().parents[1] / 'completion-contract.json').read_bytes())
    addition = validate_preparation_splits(contract, plan)
    result = validate_teams_legacy(policy, legacy_plan(contract, plan))
    assignment = result['assignment']
    for uid, child in addition['children'].items():
        require(child['team'] in TEAMS, 'unknown preparation team')
        assignment[uid] = child['team']
    result['cross_team_edges'] = [
        {'from': source, 'to': target, 'from_team': assignment[source.split(':', 1)[1]],
         'to_team': assignment[target.split(':', 1)[1]]}
        for source in sorted(plan['readiness_graph']) for target in sorted(plan['readiness_graph'][source])
        if assignment[source.split(':', 1)[1]] != assignment[target.split(':', 1)[1]]]
    result['team_counts'] = {team: list(assignment.values()).count(team) for team in TEAMS}
    return result


def contains(parent, child):
    return child == parent.rstrip('/') or child.startswith(parent.rstrip('/') + '/')


def validate_projection(policy, plan, projection):
    require(canonical_hash(projection) == canonical_hash(validate_teams(policy, plan)),
            'generated cross-team projection changed')


def validate_dispatch(policy, plan, dispatch):
    """Check declared leases; this never proves a live-owner handoff or readiness."""
    projection = validate_teams(policy, plan)
    require(set(dispatch) == {'coordinator', 'authors'}, 'dispatch fields changed')
    coordinator, leases = dispatch['coordinator'], dispatch['authors']
    require(type(coordinator) is str and coordinator.strip(), 'missing dispatch coordinator')
    require(type(leases) is list, 'dispatch authors must be a list')
    byid = {unit['id']: unit for unit in plan['units']}
    crate_roots = {'/'.join(PurePosixPath(path).parts[:3])
                   for unit in plan['units'] for path in unit['paths']
                   if len(PurePosixPath(path).parts) >= 3 and
                   PurePosixPath(path).parts[:2] in [('rust', 'crates'), ('rust', 'tools')] and
                   '.' not in PurePosixPath(path).parts[2]}
    authors, units, paths = set(), set(), []
    fields = {'unit', 'team', 'author', 'reviewer', 'write_paths',
              'ownership_admitted_by', 'overlaps_held_scope', 'handoff_verified', 'shared_wiring'}
    for lease in leases:
        require(type(lease) is dict and set(lease) == fields, 'lease fields changed')
        uid, author, reviewer = lease['unit'], lease['author'], lease['reviewer']
        require(uid in byid and uid not in units, 'unknown or duplicate dispatch unit')
        declared = byid[uid].get('coordinator_wiring', {})
        wiring = declared if isinstance(declared, list) else declared.get('paths', [])
        require(lease['team'] == projection['assignment'][uid], 'lease primary team changed')
        require(type(author) is str and author.strip() and author != coordinator,
                'author/coordinator role conflict')
        require(type(reviewer) is str and reviewer.strip() and reviewer != author,
                'self-review or missing independent reviewer')
        require(lease['ownership_admitted_by'] == coordinator, 'ownership admission missing')
        require(type(lease['overlaps_held_scope']) is bool and
                type(lease['handoff_verified']) is bool, 'held scope declaration missing')
        require(not lease['overlaps_held_scope'] or lease['handoff_verified'],
                'held scope bypass without owner handoff')
        require(lease['shared_wiring'] is False, 'shared wiring is coordinator-only')
        require(type(lease['write_paths']) is list and lease['write_paths'], 'missing write lease')
        for path in lease['write_paths']:
            require(type(path) is str and path and
                    str(PurePosixPath(path)) == path.rstrip('/') and
                    not PurePosixPath(path).is_absolute() and '..' not in PurePosixPath(path).parts,
                    'noncanonical lease path')
        protected = uid == 'M02' or any(
            contains(prefix, path) for path in lease['write_paths']
            for prefix in ['rust/validation/', 'porting/validation/'])
        require(not protected or (lease['overlaps_held_scope'] and lease['handoff_verified']),
                'held scope bypass without owner handoff')
        for path in lease['write_paths']:
            require(any(contains(allowed, path) for allowed in byid[uid]['paths']),
                    'write lease outside unit scope')
            require(PurePosixPath(path).name not in ['Cargo.toml', 'Cargo.lock'] and
                    not any(contains(path, root + '/' + name) for root in crate_roots
                            for name in ['Cargo.toml', 'Cargo.lock']) and
                    not any(contains(shared, path) or contains(path, shared) for shared in wiring),
                    'shared wiring is coordinator-only')
            require(not any(contains(old, path) or contains(path, old) for old in paths),
                    'overlapping author write leases')
            paths.append(path)
        authors.add(author)
        units.add(uid)
    require(len(authors) <= 3 and len(authors | {coordinator}) <= 4,
            'concurrent author/agent cap exceeded')
    return {'team_execution': projection, 'dispatch': dispatch}
