#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Validate evidence integrity without executing artifacts or fetching dependencies."""
import argparse
from datetime import datetime
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import subprocess
import sys


class Invalid(ValueError):
    pass


def require(condition, message):
    if not condition:
        raise Invalid(message)


def strict_json(raw):
    def pairs(items):
        result = {}
        for key, value in items:
            require(key not in result, f'duplicate JSON key: {key}')
            result[key] = value
        return result
    def constant(value):
        raise Invalid(f'non-JSON number: {value}')
    return json.loads(raw, object_pairs_hook=pairs, parse_constant=constant)


def shape(value, schema, document, at='$'):
    """Small strict evaluator for the checked-in schema vocabulary, not all JSON Schema."""
    if '$ref' in schema:
        schema = document['$defs'][schema['$ref'].removeprefix('#/$defs/')]
    kinds = schema.get('type', [])
    if isinstance(kinds, str):
        kinds = [kinds]
    matches = {'object': isinstance(value, dict), 'array': isinstance(value, list),
               'string': isinstance(value, str), 'integer': type(value) is int,
               'boolean': type(value) is bool, 'null': value is None}
    require(not kinds or any(matches.get(kind, False) for kind in kinds), f'{at}: wrong type')
    if value is None:
        return
    if 'enum' in schema:
        require(value in schema['enum'], f'{at}: invalid enum')
    if isinstance(value, dict):
        for key in schema.get('required', []):
            require(key in value, f'{at}: missing {key}')
        properties = schema.get('properties', {})
        for key, child in value.items():
            rule = properties.get(key, schema.get('additionalProperties', True))
            require(rule is not False, f'{at}: unknown property {key}')
            if isinstance(rule, dict):
                shape(child, rule, document, f'{at}.{key}')
    elif isinstance(value, list):
        require(len(value) >= schema.get('minItems', 0), f'{at}: too few items')
        if schema.get('uniqueItems'):
            require(len({json.dumps(v, sort_keys=True) for v in value}) == len(value), f'{at}: duplicate items')
        for index, child in enumerate(value):
            shape(child, schema.get('items', {}), document, f'{at}[{index}]')
    elif isinstance(value, str):
        require(len(value) >= schema.get('minLength', 0), f'{at}: empty string')
        if 'pattern' in schema:
            require(re.fullmatch(schema['pattern'], value) is not None, f'{at}: pattern mismatch')
    elif type(value) is int:
        require(value >= schema.get('minimum', value), f'{at}: below minimum')


def relative(path):
    value = PurePosixPath(path)
    require(path and not value.is_absolute() and '..' not in value.parts and '\\' not in path,
            f'unsafe repository-relative path: {path}')
    require(str(value) == path and path != '.', f'noncanonical path: {path}')
    return path


def local_bytes(root, ref):
    path = root / relative(ref['path'])
    require(path.resolve().is_relative_to(root.resolve()), f'artifact escapes repository: {path}')
    require(path.is_file(), f'missing artifact: {ref["path"]}')
    raw = path.read_bytes()
    require(hashlib.sha256(raw).hexdigest() == ref['sha256'], f'artifact hash mismatch: {ref["path"]}')
    return raw


def git_bytes(root, revision, path):
    relative(path)
    process = subprocess.run(['git', '-C', str(root), 'show', f'{revision}:{path}'],
                             stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    require(process.returncode == 0, f'missing pinned Git blob {revision}:{path}')
    return process.stdout


def pinned(root, revision, ref):
    raw = git_bytes(root, revision, ref['path'])
    require(hashlib.sha256(raw).hexdigest() == ref['sha256'], f'pinned hash mismatch: {ref["path"]}')
    return raw


def validate(root, ledger, schema, dependency_roots=None, complete=False):
    shape(ledger, schema, schema)
    roots = {'syson': root, **(dependency_roots or {})}
    dependencies = {dep['id']: dep for dep in ledger['dependencies']}
    require(len(dependencies) == len(ledger['dependencies']), 'duplicate dependency IDs')
    require('syson' in dependencies, 'missing syson dependency')
    for dep in dependencies.values():
        require(dep['status'] != 'pinned' or dep['revision'] is not None, f'{dep["id"]}: pinned dependency lacks revision')
    require(dependencies['syson']['revision'] == ledger['upstream_revision'], 'baseline differs from syson dependency')
    ids = set()
    case_ids = set()
    verified = 0
    for row in ledger['behaviors']:
        name = row['id']
        require(name not in ids, f'duplicate behavior ID {name}')
        ids.add(name)
        for ref in row['source_evidence']:
            dep = dependencies.get(ref['dependency_id'])
            require(dep is not None, f'{name}: unknown dependency')
            require(dep['revision'] == ref['revision'], f'{name}: dependency revision differs')
            require(dep['id'] in roots, f'{name}: supply offline dependency root for {dep["id"]}')
            raw = pinned(roots[dep['id']], ref['revision'], ref)
            require(1 <= ref['line_start'] <= ref['line_end'] <= len(raw.splitlines()), f'{name}: source line range invalid')
        for dependency_id in row['dependency_ids']:
            require(dependency_id in dependencies, f'{name}: unknown dependency ID {dependency_id}')
        cases = [case for group in row['cases'].values() for case in group]
        row_cases = set()
        fixture_hashes = set()
        for case in cases:
            require(case['id'] not in case_ids, f'duplicate case ID {case["id"]}')
            case_ids.add(case['id'])
            row_cases.add(case['id'])
            for fixture in case['fixtures']:
                local_bytes(root, fixture)
                fixture_hashes.add(fixture['sha256'])
        implementation = row['implementation']
        if implementation:
            for ref in implementation['files']:
                pinned(root, implementation['revision'], ref)
        # Even pending rows cannot carry missing/corrupted recorded artifacts.
        for side in ('upstream', 'rust'):
            for ref in row['runs'][side]:
                local_bytes(root, ref)
        review = row['review']
        if review:
            local_bytes(root, review['artifact'])
        if row['status'] != 'verified':
            continue
        verified += 1
        require(implementation is not None, f'{name}: verified without implementation')
        require(all(row['cases'][group] for group in ('positive', 'negative', 'failure')), f'{name}: verified requires all three case groups')
        require(cases and all(case['fixtures'] for case in cases), f'{name}: verified cases require hashed fixtures')
        require(row['dependency_ids'], f'{name}: verified without dependency ownership')
        require(all(dependencies[item]['status'] == 'pinned' for item in row['dependency_ids']), f'{name}: unresolved dependency')
        require(row['source_evidence'], f'{name}: verified without source evidence')
        require({ref['dependency_id'] for ref in row['source_evidence']} <= set(row['dependency_ids']), f'{name}: evidence dependency not declared')
        for side in ('upstream', 'rust'):
            require(row['runs'][side], f'{name}: verified without {side} runs')
            covered = set()
            for ref in row['runs'][side]:
                run = strict_json(local_bytes(root, ref))
                shape(run, schema['$defs']['run_report'], schema)
                require(run['side'] == side and run['behavior_id'] == name, f'{name}: run identity differs')
                expected_revision = ledger['upstream_revision'] if side == 'upstream' else implementation['revision']
                require(run['revision'] == expected_revision, f'{name}: run revision differs')
                require(run['executed'] and run['exit_code'] == 0 and run['result'] == 'passed', f'{name}: run not passing/executed')
                try:
                    started = datetime.strptime(run['started_at'], '%Y-%m-%dT%H:%M:%SZ')
                    finished = datetime.strptime(run['finished_at'], '%Y-%m-%dT%H:%M:%SZ')
                except ValueError as error:
                    raise Invalid(f'{name}: invalid run date') from error
                require(finished >= started, f'{name}: run dates reversed')
                require(set(run['case_ids']) <= row_cases, f'{name}: unexpected case ID in run')
                require(set(run['fixture_hashes']) >= fixture_hashes, f'{name}: run fixture coverage incomplete')
                require(local_bytes(root, run['transcript']).strip(), f'{name}: empty run transcript')
                covered.update(run['case_ids'])
            require(covered == row_cases, f'{name}: {side} case coverage incomplete')
        require(review is not None, f'{name}: verified without independent review')
        require(review['disposition'] == 'accepted' and review['independent'], f'{name}: review not independent/accepted')
        require(review['reviewer'] not in implementation['authors'], f'{name}: reviewer authored implementation')
        report = strict_json(local_bytes(root, review['artifact']))
        shape(report, schema['$defs']['review_report'], schema)
        require(report['reviewer'] == review['reviewer'] and report['implementation_revision'] == implementation['revision'], f'{name}: review identity/revision differs')
        require(report['behavior_id'] == name and report['disposition'] == 'accepted' and report['independent'], f'{name}: review report not accepted/independent')
        require(set(report['case_ids']) == row_cases, f'{name}: review case coverage incomplete')
    if complete:
        require(ledger['scope']['inventory_closed'] and ledger['scope']['dependency_closure_closed'], 'completion blocked: inventory/dependency closure open')
        require(ledger['scope']['closure_evidence'], 'completion requires closure evidence')
        for ref in ledger['scope']['closure_evidence']:
            local_bytes(root, ref)
        require(all(dep['status'] == 'pinned' for dep in dependencies.values()), 'completion blocked: unresolved dependency')
        required = [row for row in ledger['behaviors'] if row['required']]
        require(required and all(row['status'] == 'verified' for row in required), 'completion blocked: required behavior not verified')
    return {'behaviors': len(ids), 'verified': verified, 'complete': complete}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parents[3])
    parser.add_argument('--ledger', default='porting/behavior-ledger.json')
    parser.add_argument('--schema', default='porting/ledger-schema/behavior-ledger.schema.json')
    parser.add_argument('--dependency-root', action='append', default=[], metavar='ID=PATH')
    parser.add_argument('--require-complete', action='store_true')
    args = parser.parse_args()
    try:
        roots = {}
        for value in args.dependency_root:
            name, path = value.split('=', 1)
            require(name not in roots and name != 'syson', 'duplicate/reserved dependency root')
            roots[name] = Path(path)
        ledger = strict_json((args.root / relative(args.ledger)).read_bytes())
        schema = strict_json((args.root / relative(args.schema)).read_bytes())
        print(json.dumps(validate(args.root, ledger, schema, roots, args.require_complete), sort_keys=True))
    except (Invalid, OSError, ValueError, KeyError) as error:
        print(f'ledger rejected: {error}', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
