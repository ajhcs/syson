#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Check the durable execution plan without executing referenced artifacts."""
import argparse
from collections import deque
import json
from pathlib import Path, PurePosixPath
import re
import sys
import subprocess
from datetime import datetime

BASELINE = '6ba2ac64e6b16ee2b05752ad55395eebb87d4910'
EXPECTED_AREA_IDS = (
    'DOMAIN-NAME', 'DOMAIN-ID', 'DOMAIN-QUERY', 'EDITOR-ROTATE',
    'EDITOR-GEOMETRY', 'MIGRATION-ID', 'MODEL', 'PARSER', 'UI', 'LAYOUT',
    'API', 'DB', 'RECOVERY', 'HISTORY', 'ACCESSIBILITY', 'EXPORT', 'EXTENSIONS', 'DEPS',
)


class Invalid(ValueError):
    pass


def require(condition, message):
    if not condition:
        raise Invalid(message)


def strict_json(raw):
    def pairs(items):
        out = {}
        for key, value in items:
            require(key not in out, 'duplicate JSON key: ' + key)
            out[key] = value
        return out
    def constant(value):
        raise Invalid('non-JSON number: ' + value)
    try:
        return json.loads(raw, object_pairs_hook=pairs, parse_constant=constant)
    except (json.JSONDecodeError, RecursionError) as exc:
        raise Invalid('invalid JSON: ' + str(exc)) from exc


def obj(value, fields, at, optional=()):
    require(isinstance(value, dict), at + ': expected object')
    require(set(fields) <= set(value), at + ': missing fields ' + str(sorted(set(fields)-set(value))))
    require(set(value) <= set(fields) | set(optional), at + ': unexpected fields ' + str(sorted(set(value)-set(fields)-set(optional))))


def text(value, at):
    require(isinstance(value, str) and bool(value.strip()), at + ': expected nonempty text')
    return value


def array(value, at, nonempty=False):
    require(isinstance(value, list), at + ': expected array')
    require(not nonempty or bool(value), at + ': expected nonempty array')
    return value


def texts(value, at, nonempty=False):
    array(value, at, nonempty)
    for i, item in enumerate(value):
        text(item, f'{at}[{i}]')
    require(len(value) == len(set(value)), at + ': duplicate values')
    return value


def boundary(value, at):
    text(value, at)
    plain = value
    p = PurePosixPath(plain)
    require(plain != '.' and not p.is_absolute() and '..' not in p.parts
            and str(p) == plain and not any(c in plain for c in '\\*?[]{}\x00')
            and not any(ord(c) < 32 or ord(c) == 127 for c in plain),
            at + ': expected canonical repository-relative file/directory boundary')
    return plain


def boundaries(value, at):
    texts(value, at, nonempty=True)
    return [boundary(v, at) for v in value]


def overlaps(a, b):
    return a == b or a.startswith(b + '/') or b.startswith(a + '/')


def dag(graph, at):
    indegree = {key: 0 for key in graph}
    outgoing = {key: [] for key in graph}
    for key, dependencies in graph.items():
        texts(dependencies, at + '.' + key)
        for dependency in dependencies:
            require(dependency in graph, at + ': unknown dependency ' + dependency)
            require(dependency != key, at + ': self dependency ' + key)
            indegree[key] += 1
            outgoing[dependency].append(key)
    ready = deque(key for key, degree in indegree.items() if not degree)
    visited = 0
    while ready:
        key = ready.popleft()
        visited += 1
        for other in outgoing[key]:
            indegree[other] -= 1
            if not indegree[other]:
                ready.append(other)
    require(visited == len(graph), at + ': dependency cycle')


def blockers(value, at):
    array(value, at)
    for i, blocker in enumerate(value):
        name = f'{at}[{i}]'
        obj(blocker, ['action', 'target', 'reason', 'status'], name)
        for key in ['action', 'target', 'reason']:
            text(blocker[key], name + '.' + key)
        require(isinstance(blocker['status'], str) and blocker['status'].lower() in ['pending', 'denied', 'uncertain', 'unavailable'],
                name + ': invalid blocker status')


EVIDENCE_KINDS = {'source', 'rust_test', 'upstream_run', 'independent_review',
                  'integration_run', 'real_user_acceptance', 'license'}
EVIDENCE_STATUS = {'RECORDED', 'PASSED', 'ACCEPTED', 'PENDING', 'NOT_RUN'}


def artifact(root, ref, at):
    obj(ref, ['path', 'kind', 'status', 'scope', 'revision'], at)
    path = boundary(ref['path'], at + '.path')
    require(path == ref['path'], at + ': evidence must name a literal file')
    require(isinstance(ref['kind'], str) and ref['kind'] in EVIDENCE_KINDS, at + ': invalid evidence kind')
    require(isinstance(ref['status'], str) and ref['status'] in EVIDENCE_STATUS, at + ': invalid evidence status')
    text(ref['scope'], at + '.scope')
    revision = ref['revision']
    require(revision is None or (isinstance(revision, str) and re.fullmatch(r'[0-9a-f]{40}', revision)),
            at + ': expected exact Git revision or null for pending evidence')
    if ref['status'] in {'PENDING', 'NOT_RUN'}:
        return None
    require(revision is not None, at + ': recorded evidence lacks exact revision')
    result = subprocess.run(['git', '-C', str(root), 'show', revision + ':' + path],
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    require(result.returncode == 0, at + ': evidence revision/path not available in Git')
    require(bool(result.stdout), at + ': empty pinned evidence artifact')
    if ref['kind'] == 'source':
        # A historical source proof names this exact Git blob, not today's file.
        return result.stdout
    file = root / path
    require(file.resolve().is_relative_to(root.resolve()), at + ': evidence escapes repository')
    require(file.is_file(), at + ': missing immutable evidence artifact')
    raw = file.read_bytes()
    require(result.stdout == raw, at + ': immutable report differs from pinned Git bytes')
    return raw


def utc(value, at):
    text(value, at)
    try:
        stamp = datetime.fromisoformat(value.replace('Z', '+00:00'))
    except ValueError as exc:
        raise Invalid(at + ': invalid timestamp') from exc
    require(stamp.tzinfo is not None and stamp.utcoffset().total_seconds() == 0,
            at + ': timestamp must explicitly identify UTC')
    return stamp


def lease(value, item, at):
    if value is None:
        require(item['state'] not in {'ACTIVE', 'DONE'}, at + ': ACTIVE/DONE item requires an explicit lease record')
        return None
    obj(value, ['holder', 'writable_boundary', 'status', 'release_mode',
                'recorded_at_utc', 'acknowledgement'], at)
    require(value['holder'] == item['owner_lane'], at + ': lease holder differs from owner')
    actual = boundaries(value['writable_boundary'], at + '.writable_boundary')
    intended = boundaries(item['writable_boundary'], at + '.item_boundary')
    require(set(actual) == set(intended), at + ': lease differs from declared writable boundary')
    require(isinstance(value['status'], str) and value['status'] in {'ACTIVE', 'RELEASED'}, at + ': invalid lease status')
    require(value['release_mode'] == 'EXPLICIT_ONLY', at + ': leases may not expire automatically')
    utc(value['recorded_at_utc'], at + '.recorded_at_utc')
    text(value['acknowledgement'], at + '.acknowledgement')
    if item['state'] == 'ACTIVE':
        require(value['status'] == 'ACTIVE', at + ': ACTIVE item cannot have a released lease')
    if item['state'] == 'DONE':
        require(value['status'] == 'RELEASED', at + ': DONE requires explicit lease release')
    return actual if value['status'] == 'ACTIVE' else None


def plan_entries(value, at, nonempty=True):
    array(value, at, nonempty)
    def substantive(item):
        if isinstance(item, str):
            return bool(item.strip())
        if isinstance(item, dict):
            return bool(item) and any(substantive(v) for v in item.values())
        if isinstance(item, list):
            return any(substantive(v) for v in item)
        return False
    for i, item in enumerate(value):
        require(isinstance(item, (str, dict)) and substantive(item),
                f'{at}[{i}]: empty or nontextual evidence plan')


def evidence(root, value, at):
    array(value, at)
    seen = set()
    for i, ref in enumerate(value):
        artifact(root, ref, f'{at}[{i}]')
        key = (ref['kind'], ref['path'], ref['revision'])
        require(key not in seen, at + ': duplicate evidence reference')
        seen.add(key)
    return value


def denominator(root, value):
    obj(value, ['baseline_area_ids', 'current_area_ids', 'inventory_complete', 'unit', 'history'], 'denominator')
    text(value['unit'], 'denominator.unit')
    expected = set(EXPECTED_AREA_IDS)
    for key in ['baseline_area_ids', 'current_area_ids']:
        texts(value[key], 'denominator.' + key, True)
        require(set(value[key]) == expected, 'denominator: exact original eighteen anchors required')
    require(type(value['inventory_complete']) is bool, 'denominator.inventory_complete: expected boolean')
    history = array(value['history'], 'denominator.history', True)
    require(len(history) == 1, 'denominator: future change events require a reviewed validator extension')
    event = history[0]
    obj(event, ['id', 'type', 'recorded_at_utc', 'area_ids', 'reason', 'source', 'source_revision'], 'denominator.history[0]')
    text(event['id'], 'denominator.history.id')
    require(event['type'] == 'ESTABLISH', 'denominator: initial event must ESTABLISH; future events require review')
    utc(event['recorded_at_utc'], 'denominator.history.recorded_at_utc')
    texts(event['area_ids'], 'denominator.history.area_ids', True)
    require(set(event['area_ids']) == expected, 'denominator: establishment map differs')
    text(event['reason'], 'denominator.history.reason')
    require(event['source'] == 'porting/PARITY-LEDGER.json', 'denominator: wrong original anchor source')
    revision = event['source_revision']
    require(isinstance(revision, str) and re.fullmatch(r'[0-9a-f]{40}', revision), 'denominator: exact source revision required')
    raw = subprocess.run(['git', '-C', str(root), 'show', revision + ':' + event['source']],
                         stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    require(raw.returncode == 0, 'denominator: pinned original anchor source missing')
    original = strict_json(raw.stdout)
    require(isinstance(original, dict) and isinstance(original.get('features'), list), 'denominator: source lacks features')
    source_ids = [entry.get('id') for entry in original['features'] if isinstance(entry, dict)]
    require(all(isinstance(value, str) for value in source_ids), 'denominator: invalid source area IDs')
    require(len(source_ids) == len(EXPECTED_AREA_IDS) and set(source_ids) == expected,
            'denominator: pinned source does not establish exact eighteen anchors')


def inventory_proof(root, area_id, area):
    require(not area['unclosed_surface'], area_id + ': closed inventory retains unclosed surface')
    proofs = [ref for ref in area['completion_evidence'] if ref['kind'] == 'source' and ref['status'] in {'PASSED', 'ACCEPTED'}]
    require(bool(proofs), area_id + ': inventory closure lacks accepted source proof')
    found = False
    for ref in proofs:
        raw = artifact(root, ref, area_id + '.inventory_proof')
        try:
            report = strict_json(raw)
        except (Invalid, UnicodeDecodeError):
            continue
        if (isinstance(report, dict) and report.get('area_id') == area_id
                and report.get('baseline') == BASELINE and report.get('inventory_complete') is True):
            texts(report.get('behavior_ids'), area_id + '.inventory.behavior_ids', True)
            found = True
    require(found, area_id + ': accepted source proof does not establish scoped inventory closure')


def parity_proof(root, area_id, area, inventory_complete):
    refs = area['completion_evidence']
    qualified = {ref['kind']: ref for ref in refs if ref['status'] in {'PASSED', 'ACCEPTED'}}
    required = {'source', 'rust_test', 'upstream_run', 'independent_review',
                'integration_run', 'real_user_acceptance', 'license'}
    require(inventory_complete, area_id + ': parity requires closed inventory')
    require(not area['blockers'], area_id + ': parity cannot retain unresolved blockers')
    require(area['status']['implemented'] == 'COMPLETE' and area['status']['rust_tested'] == 'COMPLETE',
            area_id + ': parity requires complete implementation and Rust qualification')
    require(required <= set(qualified), area_id + ': parity lacks passed/accepted evidence layers')
    for kind in required:
        ref = qualified[kind]
        raw = artifact(root, ref, area_id + '.parity.' + kind)
        try:
            report = strict_json(raw)
        except (Invalid, UnicodeDecodeError) as exc:
            raise Invalid(area_id + ': parity proof requires structured JSON reports') from exc
        require(isinstance(report, dict) and report.get('area_id') == area_id
                and report.get('evidence_kind') == kind, area_id + ': parity evidence scope/kind mismatch')
        require(report.get('baseline') == BASELINE, area_id + ': parity evidence baseline mismatch')
        if kind in {'rust_test', 'upstream_run', 'integration_run'}:
            require(report.get('executed') is True and report.get('result') == 'passed'
                    and type(report.get('exit_code')) is int and report['exit_code'] == 0,
                    area_id + ': parity run is not actual passed execution')
            texts(report.get('command'), area_id + '.parity.command', True)
            start = utc(report.get('started_at_utc'), area_id + '.parity.started_at_utc')
            end = utc(report.get('finished_at_utc'), area_id + '.parity.finished_at_utc')
            require(end >= start, area_id + ': parity run chronology reversed')
            texts(report.get('case_ids'), area_id + '.parity.case_ids', True)
        elif kind == 'independent_review':
            require(report.get('independent') is True and report.get('disposition') == 'accepted',
                    area_id + ': parity needs accepted independent review')
            reviewer = text(report.get('reviewer_lane'), area_id + '.parity.reviewer_lane')
            require(reviewer != area['owner_lane'], area_id + ': implementation owner cannot self-review parity')
        elif kind == 'source':
            require(report.get('inventory_complete') is True, area_id + ': source inventory closure not established')
        else:
            require(report.get('disposition') == 'accepted', area_id + ': acceptance/license evidence not accepted')


def metadata(document):
    utc(document['recorded_at_utc'], 'plan.recorded_at_utc')
    text(document['integrator'], 'integrator')
    texts(document['existing_lanes'], 'existing_lanes', True)
    require(document['integrator'] in document['existing_lanes'], 'integrator is not an existing lane')
    authority = document['authority']
    obj(authority, ['authorized', 'not_granted'], 'authority')
    texts(authority['authorized'], 'authority.authorized', True)
    texts(authority['not_granted'], 'authority.not_granted', True)
    continuity = document['continuity']
    obj(continuity, ['mode', 'future_wakeup_guaranteed', 'scheduler_report', 'resume_entrypoint'], 'continuity')
    text(continuity['mode'], 'continuity.mode')
    text(continuity['resume_entrypoint'], 'continuity.resume_entrypoint')
    require(type(continuity['future_wakeup_guaranteed']) is bool, 'continuity: wakeup flag must be boolean')
    report = continuity['scheduler_report']
    obj(report, ['source', 'result', 'automation_created', 'workaround_created'], 'continuity.scheduler_report')
    text(report['source'], 'continuity.scheduler_report.source')
    text(report['result'], 'continuity.scheduler_report.result')
    require(type(report['automation_created']) is bool and type(report['workaround_created']) is bool,
            'continuity: scheduler flags must be boolean')
    require(not continuity['future_wakeup_guaranteed'] and not report['automation_created'],
            'continuity: future scheduling claims require separately reviewed evidence/schema')
    require(not report['workaround_created'], 'continuity: unsupported workaround requires separate reviewed change')
    array(document['history'], 'plan.history', True)
    for entry in document['history']:
        obj(entry, ['event', 'recorded_at_utc', 'detail'], 'plan.history event')
        text(entry['event'], 'plan.history.event')
        utc(entry['recorded_at_utc'], 'plan.history.recorded_at_utc')
        text(entry['detail'], 'plan.history.detail')
    resources = document['resources']
    obj(resources, ['global_max_compile_jobs', 'max_compile_jobs', 'allocation_authority', 'per_process_jobs', 'compiler_leases', 'displays'], 'resources')
    require(type(resources['global_max_compile_jobs']) is int and 1 <= resources['global_max_compile_jobs'] <= 2,
            'resources: global compiler capacity exceeds reviewed bound')
    require(type(resources['max_compile_jobs']) is int and 0 <= resources['max_compile_jobs'] <= resources['global_max_compile_jobs'],
            'resources: project allocation exceeds global capacity')
    text(resources['allocation_authority'], 'resources.allocation_authority')
    require(type(resources['per_process_jobs']) is int and resources['per_process_jobs'] == 1,
            'resources: one job per compiler process required')
    array(resources['compiler_leases'], 'resources.compiler_leases')
    text(resources['displays'], 'resources.displays')


def resource_leases(document, items):
    total = 0
    seen = set()
    for ref in document['resources']['compiler_leases']:
        obj(ref, ['holder', 'item', 'jobs'], 'compiler lease')
        text(ref['holder'], 'compiler lease.holder')
        text(ref['item'], 'compiler lease.item')
        require(ref['item'] in items and ref['item'] not in seen, 'compiler lease: missing or duplicate item')
        seen.add(ref['item'])
        item = items[ref['item']]
        require(item['state'] == 'ACTIVE' and item['lease'] is not None
                and item['lease']['status'] == 'ACTIVE' and item['owner_lane'] == ref['holder'],
                'compiler lease: holder/item is not active with matching ownership')
        require(type(ref['jobs']) is int and ref['jobs'] == document['resources']['per_process_jobs'],
                'compiler lease: jobs differ from reviewed per-process bound')
        total += ref['jobs']
    require(total <= document['resources']['max_compile_jobs'], 'compiler leases exceed concurrent bound')


def validate(root, document):
    root = Path(root)
    obj(document, ['schema_version', 'plan_id', 'recorded_at_utc', 'baseline', 'denominator',
                   'integrator', 'existing_lanes', 'resources', 'authority', 'continuity',
                   'history', 'areas', 'items'], 'plan')
    require(type(document['schema_version']) is int and document['schema_version'] == 1, 'schema_version: expected 1')
    text(document['plan_id'], 'plan_id')
    baseline = document['baseline']
    obj(baseline, ['syson', 'sirius_source_candidate', 'syside_source_candidate', 'artifact_correspondence'], 'baseline')
    require(baseline['syson'] == BASELINE, 'plan: pinned baseline differs')
    require(baseline['sirius_source_candidate'] == '3191e2b8990373afee3415f6b26f8c5039a28321', 'baseline: Sirius source pin differs')
    require(baseline['syside_source_candidate'] == '1326f234cae23b0e4777d5a48a069b17e9e332f1', 'baseline: Syside source pin differs')
    text(baseline['artifact_correspondence'], 'baseline.artifact_correspondence')
    metadata(document)
    denominator(root, document['denominator'])
    areas, items = document['areas'], document['items']
    require(isinstance(areas, dict) and set(areas) == set(EXPECTED_AREA_IDS), 'areas: exact original eighteen anchors required')
    require(isinstance(items, dict) and bool(items), 'items: nonempty item map required')
    area_fields = ['title', 'lifecycle', 'inventory_complete', 'unclosed_surface', 'dependency_semantics',
                   'upstream_baselines', 'inventory_refs', 'known_behaviors', 'exclusions',
                   'owner_lane', 'writable_boundary', 'dependencies', 'milestones', 'rust_tests',
                   'upstream_tests', 'integration_acceptance', 'real_user_acceptance',
                   'provenance_license_gates', 'completion_evidence', 'status', 'blockers', 'next_ready_item']
    for aid, area in areas.items():
        obj(area, area_fields, aid)
        text(area['title'], aid + '.title')
        text(area['dependency_semantics'], aid + '.dependency_semantics')
        require(area['lifecycle'] in ['ACTIVE', 'BLOCKED', 'COMPLETE'], aid + ': invalid lifecycle')
        require(type(area['inventory_complete']) is bool, aid + ': inventory_complete must be boolean')
        plan_entries(area['unclosed_surface'], aid + '.unclosed_surface', not area['inventory_complete'])
        text(area['owner_lane'], aid + '.owner_lane')
        require(area['owner_lane'] in document['existing_lanes'], aid + ': unknown owner lane')
        boundaries(area['writable_boundary'], aid + '.writable_boundary')
        for key in ['upstream_baselines', 'inventory_refs', 'known_behaviors', 'rust_tests', 'upstream_tests',
                    'integration_acceptance', 'real_user_acceptance', 'provenance_license_gates']:
            plan_entries(area[key], aid + '.' + key)
        plan_entries(area['exclusions'], aid + '.exclusions', False)
        texts(area['milestones'], aid + '.milestones', True)
        blockers(area['blockers'], aid + '.blockers')
        evidence(root, area['completion_evidence'], aid + '.completion_evidence')
        obj(area['status'], ['implemented', 'rust_tested', 'parity_verified'], aid + '.status')
        for key in ['implemented', 'rust_tested']:
            require(isinstance(area['status'][key], str) and area['status'][key] in {'NONE', 'BOUNDED', 'COMPLETE'}, aid + ': invalid qualification status')
        require(isinstance(area['status']['parity_verified'], str) and area['status']['parity_verified'] in {'NO', 'YES'}, aid + ': invalid parity status')
        if area['inventory_complete']:
            inventory_proof(root, aid, area)
        qualified = {(r['kind'], r['status']) for r in area['completion_evidence']}
        if area['status']['implemented'] != 'NONE':
            require(any(('source', state) in qualified for state in ['RECORDED', 'PASSED', 'ACCEPTED']), aid + ': implementation claim lacks recorded source evidence')
        if area['status']['rust_tested'] != 'NONE':
            require(('rust_test', 'PASSED') in qualified, aid + ': Rust qualification lacks passed evidence')
        if 'COMPLETE' in [area['status']['implemented'], area['status']['rust_tested']]:
            require(area['inventory_complete'], aid + ': COMPLETE requires area inventory closure')
        if area['lifecycle'] == 'COMPLETE':
            require(area['status']['parity_verified'] == 'YES', aid + ': lifecycle completion requires qualified parity')
        if area['status']['parity_verified'] == 'YES':
            require(area['inventory_complete'], aid + ': parity requires area inventory closure')
            parity_proof(root, aid, area, document['denominator']['inventory_complete'])
    dag({aid: a['dependencies'] for aid, a in areas.items()}, 'area dependencies')
    item_fields = ['area_ids', 'description', 'kind', 'state', 'owner_lane', 'writable_boundary', 'depends_on',
                   'authorization_scope', 'next_action', 'rust_tests', 'upstream_tests', 'acceptance', 'completion_evidence',
                   'blockers', 'lease']
    held = []
    for iid, item in items.items():
        text(iid, 'item id')
        obj(item, item_fields, iid)
        texts(item['area_ids'], iid + '.area_ids', True)
        require(set(item['area_ids']) <= set(areas), iid + ': unknown area ID')
        for key in ['description', 'kind', 'owner_lane', 'authorization_scope', 'next_action']:
            text(item[key], iid + '.' + key)
        require(item['owner_lane'] in document['existing_lanes'], iid + ': unknown owner lane')
        boundaries(item['writable_boundary'], iid + '.writable_boundary')
        require(isinstance(item['state'], str) and item['state'] in {'READY', 'ACTIVE', 'BLOCKED', 'DONE'}, iid + ': invalid state')
        for key in ['rust_tests', 'upstream_tests', 'acceptance']:
            plan_entries(item[key], iid + '.' + key)
        blockers(item['blockers'], iid + '.blockers')
        refs = evidence(root, item['completion_evidence'], iid + '.completion_evidence')
        if item['state'] == 'BLOCKED':
            require(bool(item['blockers']), iid + ': BLOCKED requires explicit action/target/reason/status')
        if item['state'] in {'READY', 'DONE'}:
            require(not item['blockers'], iid + ': READY/DONE cannot retain unresolved blockers')
        if item['state'] == 'DONE':
            require(any(ref['status'] in {'RECORDED', 'PASSED', 'ACCEPTED'} for ref in refs),
                    iid + ': DONE lacks bounded recorded completion evidence')
        claimed = lease(item['lease'], item, iid + '.lease')
        if claimed is not None:
            held.append((iid, item['owner_lane'], claimed))
    dag({iid: i['depends_on'] for iid, i in items.items()}, 'item dependencies')
    for iid, item in items.items():
        if item['state'] in {'READY', 'ACTIVE', 'DONE'}:
            require(all(items[dep]['state'] == 'DONE' for dep in item['depends_on']),
                    iid + ': ready/active/done item has unmet dependencies')
    for i, (left_id, left_owner, left_paths) in enumerate(held):
        for right_id, right_owner, right_paths in held[i+1:]:
            require(left_owner == right_owner or not any(overlaps(a, b) for a in left_paths for b in right_paths),
                    left_id + '/' + right_id + ': conflicting held writable leases')
    for aid, area in areas.items():
        for milestone in area['milestones']:
            require(milestone in items and aid in items[milestone]['area_ids'], aid + ': milestone item missing or mismatched area')
        ready = [iid for iid, item in items.items() if aid in item['area_ids'] and item['state'] in {'READY', 'ACTIVE'}]
        selected = area['next_ready_item']
        require(selected is None or (isinstance(selected, str) and selected in ready), aid + ': next_ready_item is not a matching READY/ACTIVE item')
        require(not ready or selected is not None, aid + ': available work lacks next_ready_item')
    resource_leases(document, items)
    require(not document['denominator']['inventory_complete'] or all(a['inventory_complete'] for a in areas.values()), 'denominator: global inventory cannot close while an area remains open')
    return {'plan_id': document['plan_id'], 'areas': len(areas), 'items': len(items),
            'states': {state: sum(i['state'] == state for i in items.values()) for state in ['READY', 'ACTIVE', 'BLOCKED', 'DONE']},
            'held_leases': len(held), 'parity_verified_areas': sum(a['status']['parity_verified'] == 'YES' for a in areas.values()),
            'inventory_complete': document['denominator']['inventory_complete']}


def validate_transition(previous, current):
    """Retain held ownership across snapshots; current must also pass validate()."""
    require(isinstance(previous, dict) and previous.get('plan_id') == current['plan_id'],
            'lease history: previous plan identity differs')
    old_items = previous.get('items')
    require(isinstance(old_items, dict), 'lease history: previous items missing')
    compared = 0
    for iid, old in old_items.items():
        require(isinstance(old, dict), 'lease history: invalid previous item')
        prior = old.get('lease')
        if prior is None:
            continue
        obj(prior, ['holder', 'writable_boundary', 'status', 'release_mode', 'recorded_at_utc', 'acknowledgement'], 'lease history previous record')
        require(prior['status'] in ['ACTIVE', 'RELEASED'] and prior['release_mode'] == 'EXPLICIT_ONLY', 'lease history: invalid previous release policy')
        require(prior['holder'] == old.get('owner_lane'), 'lease history: previous ownership mismatch')
        text(prior['acknowledgement'], 'lease history previous acknowledgement')
        utc(prior['recorded_at_utc'], 'lease history previous UTC')
        if prior['status'] != 'ACTIVE':
            continue
        compared += 1
        require(iid in current['items'], iid + ': held item may not disappear')
        new = current['items'][iid]
        after = new['lease']
        require(after is not None, iid + ': held lease silently deleted')
        require(after['holder'] == prior.get('holder'), iid + ': held ownership changed without explicit release snapshot')
        before_paths = boundaries(prior.get('writable_boundary'), iid + '.previous_boundary')
        after_paths = boundaries(after['writable_boundary'], iid + '.next_boundary')
        require(set(before_paths) == set(after_paths), iid + ': held scope changed without explicit release snapshot')
        require(utc(after['recorded_at_utc'], iid + '.lease.next_utc') >= utc(prior['recorded_at_utc'], iid + '.lease.previous_utc'), iid + ': lease history timestamp moved backwards')
        if after['status'] == 'RELEASED':
            require(after['acknowledgement'] != prior.get('acknowledgement'), iid + ': release lacks a new explicit acknowledgement')
    return compared


def prior_snapshot(root, plan_path, explicit):
    if explicit is not None:
        path = explicit if explicit.is_absolute() else root / explicit
        return strict_json(path.read_bytes()), 'explicit previous snapshot'
    try:
        relative = str(plan_path.resolve().relative_to(root.resolve()))
    except ValueError:
        return None, 'not available: plan outside repository'
    result = subprocess.run(['git', '-C', str(root), 'show', 'HEAD:' + relative],
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    if result.returncode != 0:
        return None, 'not available: initial plan has no committed predecessor'
    return strict_json(result.stdout), 'committed HEAD snapshot'


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--root', type=Path, default=Path(__file__).resolve().parents[3])
    p.add_argument('--plan', type=Path, default=Path('porting/execution-backlog.json'))
    p.add_argument('--previous-plan', type=Path, help='explicit prior snapshot; otherwise compare committed HEAD when available')
    args = p.parse_args()
    path = args.plan if args.plan.is_absolute() else args.root / args.plan
    try:
        current = strict_json(path.read_bytes())
        result = validate(args.root, current)
        previous, source = prior_snapshot(args.root, path, args.previous_plan)
        result['lease_history_source'] = source
        result['prior_held_leases_checked'] = validate_transition(previous, current) if previous is not None else 0
    except (Invalid, OSError, UnicodeDecodeError) as exc:
        print('EXECUTION BACKLOG INVALID: ' + str(exc), file=sys.stderr)
        return 1
    print(json.dumps({'status': 'VALID', **result}, sort_keys=True))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
