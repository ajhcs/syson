"""Real captured baseline; all new children and candidate code bytes are SYNTHETIC_UNSEALED."""
import copy
import hashlib
import json
import os
from pathlib import Path
import tempfile
import types
import unittest
from unittest.mock import patch
import prepare as P

ROOT = Path(__file__).resolve().parents[2]


class PreparationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.old_lock = (ROOT / 'completion-contract.lock.json').read_bytes()
        cls.old_snapshot = (ROOT / 'remaining-work/snapshot.lock.json').read_bytes()
        cls.old_register = (ROOT / 'remaining-work/units.json').read_bytes()
        names = set(P.parse(cls.old_snapshot)['sha256']) | set(P.PRESERVED)
        cls.files = {name: (ROOT / name).read_bytes() for name in names}
        provenance = P.parse((ROOT / 'composition/fixtures/baseline-provenance.json').read_bytes())
        for name, row in provenance['files'].items():
            raw = (ROOT / 'composition/fixtures' / name).read_bytes()
            if hashlib.sha256(raw).hexdigest() != row['sha256'] or len(raw) != row['bytes']:
                raise ValueError('captured real baseline changed')
            cls.files[row['original_path']] = raw
        amendment = b'# SYNTHETIC_UNSEALED control; no work registered\n'
        lane = {'lane': 'storage-source', 'units': []}
        for uid, deps, team in [('RR13a', ['RR04'], 'services-persistence'),
                                ('RR13b', ['RR13a'], 'verification-release')]:
            lane['units'].append({'id': uid, 'title': 'SYNTHETIC_UNSEALED control',
                'dependencies': deps, 'implementation_ready_dependencies': deps.copy(), 'primary_team': team,
                'paths': ['synthetic/unused'], 'tasks': ['SYNTHETIC_UNSEALED complete control'],
                'verification': ['No product claim'], 'source_freeze': {'synthetic_only': True},
                'budget': {'production_lines_max': 400, 'authored_lines_max': 900}, 'external_prs': []})
        policy = {'schema': 1, 'amendment': {'path': 'amendments/1.0.10-storage-source.md',
            'sha256': hashlib.sha256(amendment).hexdigest()}, 'base_contract_id': 'SYSON-RUST-COMPLETION/1.0.9',
            'base_contract_sha256': P.BASE_CONTRACT, 'original_unit_count': 792,
            'original_ids_sha256': P.BASE_IDS, 'original_normalized_units_sha256': P.BASE_UNITS,
            'original_readiness_graph_sha256': P.BASE_GRAPH, 'historical_unit_count': 55,
            'historical_units_sha256': P.HISTORY, 'preserved_inputs': dict(P.PRESERVED),
            'parent_intent': {'RR13': ['RR13a', 'RR13b']}, 'children': {}, 'record_updates': {}, 'graph_additions': {}}
        cls.files.update({'lanes/storage-source.json': P.encode(lane), 'storage-policy-source.json': P.encode(policy),
                          'amendments/1.0.10-storage-source.md': amendment})
        cls.proposal = P.derive(cls.files)
        cls.current = dict(cls.files)
        for name in P.EXTRA:
            path = ROOT / name
            cls.current.setdefault(name, path.read_bytes() if path.is_file() else
                                   b'SYNTHETIC_UNSEALED inactive adapter binding; never executed\n')
        cls.guard_path = ROOT / 'composition/storage_split_guard.py'
        cls.pending_guard = cls.guard_path.read_bytes()
        cls.sealed_guard = cls.pending_guard.replace(b"POLICY_SHA256 = 'PENDING_ROOT_FREEZE'",
            ("POLICY_SHA256 = '" + P.digest(cls.proposal['artifacts']['policy.json']) + "'").encode())
        cls.current['composition/storage_split_guard.py'] = cls.sealed_guard
        cls.current.update({name: P.encode(value) for name, value in cls.proposal['artifacts'].items()
                            if name in ['completion-contract.json', 'acceptance-packet.schema.json']})
        cls.current['plan.json'] = P.encode(cls.proposal['artifacts']['expected-plan.json'])
        for uid in ['RR13a', 'RR13b']:
            cls.current['prs/' + uid + '.md'] = b'# SYNTHETIC_UNSEALED brief; no adopted work\n'
        cls.current['completion-contract.lock.json'] = P.encode(P.contract_lock(cls.current, cls.old_lock)['artifacts']['completion-contract.lock.json'])

    def snapshot(self, files=None, progress_inputs=()):
        files = self.current if files is None else files
        read_bytes = Path.read_bytes
        def synthetic_guard_read(path):
            return self.sealed_guard if path == self.guard_path else read_bytes(path)
        with patch.object(Path, 'read_bytes', synthetic_guard_read):
            return P.snapshot(files, self.old_register, self.old_snapshot, self.old_lock,
                'SYSON-RUST-REMAINING/SYNTHETIC_UNSEALED', 'a' * 40, progress_inputs)['artifacts']

    def test_strict_json_paths_and_real_aggregate_refusal(self):
        for raw in [b'{"x":1,"x":2}', b'NaN', b'Infinity', b'{"x":1e999}']:
            with self.subTest(raw=raw), self.assertRaises(ValueError):
                P.parse(raw)
        for files in [{'../escape': b''}, {'a//b': b''}, {'/absolute': b''}, {'a': 'text'},
                      {'a': b'x' * (64 * 1024**2 + 1)}]:
            with self.subTest(keys=list(files)), self.assertRaises(ValueError):
                P.inputs(files)

    def test_full_contract_and_all792_records_restore_without_shared_mutation(self):
        before = dict(self.files)
        actual = P.derive(self.files)
        self.assertEqual(actual, self.proposal)
        self.assertEqual(self.files, before)
        artifacts = actual['artifacts']
        contract = copy.deepcopy(artifacts['completion-contract.json'])
        del contract['storage_source_splits']
        contract.update(version='1.0.9', contract_id='SYSON-RUST-COMPLETION/1.0.9')
        contract['unit_ids'] = [u for u in contract['unit_ids'] if u not in ['RR13a', 'RR13b']]
        contract['families']['repairs'] = [u for u in contract['families']['repairs'] if u not in ['RR13a', 'RR13b']]
        self.assertEqual(contract, P.parse(self.files['completion-contract.json']))
        old = {u['id']: u for u in P.parse(self.files['plan.json'])['units']}
        policy = artifacts['policy.json']
        for unit in artifacts['expected-plan.json']['units']:
            if unit['id'] not in old:
                continue
            restored = copy.deepcopy(unit)
            for field, change in policy['record_updates'].get(unit['id'], {}).items():
                if change['before']['present']:
                    restored[field] = change['before']['value']
                else:
                    del restored[field]
            self.assertEqual({k: v for k, v in restored.items() if k not in P.OMIT},
                             {k: v for k, v in old[unit['id']].items() if k not in P.OMIT})
        restored_graph = copy.deepcopy(artifacts['expected-plan.json']['readiness_graph'])
        for node, added in policy['graph_additions'].items():
            if node.split(':', 1)[1] in ['RR13a', 'RR13b']:
                del restored_graph[node]
            else:
                restored_graph[node] = [edge for edge in restored_graph[node] if edge not in added]
        self.assertEqual(restored_graph, P.parse(self.files['plan.json'])['readiness_graph'])
        artifacts['expected-plan.json']['units'][0]['tasks'] = []
        self.assertEqual(P.derive(self.files), self.proposal)

    def test_old_unknown_fields_hold_bytes_and_new_amendment_cannot_disappear(self):
        for name in ['plan.json', 'completion-contract.json', 'execution.json', 'teams.json',
                     'rereview-batch11-coordinator-ownership.json', 'amendments/1.0.10-storage-source.md']:
            changed = dict(self.files)
            changed[name] += b'\n'
            if name.endswith('.json') and name != 'plan.json':
                value = P.parse(changed[name])
                target = value['units']['RR13'] if name == 'execution.json' else value
                target['unknown_future_obligation'] = 'must not be ignored'
                changed[name] = P.encode(value)
            with self.subTest(name=name), self.assertRaises(ValueError):
                P.derive(changed)

    def test_child_full_fields_caps_and_independent_typed_cycle(self):
        for mutation in ['tasks', 'caps', 'cycle', 'dependency']:
            files = dict(self.files)
            lane = P.parse(files['lanes/storage-source.json'])
            child = lane['units'][0]
            if mutation == 'tasks':
                child['tasks'] = []
            if mutation == 'caps':
                child['budget']['production_lines_max'] = True
            if mutation == 'cycle':
                child['acceptance_gates'] = ['RR13']
            if mutation == 'dependency':
                child['dependencies'] = ['UNKNOWN']
            files['lanes/storage-source.json'] = P.encode(lane)
            message = 'typed cycle' if mutation == 'cycle' else '.'
            with self.subTest(mutation=mutation), self.assertRaisesRegex(ValueError, message):
                P.derive(files)
        lane = P.parse(self.files['lanes/storage-source.json'])
        lane['units'][0]['future_contract'] = {'retain': ['all', 'fields']}
        files = dict(self.files, **{'lanes/storage-source.json': P.encode(lane)})
        unit = next(u for u in P.derive(files)['artifacts']['expected-plan.json']['units'] if u['id'] == 'RR13a')
        self.assertEqual(unit['future_contract'], {'retain': ['all', 'fields']})

    def test_exact_contract_lock_covers_all_prior_paths_and_new_source(self):
        result = P.parse(self.current['completion-contract.lock.json'])
        self.assertEqual(set(result['sha256']), set(P.parse(self.old_lock)['sha256']) | P.EXTRA)
        for name, pin in result['sha256'].items():
            self.assertEqual(pin, hashlib.sha256(self.current[name]).hexdigest())
        files = dict(self.current)
        del files['composition/registration/prepare.py']
        with self.assertRaises(ValueError): P.contract_lock(files, self.old_lock)
        with self.assertRaises(ValueError): P.contract_lock(self.current, self.old_lock + b'\n')

    def test_snapshot_has_every_field_brief_full_history_and_exact_lockset(self):
        before = dict(self.current)
        artifacts = self.snapshot()
        register, lock = artifacts['units.json'], artifacts['snapshot.lock.json']
        plan, old = P.parse(self.current['plan.json']), P.parse(self.old_register)
        changed = {'snapshot_id', 'contract_id', 'base_main', 'unit_count', 'copied_plan_fields', 'units', 'integrated_scope_at_snapshot'}
        self.assertEqual({k: v for k, v in register.items() if k not in changed}, {k: v for k, v in old.items() if k not in changed})
        self.assertEqual(len(register['units']), 794)
        history = P.parse(self.current['execution.json'])['units']
        derived = {'primary_family', 'brief', 'brief_sha256', 'complete_plan_unit_sha256', 'feature_acceptance'}
        self.assertEqual(register['copied_plan_fields'], sorted({key for unit in plan['units'] for key in unit}))
        self.assertEqual([row['id'] for row in register['units']], [unit['id'] for unit in plan['units']])
        for unit, row in zip(plan['units'], register['units']):
            self.assertEqual(set(row), set(unit) | derived)
            self.assertEqual({key: value for key, value in row.items() if key not in derived},
                             {key: value for key, value in unit.items() if key not in derived})
            self.assertEqual({key: row[key] for key in derived}, {
                'primary_family': unit['completion_contract']['family'],
                'brief': '../prs/' + unit['id'] + '.md',
                'brief_sha256': hashlib.sha256(self.current['prs/' + unit['id'] + '.md']).hexdigest(),
                'complete_plan_unit_sha256': hashlib.sha256(json.dumps(unit, sort_keys=True,
                    separators=(',', ':'), ensure_ascii=False).encode()).hexdigest(),
                'feature_acceptance': history.get(unit['id'], {}).get('feature_acceptance', 'OPEN_NO_ACCEPTANCE_RECORD')})
        source = next(unit for unit in plan['units'] if unit['id'] == 'M06a1')
        projected = next(row for row in register['units'] if row['id'] == 'M06a1')
        self.assertEqual(source['feature_acceptance'], 'OPEN')
        self.assertNotIn('M06a1', history)
        self.assertEqual(projected['feature_acceptance'], 'OPEN_NO_ACCEPTANCE_RECORD')
        self.assertNotEqual(projected['complete_plan_unit_sha256'],
                            P.digest(dict(source, feature_acceptance=projected['feature_acceptance'])))
        self.assertEqual(self.current, before)
        expected = set(P.parse(self.old_snapshot)['progress_inputs']) | set(P.parse(self.current['completion-contract.lock.json'])['sha256'])
        expected.update(['README.md', 'REMAINING-WORK.md', 'plan.json', 'execution.json', 'dispatch.json', 'PR-INDEX.md', 'completion-contract.lock.json', 'remaining-work/units.json', 'remaining-work/verify_snapshot.py', 'team-dispatch.json'])
        expected.update('prs/' + u['id'] + '.md' for u in plan['units'])
        self.assertEqual(set(lock['sha256']), expected)
        self.assertEqual(lock['sha256']['remaining-work/units.json'], hashlib.sha256(P.encode(register)).hexdigest())

    def test_explicit_new_progress_keeps_all_prior_inputs_and_unknown_lock_metadata(self):
        files = dict(self.current, **{'new-evidence.json': b'{"synthetic":true}\n'})
        lock = self.snapshot(files, ['new-evidence.json'])['snapshot.lock.json']
        old = P.parse(self.old_snapshot)
        self.assertEqual(set(lock['progress_inputs']), set(old['progress_inputs']) | {'new-evidence.json'})
        self.assertEqual(lock['sha256']['new-evidence.json'], hashlib.sha256(files['new-evidence.json']).hexdigest())
        changed = {'snapshot_id', 'contract_id', 'base_main', 'sha256', 'progress_inputs'}
        self.assertEqual({k: v for k, v in lock.items() if k not in changed}, {k: v for k, v in old.items() if k not in changed})

    def test_snapshot_refuses_lost_provider_pin_pointer_history_and_stale_prior(self):
        for name, edit in [('completion-contract.lock.json', lambda v: v['sha256'].pop('composition/storage_split_guard.py')),
                           ('plan.json', lambda v: v['units'][0]['completion_contract'].update(id='old')),
                           ('execution.json', lambda v: v['units'].pop('RR13'))]:
            files = dict(self.current)
            value = P.parse(files[name])
            edit(value)
            files[name] = P.encode(value)
            with self.subTest(name=name), self.assertRaises(ValueError): self.snapshot(files)
        with self.assertRaises(ValueError):
            P.snapshot(self.current, self.old_register + b'\n', self.old_snapshot, self.old_lock, 'new', 'a' * 40)

    def relock(self, files):
        files['completion-contract.lock.json'] = P.encode(P.contract_lock(files, self.old_lock)
            ['artifacts']['completion-contract.lock.json'])
        return files

    def test_snapshot_rejects_coherent_record_hold_and_policy_changes(self):
        for mutation in ['old_tasks', 'held_repin', 'policy_repin']:
            files = dict(self.current)
            if mutation == 'old_tasks':
                plan = P.parse(files['plan.json'])
                next(u for u in plan['units'] if u['id'] == 'M06a1')['tasks'] = []
                files['plan.json'] = P.encode(plan)
            else:
                teams = P.parse(files['teams.json'])
                teams['unknown_future_obligation'] = 'coherently repinned held change'
                files['teams.json'] = P.encode(teams)
                if mutation == 'policy_repin':
                    contract = P.parse(files['completion-contract.json'])
                    contract['storage_source_splits']['preserved_inputs']['teams.json'] = P.sha(files['teams.json'])
                    files['completion-contract.json'] = P.encode(contract)
            self.relock(files)
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                self.snapshot(files)

    def test_whole_execution_attribution_is_pinned_in_both_entry_points(self):
        for key in ['current_integration', 'unknown_future_attribution']:
            for baseline in [self.files, self.current]:
                files = dict(baseline)
                execution = P.parse(files['execution.json'])
                old_units = copy.deepcopy(execution['units'])
                execution[key] = {'changed': 'SYNTHETIC_UNSEALED'}
                self.assertEqual(execution['units'], old_units)
                files['execution.json'] = P.encode(execution)
                if baseline is self.current:
                    self.relock(files)
                with self.subTest(key=key, current=baseline is self.current), self.assertRaisesRegex(ValueError, 'execution attribution'):
                    self.snapshot(files) if baseline is self.current else P.derive(files)
        register = self.snapshot()['units.json']
        self.assertEqual(register['integrated_scope_at_snapshot'],
                         P.parse(self.files['execution.json'])['current_integration'])

    def test_input_metadata_reserved_namespace_and_early_prior_bounds(self):
        with patch.object(P, 'LIMIT', 100), patch.object(P, 'sha', side_effect=AssertionError('hashed before bounds')):
            for files in [{'a' * 100: b''}, {'a': b'', 'b': b''}]:
                with self.subTest(files=files), self.assertRaises(ValueError):
                    P.inputs(files)
        for name in ['prior', 'prior/contract-lock.json', 'prior/register.json', 'prior/future.json']:
            files = dict(self.current, **{name: b'collision'})
            with self.subTest(name=name), patch.object(P, 'sha', side_effect=AssertionError('hashed collision')):
                with self.assertRaisesRegex(ValueError, 'reserved prior'):
                    P.contract_lock(files, self.old_lock)
                with self.assertRaisesRegex(ValueError, 'reserved prior'):
                    P.snapshot(files, self.old_register, self.old_snapshot, self.old_lock, 'new', 'a' * 40)
        with patch.object(P, 'LIMIT', 100), patch.object(P, 'sha', side_effect=AssertionError('hashed before aggregate')):
            with self.assertRaises(ValueError):
                P.contract_lock({}, b'x' * 101)
            with self.assertRaises(ValueError):
                P.snapshot({}, b'', b'', b'x' * 101, 'new', 'a' * 40)
        with patch.object(P, 'parse', side_effect=AssertionError('parsed before old lock pin')):
            with self.assertRaisesRegex(ValueError, 'old contract lock'):
                P.snapshot(self.current, self.old_register, self.old_snapshot, self.old_lock + b'\n', 'new', 'a' * 40)

    def test_default_pending_and_provider_raw_closure_refuse(self):
        files = dict(self.current, **{'composition/storage_split_guard.py': self.pending_guard})
        self.relock(files)
        with self.assertRaisesRegex(ValueError, 'sealed policy'):
            P.snapshot(files, self.old_register, self.old_snapshot, self.old_lock,
                       'SYSON-RUST-REMAINING/SYNTHETIC_UNSEALED', 'a' * 40)
        self.assertEqual(self.guard_path.read_bytes(), self.pending_guard)
        self.snapshot()
        self.assertEqual(self.guard_path.read_bytes(), self.pending_guard)
        for name in ['composition/storage_split_guard.py', 'tools/preparation_splits.py']:
            files = dict(self.current)
            files[name] += b'\n'
            self.relock(files)
            with self.subTest(name=name), self.assertRaisesRegex(ValueError, 'actual provider'):
                self.snapshot(files)
        files = dict(self.current)
        files['composition/storage_split_guard.py'] = self.pending_guard
        self.relock(files)
        with self.assertRaisesRegex(ValueError, 'actual provider'):
            self.snapshot(files)

    def test_provider_body_and_seal_syntax_validate_before_execution(self):
        pending_files = dict(self.current, **{'composition/storage_split_guard.py': self.pending_guard})
        pending_provider = P.storage_guard(pending_files)
        self.assertEqual(pending_provider.POLICY_SHA256, 'PENDING_ROOT_FREEZE')
        read_bytes = Path.read_bytes
        invalid_sources = [self.pending_guard + b'\n',
            self.pending_guard.replace(b'PENDING_ROOT_FREEZE', b'A' * 64),
            self.pending_guard + b"POLICY_SHA256 = 'PENDING_ROOT_FREEZE'\n"]
        for raw in invalid_sources:
            files = dict(self.current, **{'composition/storage_split_guard.py': raw})
            def synthetic_read(path):
                return raw if path == self.guard_path else read_bytes(path)
            with self.subTest(pin=P.sha(raw)), patch.object(Path, 'read_bytes', synthetic_read), \
                    patch('builtins.compile', side_effect=AssertionError('executed unverified source')):
                with self.assertRaises(ValueError):
                    P.storage_guard(files)
        artifacts = self.snapshot()
        with patch.object(Path, 'read_bytes', lambda path: self.sealed_guard if path == self.guard_path else read_bytes(path)):
            proposal = P.snapshot(self.current, self.old_register, self.old_snapshot, self.old_lock,
                'SYSON-RUST-REMAINING/SYNTHETIC_UNSEALED', 'a' * 40)
        self.assertEqual(proposal['artifacts'], artifacts)
        for name in ['composition/storage_split_guard.py', 'tools/preparation_splits.py']:
            self.assertEqual(proposal['input_sha256'][name], P.sha(self.current[name]))
        self.assertIn((P.sha(self.current['tools/preparation_splits.py']), P.sha(self.sealed_guard)), P._PROVIDERS)
        self.assertIn((P.sha(self.current['tools/preparation_splits.py']), P.sha(self.pending_guard)), P._PROVIDERS)

    def test_import_is_passive_and_emit_is_fresh_private_unsigned_bounded(self):
        source = Path(P.__file__).read_bytes()
        with patch.object(Path, 'read_bytes', side_effect=AssertionError('import IO')):
            exec(compile(source, P.__file__, 'exec'), types.ModuleType('passive').__dict__)
        self.assertEqual(P.OUTPUT_ROOT, Path('/workspace/build/builderbob'))
        root = Path(os.environ['RUNNER_TEMP']) if os.environ.get('GITHUB_ACTIONS') == 'true' else Path('/workspace/build/builderbob')
        with tempfile.TemporaryDirectory(prefix='registration-controls-', dir=root) as temporary, patch.object(P, 'OUTPUT_ROOT', Path(temporary)):
            out = Path(temporary) / 'candidate'
            P.emit(out, {'status': 'UNSEALED_PROPOSAL_ONLY', 'input_sha256': {}, 'artifacts': {'policy.json': {'synthetic': True}}})
            self.assertEqual(out.stat().st_mode & 0o777, 0o700)
            self.assertEqual(P.parse((out / 'manifest.json').read_bytes())['status'], 'UNSEALED_PROPOSAL_ONLY')
            with self.assertRaises(ValueError): P.emit(out, self.proposal)
            link = Path(temporary) / 'link'
            link.symlink_to(out, target_is_directory=True)
            with self.assertRaises(ValueError): P.emit(link / 'escape', self.proposal)
            bad = {'status': 'UNSEALED_PROPOSAL_ONLY', 'input_sha256': {}, 'artifacts': {'policy.json': 'x' * (64 * 1024**2)}}
            with self.assertRaises(ValueError): P.emit(Path(temporary) / 'oversize', bad)
            self.assertFalse((Path(temporary) / 'oversize').exists())
