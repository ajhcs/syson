"""Full fixed baseline controls; only root-admitted tests execute these programs."""
import copy
import hashlib
import json
import os
import sys
import types
import unittest
from pathlib import Path
from unittest import mock


def modules():
    directory = Path(__file__).resolve().parent
    package = types.ModuleType('fixed_identity_partition_tests')
    package.__path__ = [str(directory)]
    if package.__name__ in sys.modules:
        raise ValueError('test namespace collision')
    sys.modules[package.__name__] = package
    for name, expected in [
            ('derive', '4bfa0cf3080e341efc342fbda38ac674ee106e99ff0d123f98f63861442fe1b4'),
            ('guard', '125b82688181a1e8a89e3b129530761baafeec32d61ea4858e95e7592eed462c')]:
        path = directory / (name + '.py')
        raw = path.read_bytes()
        if (path.resolve() != path or path.is_symlink() or len(raw) > 65536
                or hashlib.sha256(raw).hexdigest() != expected):
            raise ValueError('whole actual test subject source changed')
        module = types.ModuleType(package.__name__ + '.' + name)
        module.__file__, module.__package__ = str(path), package.__name__
        sys.modules[module.__name__] = module
        setattr(package, name, module)
        exec(compile(raw, str(path), 'exec'), module.__dict__)
    return package.derive, package.guard


D, G = modules()
_CACHE = None


class Fixture:
    @classmethod
    def setUpClass(cls):
        global _CACHE
        if _CACHE is None:
            directory = Path(D.__file__).resolve().parent
            fixture = Path(os.environ['SYSON_IDENTITY_PARTITION_FIXTURES'])
            if fixture.resolve() != fixture or fixture != directory / 'fixtures':
                raise ValueError('declared fixed sibling fixture root required')
            root = directory.parents[1]
            lock = json.loads((root / 'completion-contract.lock.json').read_bytes())
            files = {name: (root / name).read_bytes() for name in set(lock['sha256']) | D.EXTRA}
            args = tuple((fixture / name).read_bytes() for name in ['lane.json', 'amendment.md'])
            proposed_generator = files['tools/build_plan.py'] + (
                b'\n# unsigned .12 generator byte claim; never executed\n')
            args += (proposed_generator,)
            result = D.derive(files, *args)
            _CACHE = files, args, result
        cls.files, cls.args, cls.result = _CACHE
        cls.artifacts = cls.result['artifacts']
        cls.contract = cls.artifacts['completion-contract.json']
        cls.plan = cls.artifacts['expected-plan.json']
        cls.old = json.loads(cls.files['plan.json'])


class DeriveTests(Fixture, unittest.TestCase):
    def test_complete799_model_and_all_original_records_and_gates(self):
        self.assertEqual((self.plan['unit_count'], len(self.plan['readiness_graph'])), (799, 1598))
        self.assertEqual(self.result['status'], 'UNSEALED_PROPOSAL_ONLY')
        now = {u['id']: u for u in self.plan['units']}
        changes = self.artifacts['policy.json']['record_updates']
        for before in self.old['units']:
            changed = set(changes.get(before['id'], {})) | {'completion_contract', 'source_fragment_sha256'}
            self.assertEqual({k: v for k, v in before.items() if k not in changed},
                             {k: v for k, v in now[before['id']].items() if k not in changed})
        original = next(u for u in self.old['units'] if u['id'] == 'M06a2')
        self.assertEqual(now['M06a2']['acceptance_gates'], original['acceptance_gates'])
        for key in ['dependencies', 'implementation_ready_dependencies',
                    'acceptance_children', 'source_preparation_children']:
            self.assertEqual(now['M06a2'][key], sorted(set(original.get(key, []) + D.IDS)))
        self.assertEqual(now['M06a2a']['dependencies'], ['M06a1', 'RR13a'])
        self.assertEqual(now['M06a2b']['dependencies'], ['M06a2a'])
        self.assertEqual(now['M06']['tasks'], next(u for u in self.old['units'] if u['id'] == 'M06')['tasks'])
        for node, edges in self.old['readiness_graph'].items():
            self.assertTrue(set(edges) <= set(self.plan['readiness_graph'][node]))
        self.assertEqual(self.plan['unordered_write_conflicts'], [])
        self.assertEqual(self.artifacts['policy.json']['prior_progress_count'], 342)
        self.assertEqual(self.artifacts['policy.json']['historical_unit_count'], 55)

    def test_complete_input_and_actual_provider_provenance(self):
        pins = self.result['input_sha256']
        for name, raw in self.files.items():
            self.assertEqual(pins['supplied-baseline/' + name], D.sha(raw))
        self.assertEqual(pins['proposed/current-generator.py'], D.sha(self.args[2]))
        c99 = next(u for u in self.plan['units'] if u['id'] == 'C99')
        self.assertEqual(c99['source_fragment_sha256'], D.sha(self.args[2]))
        schema = self.artifacts['acceptance-packet.schema.json']
        self.assertEqual(schema['properties']['contract_id']['const'], D.VERSION)
        self.assertEqual(len(self.contract['unit_ids']), len(set(self.contract['unit_ids'])))
        self.assertEqual(self.contract['identity_partition_splits']['prior_progress_sha256'], D.PROGRESS)
        for name in ['derive.py', 'guard.py', 'construct.py']:
            self.assertIn('actual-provider/' + name, pins)
        self.assertIn('actual-entry/derive.py', pins)
        self.assertEqual(G.POLICY_SHA256, 'PENDING_ROOT_FREEZE')

    def test_missing_extra_and_changed_baseline_inputs_refuse(self):
        for mutation in ['missing', 'extra', 'changed']:
            files = dict(self.files)
            if mutation == 'missing':
                del files['teams.json']
            elif mutation == 'extra':
                files['extra.json'] = b'{}'
            else:
                files['execution.json'] += b'\n'
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                D.derive(files, *self.args)

    def test_coherently_rehashed_normative_baseline_still_refuses(self):
        files = dict(self.files)
        lock = json.loads(files['completion-contract.lock.json'])
        files['execution.json'] += b'\n'
        lock['sha256']['execution.json'] = D.sha(files['execution.json'])
        files['completion-contract.lock.json'] = json.dumps(lock).encode()
        with self.assertRaisesRegex(ValueError, 'whole fixed baseline'):
            D.derive(files, *self.args)

    def test_fixed_complete_child_body_survives_envelope_rehash_control(self):
        lane = json.loads(self.args[0])
        lane['units'][1]['tasks'].pop()
        raw = json.dumps(lane).encode()
        with mock.patch.object(D, 'LANE_SHA', D.sha(raw)), self.assertRaisesRegex(
                ValueError, 'whole fixed child'):
            D.derive(self.files, raw, *self.args[1:])

    def test_changed_amendment_and_empty_or_overlong_generator_refuse(self):
        for args in [(self.args[0], self.args[1] + b'\n', self.args[2]),
                     (*self.args[:2], b''), (*self.args[:2], b'x' * 262145)]:
            with self.subTest(lengths=tuple(map(len, args))), self.assertRaises(ValueError):
                D.derive(self.files, *args)

    def test_current_actual_old_provider_pin_and_namespace_are_enforced(self):
        with mock.patch.dict(D.SOURCE_PINS, {'construct': '0' * 64}), self.assertRaisesRegex(
                ValueError, 'whole old provider'):
            D.provider()
        with mock.patch.object(D, '_OLD', None), self.assertRaisesRegex(ValueError, 'namespace collision'):
            D.provider()

    def test_exact_generator_byte_boundary_is_a_claim_not_execution(self):
        generator = b'x' * 262144
        result = D.derive(self.files, *self.args[:2], generator)
        c99 = next(u for u in result['artifacts']['expected-plan.json']['units'] if u['id'] == 'C99')
        self.assertEqual(c99['source_fragment_sha256'], D.sha(generator))
        self.assertEqual(result['input_sha256']['proposed/current-generator.py'], D.sha(generator))

    def test_wrong_byte_type_and_aggregate_input_bound_refuse_before_parse(self):
        files = dict(self.files)
        files['plan.json'] = 'not bytes'
        with self.assertRaisesRegex(ValueError, 'byte map'):
            D.derive(files, *self.args)
        files['plan.json'] = b'x' * (D.LIMIT + 1)
        with self.assertRaisesRegex(ValueError, 'aggregate input limit'):
            D.derive(files, *self.args)

    def test_hostile_logical_name_refuses_before_provider_or_serialization(self):
        files = dict(self.files)
        threshold = max(map(len, files)) + 1
        name = next(iter(files))
        files['x' * (threshold + 1)] = files.pop(name)
        with mock.patch.object(D, 'NAME_LIMIT', threshold), mock.patch.object(
                D, 'provider', side_effect=AssertionError('provider entered before admission')):
            with self.assertRaisesRegex(ValueError, 'input name length limit'):
                D.baseline(files)

    def test_name_framing_refuses_in_isolation_before_provider_or_serialization(self):
        files = {name: b'' for name in self.files}
        required = sum(128 + 6 * len(name) for name in files)
        with mock.patch.object(D, 'LIMIT', required - 1), mock.patch.object(
                D, 'provider', side_effect=AssertionError('provider entered before admission')):
            with self.assertRaisesRegex(ValueError, 'input name framing limit'):
                D.baseline(files)
