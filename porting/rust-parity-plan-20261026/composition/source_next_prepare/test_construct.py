"""Complete staged797 construction; proposed roles never claim physical adoption."""
from pathlib import Path
import copy
import hashlib
import importlib.util
import json
import os
import sys
import tempfile
import types
import unittest
from unittest.mock import patch

DIRECTORY = Path(__file__).resolve().parent
PIN = '89d56e9bde5aa8627346feb7e50ea7447572f0e452d1e74e95d40dc148b682a9'


def modules():
    path = DIRECTORY / 'test_guard.py'
    raw = path.read_bytes()
    if hashlib.sha256(raw).hexdigest() != '65b12e7d1c5475c1cb7b5800f72919c96edcf6ddfca427cab6c16248edfc20bf':
        raise ValueError('whole existing bootstrap source changed')
    spec = importlib.util.spec_from_file_location('fixed_constructor_existing_bootstrap', path)
    bootstrap = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(bootstrap)
    D, G = bootstrap.modules()
    path = DIRECTORY / 'construct.py'
    raw = path.read_bytes()
    if path.resolve() != path or path.is_symlink() or len(raw) > 65536 or hashlib.sha256(raw).hexdigest() != PIN:
        raise ValueError('whole canonical constructor source changed')
    C = types.ModuleType(G.__package__ + '.construct')
    C.__file__, C.__package__ = str(path), G.__package__
    sys.modules[C.__name__] = C
    exec(compile(raw, str(path), 'exec'), C.__dict__)
    return D, G, C


class ConstructorTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.D, cls.G, cls.C = modules()
        cls.root = Path(cls.C.ROOT)
        fixture = Path(os.environ['SYSON_SOURCE_NEXT_FIXTURES'])
        if fixture.resolve() != fixture or fixture != DIRECTORY / 'fixtures':
            raise ValueError('fixed existing fixture directory required')
        cls.P, _ = cls.D.provider()
        args = [(fixture / n).read_bytes() for n in ['baseline-1.0.10.json.gz',
                'baseline-provenance.json', 'lane.json', 'amendment.md']]
        args += [(cls.root / 'tools/build_plan.py').read_bytes(), (cls.root / cls.D.OWNER).read_bytes()]
        cls.artifacts = cls.D.derive(*args)['artifacts']
        cls.seal = cls.G.digest(cls.artifacts['policy.json'])
        cls.prior_contract = (cls.root / cls.C.PRIOR / 'contract-lock.json').read_bytes()
        cls.prior_snapshot = (cls.root / cls.C.PRIOR / 'snapshot-lock.json').read_bytes()
        cls.prior_register = (cls.root / cls.C.PRIOR / 'register.json').read_bytes()
        cls.old_lock = cls.P.parse(cls.prior_snapshot)
        cls.old_normative = cls.P.parse(cls.prior_contract)
        cls.current = {n: (cls.root / n).read_bytes() for n in cls.old_lock['sha256']
                       if n != 'remaining-work/units.json'}
        for target, artifact in [('plan.json', 'expected-plan.json'), ('completion-contract.json',
                'completion-contract.json'), ('acceptance-packet.schema.json', 'acceptance-packet.schema.json')]:
            cls.current[target] = cls.P.encode(cls.artifacts[artifact])
        cls.current[cls.D.FRAGMENT], cls.current['amendments/1.0.11-source-next.md'] = args[2:4]
        for name in cls.C.ADDITIONS - {cls.D.FRAGMENT, 'amendments/1.0.11-source-next.md', 'tools/source_next_splits.py'}:
            cls.current[name] = (cls.root / name).read_bytes()
        cls.current['tools/source_next_splits.py'] = b'PROPOSED_LOGICAL_ROLE_ONLY; NO_LOADER_IMPLEMENTATION\n'
        for name in cls.C.FIXTURES:
            cls.current[name] = (cls.root / name).read_bytes()
        for name in cls.C.PROOFS:
            short, relative = name[len(cls.C.BASE + 'proof/'):].split('/', 1)
            packet = {'derive': 'batch39-source-next-derive', 'guard': 'batch40-source-next-guard'}[short]
            cls.current[name] = (cls.root.parent / 'rust-parity-evidence/C03' / packet / relative).read_bytes()
        for uid in cls.D.IDS:
            cls.current['prs/' + uid + '.md'] = ('PROPOSED_BRIEF_BYTE_CLAIM ' + uid + '\n').encode()
        with patch.object(cls.G, 'POLICY_SHA256', cls.seal):
            proposal = cls.C.contract_lock(cls.lock_files(), cls.prior_contract)
            cls.current['completion-contract.lock.json'] = cls.P.encode(proposal['artifacts']['completion-contract.lock.json'])
            cls.proposal = cls.C.snapshot(cls.current, cls.prior_register, cls.prior_snapshot, cls.prior_contract,
                                         'SYSON-RUST-REMAINING/constructor-staged', '0' * 40)

    @classmethod
    def lock_files(cls):
        names = set(cls.old_normative['sha256']) | cls.C.ADDITIONS | {'plan.json', cls.D.OWNER}
        return {n: cls.current[n] for n in names}

    def setUp(self):
        root = Path(os.environ['SYSON_BOOTSTRAP_TEST_ROOT'])
        self.assertTrue(root.is_absolute() and root.resolve() == root and root.is_dir())
        if os.environ.get('GITHUB_ACTIONS') == 'true':
            self.assertTrue(root.is_relative_to(Path(os.environ['RUNNER_TEMP']).resolve()))
        else:
            self.assertTrue(root.is_relative_to('/workspace/build/builderbob'))
        temporary = tempfile.TemporaryDirectory(prefix='constructor-', dir=root)
        self.addCleanup(temporary.cleanup)
        self.output_root = Path(temporary.name)
        self.policy_patch = patch.object(self.G, 'POLICY_SHA256', self.seal)
        for mock in [self.policy_patch,
                     patch.object(self.C, 'OUTPUT_ROOT', self.output_root),
                     patch('socket.create_connection', side_effect=AssertionError('network forbidden'))]:
            mock.start()
            self.addCleanup(mock.stop)

    def build(self, files=None, **kwargs):
        args = dict(prior_register=self.prior_register, prior_snapshot_lock=self.prior_snapshot,
                    prior_contract_lock=self.prior_contract, snapshot_id='SYSON-RUST-REMAINING/constructor-new',
                    base_main='0' * 40)
        args.update(kwargs)
        return self.C.snapshot(dict(self.current) if files is None else files, **args)

    def test_pending_all_apis_refuse_before_parse_copy_provider_or_output(self):
        with patch.object(self.G, 'POLICY_SHA256', 'PENDING_ROOT_FREEZE'), \
             patch.object(self.C, 'parse', side_effect=AssertionError('pending parse')), \
             patch.object(self.C.copy, 'deepcopy', side_effect=AssertionError('pending copy')):
            for call in [lambda: self.C.contract_lock(None, b''), lambda: self.build(),
                         lambda: self.C.emit_fresh(self.output_root / 'out', {})]:
                with self.assertRaisesRegex(ValueError, 'pending'):
                    call()
        self.assertEqual(list(self.output_root.iterdir()), [])

    def test_exact60_lock_and_complete_old1167_progress318_preserved(self):
        lock = self.proposal['artifacts']['snapshot.lock.json']
        current_lock = self.P.parse(self.current['completion-contract.lock.json'])
        self.assertEqual(len(current_lock['sha256']), 60)
        self.assertEqual(set(current_lock['sha256']), set(self.old_normative['sha256']) | self.C.ADDITIONS)
        self.assertTrue(set(self.old_lock['sha256']) <= set(lock['sha256']))
        self.assertTrue(set(self.old_lock['progress_inputs']) <= set(lock['progress_inputs']))
        self.assertEqual(len(self.old_lock['sha256']), 1167)
        self.assertEqual(len(self.old_lock['progress_inputs']), 318)
        self.assertEqual(len(self.P.parse(self.current['execution.json'])['units']), 55)

    def test_all797_fields_and_only_five_overlays_match_actual_bytes(self):
        register = self.proposal['artifacts']['units.json']
        units = self.artifacts['expected-plan.json']['units']
        self.assertEqual(register['unit_count'], 797)
        self.assertEqual([u['id'] for u in register['units']], [u['id'] for u in units])
        self.assertEqual(set(register['copied_plan_fields']), {k for u in units for k in u})
        state = self.P.parse(self.current['execution.json'])
        for unit, row in zip(units, register['units']):
            expected = copy.deepcopy(unit)
            expected.update(primary_family=unit['completion_contract']['family'], brief='../prs/' + unit['id'] + '.md',
                brief_sha256=self.P.sha(self.current['prs/' + unit['id'] + '.md']),
                complete_plan_unit_sha256=self.P.digest(unit),
                feature_acceptance=state['units'].get(unit['id'], {}).get('feature_acceptance', 'OPEN_NO_ACCEPTANCE_RECORD'))
            self.assertEqual(row, expected)
        self.assertEqual(register['integrated_scope_at_snapshot'], state['current_integration'])

    def test_full_staged_output_actual_serialization_combined64_and_truthful_origins(self):
        out = self.output_root / 'complete'
        manifest = self.C.emit_fresh(out, self.proposal)
        sizes = sum(p.stat().st_size for p in out.iterdir())
        self.assertLessEqual(self.proposal.reader.used + sizes, self.C.LIMIT)
        self.assertGreater((out / 'units.json').stat().st_size, 5 * 1024**2)
        self.assertEqual(manifest['logical_input_bytes'], self.proposal.reader.used)
        for name, pin in manifest['artifacts'].items():
            self.assertEqual(self.P.sha((out / name).read_bytes()), pin)
        self.assertFalse(any(n.startswith('physical/') for n in manifest['input_sha256']))
        self.assertTrue(any('EXPLICIT_SUPPLIED_BYTE_CLAIM' in n for n in manifest['input_sha256']))
        self.assertEqual(sum(n.endswith('/prior/register.json') for n in self.proposal.reader.cache), 1)
        self.assertNotIn('remaining-work/units.json', self.proposal.reader.cache)
        self.assertEqual(json.loads((out / 'units.json').read_bytes()), self.proposal['artifacts']['units.json'])

    def test_current_files_are_not_mutated_or_referenced_as_mutable_input_dict(self):
        files = self.lock_files()
        before = {n: self.P.sha(raw) for n, raw in files.items()}
        result = self.C.contract_lock(files, self.prior_contract)
        self.assertEqual(before, {n: self.P.sha(raw) for n, raw in files.items()})
        files['plan.json'] = b'changed caller map after construction'
        self.assertNotEqual(result.reader('plan.json'), files['plan.json'])

    def test_prior_rawpins_types_and_oversize_refuse_before_parse(self):
        with patch.object(self.C, 'parse', side_effect=AssertionError('prior parse')):
            for key in ['prior_register', 'prior_snapshot_lock', 'prior_contract_lock']:
                with self.subTest(key=key), self.assertRaisesRegex(ValueError, 'fixed prior'):
                    self.build(**{key: b'wrong'})
            for raw in [None, b'x' * (8 * 1024**2 + 1)]:
                with self.assertRaisesRegex(ValueError, 'fixed prior'):
                    self.C.contract_lock(self.lock_files(), raw)

    def test_all_fixed_required_progress_and_prior_namespace_coverage_refuse(self):
        roles = ['COMPLETION-CONTRACT.md', self.D.OWNER, 'tools/source_next_splits.py',
                 self.C.PROOFS[0], next(iter(self.C.FIXTURES)), self.old_lock['progress_inputs'][0]]
        for role in roles:
            files = dict(self.current)
            del files[role]
            with self.subTest(role=role), self.assertRaisesRegex(ValueError, 'coverage'):
                self.build(files)
        for name in ['prior/x', 'physical/x', 'logical-origin', self.C.PRIOR + 'register.json', '../escape']:
            files = dict(self.current, **{name: b'extra'})
            with self.subTest(name=name), self.assertRaises(ValueError):
                self.build(files)

    def test_proof_fixture_old_progress_and_held_execution_drift_refuse(self):
        roles = [self.C.PROOFS[0], next(iter(self.C.FIXTURES)), self.old_lock['progress_inputs'][0],
                 self.D.OWNER, 'execution.json']
        for name in roles:
            files = dict(self.current)
            files[name] += b'\n'
            with self.subTest(name=name), self.assertRaises(ValueError):
                self.build(files)

    def test_coherent_current_lock_and_full_record_source_corruptions_refuse(self):
        for mode in ['lock', 'old_metadata', 'generator']:
            files = dict(self.current)
            if mode == 'lock':
                value = self.P.parse(files['completion-contract.lock.json'])
                value['sha256']['tools/source_next_splits.py'] = '0' * 64
                files['completion-contract.lock.json'] = self.P.encode(value)
            else:
                value = self.P.parse(files['plan.json'])
                unit = next(u for u in value['units'] if u['id'] == ('B02' if mode == 'old_metadata' else 'C99'))
                unit['unknown_preserved_metadata' if mode == 'old_metadata' else 'source_fragment_sha256'] = 'changed'
                files['plan.json'] = self.P.encode(value)
            with self.subTest(mode=mode), self.assertRaises(ValueError):
                self.build(files)

    def test_default_physical_root_refuses_missing_unadopted_roles(self):
        self.policy_patch.stop()
        current_lock = self.P.parse((self.root / 'completion-contract.lock.json').read_bytes())
        current_snapshot = self.P.parse((self.root / 'remaining-work/snapshot.lock.json').read_bytes())
        current_register = self.P.parse((self.root / 'remaining-work/units.json').read_bytes())
        actual = self.C.contract_lock(None, self.prior_contract)['artifacts']['completion-contract.lock.json']
        self.assertEqual(actual, current_lock)
        actual = self.C.snapshot(None, self.prior_register, self.prior_snapshot, self.prior_contract,
                                 current_snapshot['snapshot_id'], current_snapshot['base_main'])['artifacts']
        self.assertEqual(actual, {'units.json': current_register, 'snapshot.lock.json': current_snapshot})
        loader = self.root / 'tools/source_next_splits.py'
        is_file = Path.is_file
        with patch.object(Path, 'is_file', lambda path: False if path == loader else is_file(path)):
            with self.assertRaisesRegex(ValueError, 'actual canonical source'):
                self.C.contract_lock(None, self.prior_contract)

    def test_real64mib_exact_and_one_extra_before_parse_or_copy(self):
        reserve = self.C.BOUNDS['manifest.json'] + self.C.BOUNDS['units.json'] + self.C.BOUNDS['snapshot.lock.json']
        names = ['controlled/file' + str(i) for i in range(8)]
        frame = sum(len(self.C.encode({n: '0' * 64})) for n in names)
        shared = b'x' * (8 * 1024**2)
        files = {n: shared for n in names[:-1]}
        files[names[-1]] = b'x' * (self.C.LIMIT - reserve - frame - 7 * len(shared))
        reader = self.C.Reader(files, {}, reserve)
        reader.prime(set(files))
        self.assertEqual(reader.used + reserve, self.C.LIMIT)
        files[names[-1]] += b'x'
        with self.assertRaisesRegex(ValueError, 'combined input/output reservation'):
            self.C.Reader(files, {}, reserve)
        with patch.object(self.C, 'parse', side_effect=AssertionError('quota parse')), \
             patch.object(self.C.copy, 'deepcopy', side_effect=AssertionError('quota copy')):
            with self.assertRaisesRegex(ValueError, 'combined input/output reservation'):
                self.build(files)

    def test_writer_rejects_hash_only_replay_and_mutated_artifact_before_output(self):
        out = self.output_root / 'refused'
        replay = json.loads(json.dumps(self.proposal))
        with self.assertRaisesRegex(ValueError, 'live constructor result'):
            self.C.emit_fresh(out, replay)
        unit = self.proposal['artifacts']['units.json']['units'][0]
        with patch.dict(unit, {'unknown_new_metadata': 'coherent-looking changed data'}):
            with self.assertRaisesRegex(ValueError, 'constructor result changed'):
                self.C.emit_fresh(out, self.proposal)
        self.assertFalse(out.exists())

    def test_writer_existing_symlink_and_escape_refusals_preserve_prior_bytes(self):
        existing = self.output_root / 'existing'
        existing.mkdir()
        held = existing / 'held'
        held.write_bytes(b'preserve')
        link = self.output_root / 'link'
        link.symlink_to(existing, target_is_directory=True)
        for out in [existing, link, link / 'child', self.output_root.parent / 'escape']:
            with self.subTest(out=str(out)), self.assertRaises(ValueError):
                self.C.emit_fresh(out, self.proposal)
        self.assertEqual(held.read_bytes(), b'preserve')

    def test_identity_types_and_reused_snapshot_identity_refuse(self):
        for fields in [{'snapshot_id': self.old_lock['snapshot_id']}, {'snapshot_id': True},
                       {'snapshot_id': 'SYSON-RUST-REMAINING/' + 'x' * 129}, {'base_main': 'g' * 40}]:
            with self.subTest(fields=fields), self.assertRaises(ValueError):
                self.build(**fields)

    def test_actual_source_body_binding_refuses_coherently_supplied_changed_bytes(self):
        role = 'composition/source_next_prepare/guard.py'
        path = self.root / role
        files = dict(self.current)
        files[role] += b'\n# unreviewed body\n'
        original_open = Path.open
        def opened(candidate, *args, **kwargs):
            if candidate == path:
                import io
                return io.BytesIO(files[role])
            return original_open(candidate, *args, **kwargs)
        with patch.object(Path, 'open', opened), self.assertRaisesRegex(ValueError, 'fixed actual source pin'):
            self.build(files)

    def test_canonical_reader_uses_actual_bytes_once_and_early_stat_bound(self):
        name = 'data/item'
        directory = self.output_root / 'data'
        directory.mkdir()
        (directory / 'item').write_bytes(b'actual')
        with patch.object(self.C, 'ROOT', self.output_root):
            reader = self.C.Reader(None, {}, 0)
            reader.prime({name})
            with patch.object(Path, 'open', side_effect=AssertionError('duplicate read')):
                self.assertEqual(reader(name), b'actual')
                self.assertEqual(reader(name), b'actual')
            link = directory / 'link'
            link.symlink_to(directory / 'item')
            with self.assertRaisesRegex(ValueError, 'actual canonical source'):
                self.C.Reader(None, {}, 0).prime({'data/link'})
            with patch.object(self.C, 'LIMIT', 1), \
                 patch.object(Path, 'open', side_effect=AssertionError('stat refusal read')):
                with self.assertRaisesRegex(ValueError, 'early combined'):
                    self.C.Reader(None, {}, 0).prime({name})


if __name__ == '__main__':
    unittest.main()
