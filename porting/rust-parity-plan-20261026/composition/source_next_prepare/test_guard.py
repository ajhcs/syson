"""Actual full model tests; proposed logical reads are not physical adoption."""
import copy
import hashlib
import io
import os
from pathlib import Path
import re
import sys
import types
import unittest
from unittest import mock
from collections import Counter


def modules():
    directory = Path(__file__).resolve().parent
    package = types.ModuleType('fixed_source_next_guard_tests')
    package.__path__ = [str(directory)]
    if package.__name__ in sys.modules:
        raise ValueError('fixed package namespace collision')
    sys.modules[package.__name__] = package
    hashes = {'derive': 'fbb0352325dc948fdb1625348db69a4810dbfa61cc276a9c235716cecc8c9f17',
              'guard': 'eb80f827c8014cee147ed42c6efafa1f3856ff250fd2ea99a275cfd46aef12d8'}
    for name in ['derive', 'guard']:
        path = directory / (name + '.py')
        if path.resolve() != path or path.is_symlink() or path.stat().st_size > 262144:
            raise ValueError('canonical bounded actual source required')
        with path.open('rb') as stream:
            raw = stream.read(262145)
        normalized = raw
        if name == 'guard':
            seal = rb"(?m)^POLICY_SHA256 = '(?:PENDING_ROOT_FREEZE|[0-9a-f]{64})'$"
            if len(re.findall(seal, raw)) != 1:
                raise ValueError('single strict seal required')
            normalized = re.sub(seal, b"POLICY_SHA256 = 'PENDING_ROOT_FREEZE'", raw)
        if len(raw) > 262144 or hashlib.sha256(normalized).hexdigest() != hashes[name]:
            raise ValueError('actual whole source identity changed')
        module = types.ModuleType(package.__name__ + '.' + name)
        module.__file__, module.__package__ = str(path), package.__name__
        sys.modules[module.__name__] = module
        setattr(package, name, module)
        exec(compile(raw, str(path), 'exec'), module.__dict__)
    return package.derive, package.guard


def current_identity():
    path = Path(__file__).resolve().parents[2] / 'tools/identity_partition_splits.py'
    if path.resolve() != path or path.is_symlink() or not path.is_file() or path.stat().st_size > 65536:
        raise ValueError('canonical bounded current identity loader required')
    with path.open('rb') as stream:
        raw = stream.read(65537)
    if len(raw) > 65536 or hashlib.sha256(raw).hexdigest() != '10cffe7888a4ddae06f30e01a4dd87840cff3cc5b02bbe5b762910db008633b6':
        raise ValueError('whole current identity loader changed')
    name = 'identity_partition_splits'
    if name in sys.modules:
        actual = sys.modules[name]
        package = '_syson_identity_partition_1_0_12'
        expected = path.parents[1] / 'composition/identity_partition_prepare/facade.py'
        if (type(actual) is not types.ModuleType or actual.__name__ != package + '.facade' or
                sys.modules.get(actual.__name__) is not actual or getattr(actual, '__file__', None) != str(expected) or
                any(type(getattr(actual, field, None)) is not types.ModuleType or
                    sys.modules.get(package + '.' + name) is not getattr(actual, field) for name, field in
                    [('derive', 'D'), ('guard', 'G'), ('construct', 'C'), ('capture', 'H')])):
            raise ValueError('foreign current identity facade')
        return actual
    module = types.ModuleType(name)
    module.__file__ = str(path)
    sys.modules[name] = module
    exec(compile(raw, str(path), 'exec'), module.__dict__)
    return sys.modules[name]


class GuardTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.D, cls.G = modules()
        cls.root = Path(cls.G.__file__).resolve().parents[2]
        fixture = Path(os.environ['SYSON_SOURCE_NEXT_FIXTURES'])
        if fixture.resolve() != fixture or fixture != Path(cls.G.__file__).parent / 'fixtures':
            raise ValueError('declared fixed fixture directory required')
        cls.inputs = tuple((fixture / n).read_bytes() for n in
                           ['baseline-1.0.10.json.gz', 'baseline-provenance.json', 'lane.json', 'amendment.md'])
        cls.inputs += ((cls.root / 'tools/build_plan.py').read_bytes(),
                       (cls.root / cls.D.OWNER).read_bytes())
        cls.P, _ = cls.D.provider()
        cls.output = cls.D.derive(*cls.inputs)['artifacts']
        cls.contract, cls.plan = cls.output['completion-contract.json'], cls.output['expected-plan.json']
        cls.policy_sha = cls.G.digest(cls.output['policy.json'])
        cls.old_contract = cls.P.parse((cls.root / 'completion-contract.json').read_bytes())

    def setUp(self):
        self.seal = mock.patch.object(self.G, 'POLICY_SHA256', self.policy_sha)
        self.seal.start()
        self.addCleanup(self.seal.stop)

    def reader(self, overrides=None, counts=None, claims=None):
        overrides = overrides or {}
        def read(name):
            if counts is not None:
                counts[name] += 1
            if name in overrides:
                if claims is not None:
                    claims[name] = 'SYNTHETIC_BYTE_CLAIM'
                return overrides[name]
            if name == self.D.FRAGMENT:
                if claims is not None:
                    claims[name] = 'PROPOSED_LOGICAL_FROM_PINNED_EXISTING_FIXTURE'
                return self.inputs[2]
            if claims is not None:
                claims[name] = 'ACTUAL_CURRENT_DISK'
            path = self.root / name
            if path.resolve() != path or path.is_symlink() or path.stat().st_size > 8 * 1024**2:
                raise ValueError('actual bounded fixture read')
            with path.open('rb') as stream:
                return stream.read(8 * 1024**2 + 1)
        return read

    def calls(self, contract=None, plan=None, read=None):
        c = self.contract if contract is None else contract
        p = self.plan if plan is None else plan
        read = self.reader() if read is None else read
        G = self.G
        return [lambda: G.source_next_policy(c), lambda: G.previous_contract(c, read),
                lambda: G.previous_plan(c, p, read), lambda: G.validate_source_next_splits(c, p, read),
                lambda: G.legacy_contract(c, read), lambda: G.legacy_plan(c, p, read),
                lambda: G.validate_preparation_splits(c, p, read),
                lambda: G.storage_policy(c, read), lambda: G.validate_storage_splits(c, p, read)]

    def deny(self, change):
        contract, plan = copy.deepcopy(self.contract), copy.deepcopy(self.plan)
        change(contract, plan)
        with self.assertRaises(ValueError):
            self.G.validate_source_next_splits(contract, plan, self.reader())

    def test_pending_all_new_apis_refuse_before_context_or_copy(self):
        with mock.patch.object(self.G, 'POLICY_SHA256', 'PENDING_ROOT_FREEZE'):
            with mock.patch.object(self.G, 'Session', side_effect=AssertionError('pending I/O')), \
                 mock.patch.object(self.G.copy, 'deepcopy', side_effect=AssertionError('pending copy')), \
                 mock.patch.object(self.G.json, 'dumps', side_effect=AssertionError('pending serialization')):
                for call in self.calls():
                    with self.assertRaisesRegex(ValueError, 'pending'):
                        call()

    def test_full_positive_private_views_and_complete_old_metadata(self):
        G = self.G
        counts, claims = Counter(), {}
        policy = G.validate_source_next_splits(self.contract, self.plan,
                                             self.reader(counts=counts, claims=claims))
        self.assertTrue(all(n == 1 for n in counts.values()), 'one current read per role')
        self.assertTrue(set(self.P.PRESERVED) | {'execution.json'} <= set(counts))
        self.assertEqual(claims[self.D.FRAGMENT], 'PROPOSED_LOGICAL_FROM_PINNED_EXISTING_FIXTURE')
        self.assertEqual(policy, self.output['policy.json'])
        old = G.previous_plan(self.contract, self.plan, self.reader())
        self.assertEqual(G.digest(old), G.PLANS['1.0.10'][0])
        c10 = G.previous_contract(self.contract, self.reader())
        self.assertEqual(G.digest(c10), self.D.BASE_CONTRACT)
        session = G.historical(c10, old, self.reader())
        c9 = session.S.previous_contract(c10)
        p9 = G.finish(session.P, session.S.previous_plan(c10, old), '1.0.9')
        self.assertEqual(G.digest(p9), G.PLANS['1.0.9'][0])
        p8 = G.legacy_plan(self.contract, self.plan, self.reader())
        self.assertEqual(G.digest(p8), G.PLANS['1.0.8'][0])
        self.assertEqual(G.legacy_contract(self.contract, self.reader())['version'], '1.0.8')
        self.assertEqual(G.digest(G.legacy_plan(c9, p9, self.reader())), G.PLANS['1.0.8'][0])
        self.assertEqual(session.P.CHILDREN, ['RR13a', 'RR13b'])
        self.assertEqual(session.P.NEW_CONTRACT, 'SYSON-RUST-COMPLETION/1.0.10')
        self.assertEqual(len(G.validate_preparation_splits(self.contract, self.plan,
                         self.reader())['children']), 7)
        self.assertEqual(G.storage_policy(self.contract, self.reader()),
                         G.validate_storage_splits(self.contract, self.plan, self.reader()))
        old['units'][0]['tasks'] = []
        policy['children'].clear()
        self.assertEqual(G.digest(G.previous_plan(self.contract, self.plan, self.reader())),
                         G.PLANS['1.0.10'][0])

    def test_default_reader_is_not_a_proposed_lane_adoption(self):
        if not (self.root / self.D.FRAGMENT).exists():
            with self.assertRaisesRegex(ValueError, 'canonical actual input'):
                self.G.previous_plan(self.contract, self.plan)
        else:
            self.seal.stop()
            contract = self.P.parse((self.root / 'completion-contract.json').read_bytes())
            plan = self.P.parse((self.root / 'plan.json').read_bytes())
            guard = current_identity() if contract['version'] == '1.0.12' else self.G
            self.assertEqual(self.G.digest(guard.previous_plan(contract, plan)),
                             self.G.PLANS['1.0.10'][0])
        def missing(name):
            if name == self.D.FRAGMENT:
                raise KeyError(name)
            return self.reader()(name)
        with self.assertRaisesRegex(ValueError, 'missing current input'):
            self.G.previous_plan(self.contract, self.plan, missing)

    def test_variable_generator_requires_matching_supplied_read_claim(self):
        changed = self.inputs[4] + b'\n# synthetic generator bytes, not executed\n'
        args = self.inputs[:4] + (changed, self.inputs[5])
        proposal = self.D.derive(*args)['artifacts']
        claims = {}
        read = self.reader({'tools/build_plan.py': changed}, claims=claims)
        old = self.G.previous_plan(proposal['completion-contract.json'], proposal['expected-plan.json'], read)
        self.assertEqual(self.G.digest(old), self.G.PLANS['1.0.10'][0])
        self.assertEqual(claims['tools/build_plan.py'], 'SYNTHETIC_BYTE_CLAIM')
        with self.assertRaisesRegex(ValueError, 'source fragment'):
            self.G.previous_plan(proposal['completion-contract.json'], proposal['expected-plan.json'], self.reader())

    def test_unknown_versions_and_types_never_open_context(self):
        with mock.patch.object(self.G, 'Session', side_effect=AssertionError('bad-version I/O')):
            for value in ['1.0.8', '1.0.12', None, True]:
                contract = copy.deepcopy(self.contract)
                contract['version'] = value
                for call in self.calls(contract=contract):
                    with self.assertRaises(ValueError):
                        call()
            for call in [lambda: self.G.legacy_contract(None), lambda: self.G.storage_policy([]),
                         lambda: self.G.previous_plan(self.contract, None),
                         lambda: self.G.legacy_plan(self.contract, None)]:
                with self.assertRaises(ValueError):
                    call()

    def test_seal_policy_inventory_pointer_and_child_coherent_mutations(self):
        for mutation in ['policy', 'duplicate', 'family', 'pointer', 'child', 'float', 'bool']:
            def change(c, p):
                if mutation == 'policy':
                    c['source_next_splits']['unknown'] = 'not ignored'
                elif mutation == 'duplicate':
                    c['unit_ids'].append('M06a2')
                elif mutation == 'family':
                    c['families']['model'].append('B25d1')
                elif mutation == 'pointer':
                    p['units'][0]['completion_contract']['id'] = 'SYSON-RUST-COMPLETION/1.0.10'
                else:
                    u = next(u for u in p['units'] if u['id'] == 'B25d1')
                    if mutation == 'child':
                        u['tasks'] = []
                    else:
                        u['budget']['production_lines_max'] = 400.0 if mutation == 'float' else True
                    c['source_next_splits']['children']['B25d1']['normalized_sha256'] = self.G.digest(
                        {k: v for k, v in u.items() if k not in self.P.OMIT})
            with self.subTest(mutation=mutation):
                if mutation in ['float', 'bool']:
                    c, p = copy.deepcopy(self.contract), copy.deepcopy(self.plan)
                    change(c, p)
                    seal = self.G.digest(c['source_next_splits'])
                    with mock.patch.object(self.G, 'POLICY_SHA256', seal):
                        with self.assertRaisesRegex(ValueError, 'strict ordinary child caps'):
                            self.G.validate_source_next_splits(c, p, self.reader())
                else:
                    self.deny(change)

    def test_all_old_records_and_current_root_metadata_are_authoritative(self):
        for uid, key, value in [('M06a1', 'tasks', []), ('B02', 'unknown_future', {'must': 'persist'}),
                               ('C99', 'source_fragment_sha256', '0' * 64)]:
            self.deny(lambda c, p: next(u for u in p['units'] if u['id'] == uid).__setitem__(key, value))
        for key, value in [('state', 'ACCEPTED'), ('lane_counts', {}), ('source_author_dispatch_waves', []),
                           ('unordered_write_conflicts', []), ('team_execution', {})]:
            if key == 'unordered_write_conflicts':
                value = [{'a': 'B25d1', 'b': 'M06a2'}]
            with self.subTest(key=key):
                self.deny(lambda c, p: p.__setitem__(key, value))
        for node in ['accepted:C04', 'accepted:C05', 'accepted:C06', 'accepted:C07', 'accepted:C08', 'accepted:C09']:
            self.deny(lambda c, p: p['readiness_graph'][node].remove('implementation:M06a2'))
        self.deny(lambda c, p: next(u for u in p['units'] if u['id'] == 'C04')['expanded_implementation_inputs'].remove('M06a2'))

    def test_current_owner_history_holds_and_actual_sources_cannot_drift(self):
        for name in list(self.P.PRESERVED) + ['execution.json', 'tools/build_plan.py']:
            raw = self.reader()(name) + b'\n'
            with self.subTest(name=name), self.assertRaises(ValueError):
                self.G.validate_source_next_splits(self.contract, self.plan, self.reader({name: raw}))
        for name, value in [(self.D.OWNER, b'x' * 65537), ('tools/build_plan.py', b'x' * 262145)]:
            with self.subTest(name=name), self.assertRaisesRegex(ValueError, 'bounded byte input'):
                self.G.previous_plan(self.contract, self.plan, self.reader({name: value}))
        state = self.P.parse((self.root / 'execution.json').read_bytes())
        state['current_integration']['unknown_attribution'] = 'must remain'
        with self.assertRaisesRegex(ValueError, 'whole current execution'):
            self.G.previous_contract(self.contract, self.reader({'execution.json': self.P.encode(state)}))
        def missing(name):
            if name == self.D.OWNER:
                raise KeyError(name)
            return self.reader()(name)
        with self.assertRaisesRegex(ValueError, 'missing current input'):
            self.G.legacy_plan(self.contract, self.plan, missing)

    def test_namespace_collision_and_stale_actual_source_identity_refuse(self):
        with mock.patch.object(Path, 'open', side_effect=AssertionError('collision read')):
            with self.assertRaisesRegex(ValueError, 'namespace collision'):
                modules()
        package = self.G.__package__
        original_open = Path.open
        guard_path = Path(self.G.__file__)
        changed = guard_path.read_bytes() + b'\n# unreviewed source body\n'
        def source_open(path, *args, **kwargs):
            return io.BytesIO(changed) if path == guard_path else original_open(path, *args, **kwargs)
        with mock.patch.dict(sys.modules):
            for name in [package, package + '.derive', package + '.guard']:
                del sys.modules[name]
            with mock.patch.object(Path, 'open', source_open):
                with self.assertRaisesRegex(ValueError, 'whole source identity'):
                    modules()
        prep = 'tools/preparation_splits.py'
        with self.assertRaisesRegex(ValueError, 'actual old provider source'):
            self.G.previous_contract(self.contract, self.reader({prep: self.reader()(prep) + b'\n'}))
        old_guard = self.root / 'composition/storage_split_guard.py'
        original_read = Path.read_bytes
        def stale_source(path):
            raw = original_read(path)
            return raw + b'\n# changed actual body\n' if path == old_guard else raw
        with mock.patch.object(Path, 'read_bytes', stale_source):
            with self.assertRaisesRegex(ValueError, 'fixed guard body changed'):
                self.G.previous_contract(self.contract, self.reader())
        actual = original_read(old_guard)
        different_seal = re.sub(rb"(?m)^POLICY_SHA256 = '[^']+'$",
                                b"POLICY_SHA256 = '" + b'0' * 64 + b"'", actual)
        def mismatched_binding(path):
            return different_seal if path == old_guard else original_read(path)
        with mock.patch.object(Path, 'read_bytes', mismatched_binding):
            with self.assertRaisesRegex(ValueError, 'actual provider differs from supplied bindings'):
                self.G.previous_contract(self.contract, self.reader())

    def test_model_preflight_rejects_cycles_subclasses_surrogates_before_serialization(self):
        cycle = []
        cycle.append(cycle)
        class Hostile(list):
            def __iter__(self):
                raise AssertionError('hostile iterator')
        values = [cycle, Hostile(), float('nan'), float('inf'), '\ud800', 1 << 4097]
        deep = []
        for _ in range(130):
            deep = [deep]
        values.append(deep)
        with mock.patch.object(self.G.json, 'dumps', side_effect=AssertionError('preflight serialization')):
            with mock.patch.object(self.G.copy, 'deepcopy', side_effect=AssertionError('preflight copy')):
                for value in values:
                    with self.subTest(kind=type(value).__name__), self.assertRaises(ValueError):
                        self.G.bound(value)
        alias = ['x' * 64]
        self.assertEqual(self.G.bound(alias, alias), 2 * self.G.bound(alias))
        large = 'x' * ((self.D.LIMIT - 130) // 6)
        self.assertEqual(self.G.bound(large), self.D.LIMIT)
        with self.assertRaises(ValueError):
            self.G.bound(large + 'x')

    def test_single_reader_exact_boundary_and_one_extra_byte(self):
        P, G = self.P, self.G
        read = self.reader()
        program_charge = sum(len(read(n)) + len(P.encode({n: '0' * 64})) for n in G.OLD_SOURCES)
        name = 'budget/data'
        available = self.D.LIMIT - program_charge - len(P.encode({name: '0' * 64})) - 8
        for size in [8, 9]:
            counts = Counter()
            callback = self.reader({name: b'x' * size}, counts=counts)
            session = G.Session(available, callback)
            if size == 8:
                self.assertEqual(session.read(name), b'x' * 8)
                self.assertEqual(session.used, self.D.LIMIT)
                self.assertEqual(session.read(name), b'x' * 8)
                self.assertEqual(counts[name], 1)
            else:
                with self.assertRaisesRegex(ValueError, 'bounded byte input'):
                    session.read(name)

    def test_policy_only_resealed_malformed_child_descriptors_refuse(self):
        mutations = [lambda c: c.update(unknown='field'), lambda c: c.pop('team'),
                     lambda c: c.update(family='model'), lambda c: c.update(team='other'),
                     lambda c: c.update(dependencies=['B01']), lambda c: c.update(dependencies='B01'),
                     lambda c: c.update(normalized_sha256='g' * 64),
                     lambda c: c.update(normalized_sha256='a' * 63),
                     lambda c: c.update(normalized_sha256=True)]
        for change in mutations:
            contract = copy.deepcopy(self.contract)
            change(contract['source_next_splits']['children']['B25d1'])
            seal = self.G.digest(contract['source_next_splits'])
            with mock.patch.object(self.G, 'POLICY_SHA256', seal), \
                 mock.patch.object(self.G, 'Session', side_effect=AssertionError('policy-only I/O')):
                with self.assertRaisesRegex(ValueError, 'fixed child descriptor'):
                    self.G.source_next_policy(contract)

    def test_cheap_container_refusal_precedes_any_complete_key_scan(self):
        value = {str(i): None for i in range(64)}
        actual_len = len
        def declared_len(item):
            return declared_size if item is value else actual_len(item)
        for declared_size in [500001, self.D.LIMIT]:
            with mock.patch.object(self.G, 'len', side_effect=declared_len, create=True), \
                 mock.patch.object(self.G, 'all', side_effect=AssertionError('premature key scan'), create=True):
                with self.assertRaisesRegex(ValueError, 'container framing/node bound'):
                    self.G.bound(value)


if __name__ == '__main__':
    unittest.main()
