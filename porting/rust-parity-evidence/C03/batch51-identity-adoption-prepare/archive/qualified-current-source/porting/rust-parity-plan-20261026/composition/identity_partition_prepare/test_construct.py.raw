"""Physical staged fixtures are not real generator or adoption evidence."""
import copy
import gc
import gzip
import hashlib
import importlib.util
import json
import os
import shutil
import sys
import tempfile
import types
import unittest
import weakref
import zlib
from contextlib import ExitStack, contextmanager
from pathlib import Path
from unittest import mock
from test_derive import D, G, C, H, Fixture

PATH = Path(C.__file__)


class PhysicalFixture(Fixture):
    @classmethod
    def setUpClass(cls):
        super().setUpClass()
        cls.actual = PATH.resolve().parents[2]
        root = Path(os.environ['RUNNER_TEMP'] if os.environ.get('GITHUB_ACTIONS') == 'true'
                    else os.environ['SYSON_BOOTSTRAP_TEST_ROOT'])
        if root.resolve() != root or not root.is_dir():
            raise ValueError('declared canonical test root required')
        expected = Path(os.environ['RUNNER_TEMP']) if os.environ.get('GITHUB_ACTIONS') == 'true' else Path('/workspace/build/builderbob')
        if not root.is_relative_to(expected):
            raise ValueError('test root outside admitted workspace or hosted runner')
        temporary = tempfile.TemporaryDirectory(prefix='identity-construct-', dir=root)
        cls.addClassCleanup(temporary.cleanup)
        cls.output = Path(temporary.name)
        cls.stage = cls.output / 'physical-fixture'
        cls.stage.mkdir(mode=0o700)
        cls.previous_snapshot = json.loads(cls.files['remaining-work/snapshot.lock.json'])
        history = set(cls.previous_snapshot['progress_inputs']) - {'team-dispatch.json'}
        cls.stage_origins = {}
        for name in set(cls.previous_snapshot['sha256']):
            target = cls.stage / name
            target.parent.mkdir(parents=True, exist_ok=True)
            historical = name in history or name in cls.files and name not in C.CAPTURED
            raw = H.read(cls.actual, name, 8 * C.MIB)
            if historical and D.sha(raw) != cls.previous_snapshot['sha256'][name]:
                raise ValueError('historical physical fixture source drift')
            cls.stage_origins[name] = {
                'kind': 'VERIFIED_HISTORICAL_BYTES' if historical else 'CURRENT_ROOT_UNIT_FIXTURE_BYTES',
                'sha256': D.sha(raw)}
            target.write_bytes(raw)
        cls.catalogue = {'schema': 1, 'files': {}, 'proof_inputs': []}
        for index, name in enumerate(sorted(set(cls.files) | {'remaining-work/units.json', 'team-dispatch.json'})):
            raw = cls.files[name] if name in cls.files else cls.tails[name]
            source = name
            stored = raw
            encoding = 'raw'
            if name in C.CAPTURED:
                source = C.CATALOGUE.rsplit('/', 1)[0] + '/capture-' + str(index) + '.gz'
                stored, encoding = gzip.compress(raw, mtime=0), 'gzip'
            target = cls.stage / source
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(stored)
            cls.catalogue['files'][name] = {'source': source, 'encoding': encoding,
                'bytes': len(raw), 'sha256': D.sha(raw), 'stored_bytes': len(stored),
                'stored_sha256': D.sha(stored), 'git_blob_sha1': hashlib.sha1(
                    b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()}
        for name in C.ADDITIONS:
            target = cls.stage / name
            target.parent.mkdir(parents=True, exist_ok=True)
            if name == D.FRAGMENT:
                raw = cls.args[0]
            elif name == D.AMENDMENT:
                raw = cls.args[1]
            elif name == 'tools/identity_partition_splits.py':
                raw = b'UNEXECUTED_TEST_ONLY_LOADER_ROLE\n'
            else:
                raw = H.read(cls.actual, name, 8 * C.MIB)
            target.write_bytes(raw)
        cls.put('completion-contract.json', json.dumps(cls.contract, ensure_ascii=False, separators=(',', ':')).encode())
        cls.put('acceptance-packet.schema.json', json.dumps(cls.artifacts['acceptance-packet.schema.json']).encode())
        cls.put('tools/build_plan.py', cls.args[2])
        cls.put('plan.json', json.dumps(cls.plan, ensure_ascii=False, indent=2).encode())
        cls.put('team-dispatch.json', H.read(cls.actual, 'team-dispatch.json', 2 * C.MIB) + b'\n')
        for uid in D.IDS:
            cls.put('prs/' + uid + '.md', ('UNSIGNED UNIT FIXTURE ' + uid + '\n').encode())
        for name in C.FIXTURES:
            cls.put(name, H.read(cls.actual, name, 8 * C.MIB))
        cls.catalogue_raw = json.dumps(cls.catalogue, indent=2).encode()
        cls.put(C.CATALOGUE, cls.catalogue_raw)
        cls.legacy = C.cold()

    @classmethod
    def put(cls, name, raw):
        path = cls.stage / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(raw)
        cls.stage_origins[name] = {'kind': 'EXPLICIT_CURRENT_UNIT_FIXTURE_BYTES', 'sha256': D.sha(raw)}

    def setUp(self):
        stack = ExitStack()
        self.addCleanup(stack.close)
        for owner, name, value in [(C, 'ROOT', self.stage), (self.legacy, 'ROOT', self.stage),
                (C, 'OUTPUT_ROOT', self.output), (C, 'CATALOGUE_SHA256', D.sha(self.catalogue_raw)),
                (G, 'POLICY_SHA256', D.digest(self.artifacts['policy.json']))]:
            stack.enter_context(mock.patch.object(owner, name, value))

    @contextmanager
    def changed(self, name, raw):
        path = self.stage / name
        before = path.read_bytes()
        path.write_bytes(raw)
        try:
            yield
        finally:
            path.write_bytes(before)

    @contextmanager
    def altered_catalogue(self, change, reseal=True):
        value = copy.deepcopy(self.catalogue)
        change(value)
        raw = json.dumps(value).encode()
        with self.changed(C.CATALOGUE, raw), mock.patch.object(
                C, 'CATALOGUE_SHA256', D.sha(raw) if reseal else D.sha(self.catalogue_raw)):
            yield value

    def install_lock(self):
        proposal = C.contract_lock()
        self.put('completion-contract.lock.json', proposal.encoded['completion-contract.lock.json'])
        return proposal

class ConstructorTests(PhysicalFixture, unittest.TestCase):
    def test_pending_hooks_refuse_before_any_physical_read(self):
        for owner, name in [(C, 'CATALOGUE_SHA256'), (G, 'POLICY_SHA256')]:
            with mock.patch.object(owner, name, 'PENDING_ROOT_FREEZE'), mock.patch.object(
                    C, 'physical', side_effect=AssertionError('unadmitted physical read')):
                with self.assertRaisesRegex(ValueError, 'pending'):
                    C.contract_lock()

    def test_default_physical_normative_producer_needs_no_current_plan(self):
        path = self.stage / 'plan.json'
        raw = path.read_bytes()
        path.unlink()
        try:
            views = []
            original = C.prior
            def observed():
                result = original()
                views.append((result[1], result[2]))
                return result
            with mock.patch.object(G, 'checked', side_effect=AssertionError('actual plan invented')), \
                    mock.patch.object(C, 'prior', side_effect=observed):
                proposal = C.contract_lock()
        finally:
            path.write_bytes(raw)
        lock = proposal['artifacts']['completion-contract.lock.json']
        self.assertEqual((lock['contract_id'], len(lock['sha256'])), (D.VERSION, 72))
        self.assertEqual(proposal.reader.cache, {})
        self.assertEqual(views, [({}, {})])
        self.assertEqual(set(lock['sha256']), set(json.loads(self.files['completion-contract.lock.json'])['sha256']) | C.ADDITIONS)
        destination = self.output / 'normative-output'
        C.emit_fresh(destination, proposal)
        self.assertEqual((destination / 'completion-contract.lock.json').read_bytes(),
                         proposal.encoded['completion-contract.lock.json'])
        self.assertTrue(all('EXPLICIT_SUPPLIED' not in name for name in proposal['manifest']['input_sha256']))

    def test_whole799_snapshot_real_default_reads_and_independent_generic_verifier(self):
        self.install_lock()
        proposal = C.snapshot('SYSON-RUST-REMAINING/constructor-fixture', '0' * 40)
        register, lock = (proposal['artifacts'][n] for n in ['units.json', 'snapshot.lock.json'])
        self.assertEqual((register['unit_count'], len(register['units'])), (799, 799))
        self.assertEqual((len(proposal.inputs), len(lock['sha256'])), (1236, 1237))
        self.assertNotEqual(lock['sha256']['team-dispatch.json'], self.previous_snapshot['sha256']['team-dispatch.json'])
        self.assertEqual(lock['historical_progress']['bindings_sha256'], D.PROGRESS)
        path = self.actual / 'remaining-work/verify_snapshot.py'
        self.assertEqual(D.sha(path.read_bytes()), self.previous_snapshot['sha256']['remaining-work/verify_snapshot.py'])
        spec = importlib.util.spec_from_file_location('b50_actual_snapshot_validator', path)
        verifier = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(verifier)
        with self.changed('remaining-work/units.json', proposal.encoded['units.json']):
            target = self.stage / 'remaining-work/snapshot.lock.json'
            target.parent.mkdir(parents=True, exist_ok=True)
            before = target.read_bytes() if target.exists() else None
            target.write_bytes(proposal.encoded['snapshot.lock.json'])
            try:
                result = verifier.validate(self.stage)
            finally:
                target.unlink() if before is None else target.write_bytes(before)
        self.assertEqual(result['unit_count'], 799)
        self.assertEqual(result['feature_acceptance_counts']['NOT_ACCEPTED'], 55)
        self.assertEqual(result['feature_acceptance_counts']['OPEN_NO_ACCEPTANCE_RECORD'], 744)
        C.emit_fresh(self.output / 'snapshot-output', proposal)

    def test_catalogue_whole_pin_and_role_admission_refuse(self):
        with self.altered_catalogue(lambda c: c.update(schema=2), reseal=False):
            with self.assertRaisesRegex(ValueError, 'catalogue changed'):
                C.prior()
        changes = [lambda c: c['files'].pop('plan.json'), lambda c: c.update(schema=True),
                   lambda c: c['files']['plan.json'].update(source='../escape.gz'),
                   lambda c: c['files']['plan.json'].update(bytes=8 * C.MIB + 1),
                   lambda c: c['files']['plan.json'].update(encoding='raw'),
                   lambda c: c['files']['remaining-work/units.json'].update(bytes=6 * C.MIB + 1),
                   lambda c: c.update(proof_inputs=['remaining-work/checkpoint-identity-partition-1.0.12/proof/same.json'] * 2)]
        for change in changes:
            with self.altered_catalogue(change), self.assertRaises(ValueError):
                C.prior()

    def test_aggregate_prior_claims_refuse_before_capture_read(self):
        def enlarge(catalogue):
            names = [n for n in catalogue['files'] if n not in
                     {'remaining-work/units.json', 'team-dispatch.json'}]
            used = sum(catalogue['files'][n]['bytes'] + 128 + 6 * len(n) for n in names)
            catalogue['files']['completion-contract.json']['bytes'] += 16 * C.MIB - used
            catalogue['files']['remaining-work/units.json']['bytes'] = 6 * C.MIB
            catalogue['files']['team-dispatch.json']['bytes'] = 2 * C.MIB
        calls = []
        original = C.physical
        def observed(name, maximum, seen):
            calls.append(name)
            return original(name, maximum, seen)
        with self.altered_catalogue(enlarge), mock.patch.object(C, 'physical', side_effect=observed):
            with self.assertRaisesRegex(ValueError, 'prior phase bound'):
                C.prior()
        self.assertEqual(calls, [C.CATALOGUE])

    def test_resealed_trailing_and_truncated_gzip_members_refuse(self):
        row = self.catalogue['files']['plan.json']
        original = (self.stage / row['source']).read_bytes()
        for data in [original + b'trailing', original[:-1]]:
            def change(catalogue):
                catalogue['files']['plan.json'].update(stored_bytes=len(data), stored_sha256=D.sha(data))
            with self.altered_catalogue(change), self.changed(row['source'], data):
                with self.assertRaises((ValueError, zlib.error)):
                    C.prior()

    def test_coherent_captured_program_rewrite_is_data_and_cannot_execute(self):
        name = 'tools/build_plan.py'
        raw = b"raise AssertionError('captured source executed')\n"
        stored = gzip.compress(raw, mtime=0)
        def change(catalogue):
            catalogue['files'][name].update(bytes=len(raw), sha256=D.sha(raw), stored_bytes=len(stored),
                stored_sha256=D.sha(stored), git_blob_sha1=hashlib.sha1(
                    b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest())
        with self.altered_catalogue(change), self.changed(self.catalogue['files'][name]['source'], stored):
            with self.assertRaises(ValueError):
                C.contract_lock()

    def test_current_contract_and_current_plan_tampering_refuse(self):
        for name, value in [('completion-contract.json', copy.deepcopy(self.contract)),
                            ('plan.json', copy.deepcopy(self.plan))]:
            if name == 'completion-contract.json':
                value['families']['model'].remove('M06a2b')
            else:
                next(u for u in value['units'] if u['id'] == 'M06a2b')['tasks'].pop()
            with self.changed(name, json.dumps(value).encode()), self.assertRaises(ValueError):
                C.early(name == 'plan.json')

    def test_real_bulk_progress_drift_is_not_hidden_by_old_pins(self):
        self.install_lock()
        normative = set(json.loads(self.files['completion-contract.lock.json'])['sha256'])
        name = next(n for n in self.previous_snapshot['progress_inputs'] if n != 'team-dispatch.json'
                    and n not in normative and (self.stage / n).stat().st_size < 4096)
        with self.changed(name, b'changed historical evidence'), self.assertRaisesRegex(
                ValueError, 'historical progress bindings'):
            C.snapshot('SYSON-RUST-REMAINING/progress-refusal', '0' * 40)

    def test_stream_exact_limit_over_limit_and_token_refusal(self):
        value = {'value': 'literal bytes'}
        raw = json.dumps(value, ensure_ascii=False, separators=(',', ':')).encode() + b'\n'
        self.assertEqual(C.compact(value, len(raw)), raw)
        with self.assertRaisesRegex(ValueError, 'bounded output stream'):
            C.compact(value, len(raw) - 1)
        with self.assertRaisesRegex(ValueError, 'bounded output token'):
            C.compact({'value': 'x' * 16385}, C.MIB)

    def test_live_proposal_mutations_reload_and_changed_seals_refuse(self):
        proposal = C.contract_lock()
        output = self.output / 'refused-live'
        with self.assertRaisesRegex(ValueError, 'live proposal'):
            C.emit_fresh(output, dict(proposal))
        proposal.encoded['completion-contract.lock.json'][0] ^= 1
        with self.assertRaisesRegex(ValueError, 'live output bytes'):
            C.emit_fresh(output, proposal)
        self.assertFalse(output.exists())
        proposal = C.contract_lock()
        with mock.patch.object(C, 'CATALOGUE_SHA256', '0' * 64), self.assertRaises(ValueError):
            C.emit_fresh(output, proposal)

    def test_physical_source_drift_and_existing_output_preserve_owned_data(self):
        proposal = C.contract_lock()
        output = self.output / 'source-drift-refusal'
        with self.changed('COMPLETION-CONTRACT.md', b'changed after construction'):
            with self.assertRaisesRegex(ValueError, 'physical source changed'):
                C.emit_fresh(output, proposal)
        self.assertFalse(output.exists())
        output.mkdir()
        marker = output / 'retained'
        marker.write_bytes(b'owned previous bytes')
        with self.assertRaisesRegex(ValueError, 'fresh owned'):
            C.emit_fresh(output, proposal)
        self.assertEqual(marker.read_bytes(), b'owned previous bytes')

    def test_cleared_or_replaced_public_inputs_cannot_hide_real_source_drift(self):
        for replacement in [False, True]:
            proposal = C.contract_lock()
            proposal.inputs.clear()
            if replacement:
                proposal.inputs['invented.json'] = '0' * 64
            output = self.output / ('private-inputs-' + str(replacement))
            with self.changed('COMPLETION-CONTRACT.md', b'changed source with forged diagnostics'):
                with self.assertRaisesRegex(ValueError, 'immutable emission metadata'):
                    C.emit_fresh(output, proposal)
            self.assertFalse(output.exists())

    def test_used_reserve_and_binding_diagnostics_cannot_change_authority(self):
        for field in ['used', 'reserve', 'bindings']:
            proposal = C.contract_lock()
            if field == 'bindings':
                proposal.bindings = ('changed',) + proposal.bindings[1:]
            else:
                setattr(proposal.reader, field, 0)
            output = self.output / ('private-metadata-' + field)
            with self.assertRaisesRegex(ValueError, 'immutable emission metadata'):
                C.emit_fresh(output, proposal)
            self.assertFalse(output.exists())

    def test_pending_emission_refuses_before_serialization_reads_or_output(self):
        proposal = C.contract_lock()
        for owner, name in [(C, 'CATALOGUE_SHA256'), (G, 'POLICY_SHA256')]:
            with mock.patch.object(owner, name, 'PENDING_ROOT_FREEZE'), mock.patch.object(
                    C, 'fingerprint', side_effect=AssertionError('serialization before pending check')), \
                    mock.patch.object(C, 'physical', side_effect=AssertionError('read before pending check')), \
                    mock.patch.object(Path, 'mkdir', side_effect=AssertionError('output before pending check')):
                proposal.bindings = (C.CATALOGUE_SHA256, G.POLICY_SHA256, str(C.ROOT), str(C.OUTPUT_ROOT))
                with self.assertRaisesRegex(ValueError, 'emission binding pending'):
                    C.emit_fresh(self.output / 'pending-refusal', proposal)

    def test_root_and_output_policy_cannot_redirect_issued_provenance(self):
        proposal = C.contract_lock()
        for field in ['ROOT', 'OUTPUT_ROOT']:
            with mock.patch.object(C, field, self.output / ('different-' + field)), mock.patch.object(
                    C, 'physical', side_effect=AssertionError('redirected physical read')):
                with self.assertRaisesRegex(ValueError, 'issuance bindings'):
                    C.emit_fresh(self.output / 'root-binding-refusal', proposal)

    def test_public_contract_bound_precedes_baseline_digest_and_copy(self):
        Q, S, _, _ = D.provider()
        value = {'x': 'A' * 32}
        with mock.patch.object(Q, 'LIMIT', 340):
            self.assertEqual(S.bound(value), 340)
        with mock.patch.object(Q, 'LIMIT', 339), mock.patch.object(
                D, 'baseline', side_effect=AssertionError('baseline before current-object bound')), \
                mock.patch.object(D, 'digest', side_effect=AssertionError('digest before bound')), \
                mock.patch.object(C.copy, 'deepcopy', side_effect=AssertionError('copy before bound')):
            with self.assertRaisesRegex(ValueError, 'object string/framing bound'):
                C.contract_only(value, {}, *self.args)

    def test_public_contract_exact_types_cycles_and_nonfinite_refuse(self):
        class Hostile(dict):
            def items(self):
                raise AssertionError('hostile subclass iterated')
        cycle = {}
        cycle['self'] = cycle
        for value in [Hostile(), cycle, {'bad': object()}, {'bad': float('nan')}]:
            with mock.patch.object(D, 'baseline', side_effect=AssertionError('old baseline reached')), \
                    mock.patch.object(D, 'digest', side_effect=AssertionError('unbounded digest')):
                with self.assertRaises(ValueError):
                    C.contract_only(value, {}, *self.args)

    def test_mutated_public_body_refuses_before_fingerprint(self):
        proposal = C.contract_lock()
        proposal['artifacts']['bad'] = object()
        with mock.patch.object(C, 'fingerprint', side_effect=AssertionError('unbounded body serialized')):
            with self.assertRaisesRegex(ValueError, 'exact JSON builtin types'):
                C.emit_fresh(self.output / 'body-bound-refusal', proposal)

    def test_registry_is_bounded_and_does_not_retain_strong_proposals(self):
        proposals = [C.contract_lock() for _ in range(8)]
        with self.assertRaisesRegex(ValueError, 'bounded live proposal registry'):
            C.contract_lock()
        reference = weakref.ref(proposals[0])
        del proposals
        gc.collect()
        self.assertIsNone(reference())
        proposal = C.contract_lock()
        self.assertEqual(proposal['status'], 'UNSEALED_PROPOSAL_ONLY')
        forged = C.Proposal(dict(proposal))
        for field in ['reader', 'encoded', 'seal', 'inputs', 'bindings']:
            setattr(forged, field, getattr(proposal, field))
        with self.assertRaisesRegex(ValueError, 'registered live proposal'):
            C.emit_fresh(self.output / 'unregistered-class-refusal', forged)

    def test_hostile_encoded_container_refuses_before_iteration_hash_or_output(self):
        class Hostile(dict):
            def __iter__(self):
                raise AssertionError('unadmitted encoded iteration')
            def items(self):
                raise AssertionError('unchecked ../escape output items')
        proposal = C.contract_lock()
        proposal.encoded = Hostile(proposal.encoded)
        with mock.patch.object(D, 'provider', return_value=D.provider()), mock.patch.object(
                D, 'sha', side_effect=AssertionError('unadmitted encoded hash')), \
                mock.patch.object(C, 'fingerprint', side_effect=AssertionError('early serialization')), \
                mock.patch.object(C, 'physical', side_effect=AssertionError('early physical read')), \
                mock.patch.object(Path, 'mkdir', side_effect=AssertionError('early output')):
            with self.assertRaisesRegex(ValueError, 'fixed encoded container'):
                C.emit_fresh(self.output / 'hostile-encoded', proposal)

    def test_encoded_keys_and_exact_body_types_refuse_before_hash(self):
        proposal = C.contract_lock()
        original = proposal.encoded
        class Name(str):
            pass
        class Body(bytearray):
            pass
        wrong = dict(original)
        wrong['../escape'] = wrong.pop('manifest.json')
        subclass_key = {Name(name): raw for name, raw in original.items()}
        for value, message in [(dict(original, extra=b'x'), 'fixed encoded container'),
                (wrong, 'fixed encoded names'), (subclass_key, 'fixed encoded names'),
                ({name: Body(raw) for name, raw in original.items()}, 'encoded body ceiling')]:
            proposal.encoded = value
            with mock.patch.object(D, 'provider', return_value=D.provider()), mock.patch.object(
                    D, 'sha', side_effect=AssertionError('unchecked output hash')), \
                    mock.patch.object(C, 'fingerprint', side_effect=AssertionError('early serialization')), \
                    mock.patch.object(Path, 'mkdir', side_effect=AssertionError('unchecked output path')):
                with self.assertRaisesRegex(ValueError, message):
                    C.emit_fresh(self.output / 'encoded-key-refusal', proposal)

    def test_encoded_per_file_and_private_aggregate_bounds_precede_hash(self):
        # Only this ordering fixture uses tiny encoded outputs; it is not producer evidence.
        bounds = {'completion-contract.lock.json': 4, 'manifest.json': 4}
        with mock.patch.dict(C.BOUNDS, bounds), mock.patch.object(C, 'compact', return_value=bytearray(b'x')):
            proposal = C.contract_lock()
        for aggregate in [False, True]:
            proposal.encoded = {'completion-contract.lock.json': b'12345', 'manifest.json': b'1234'}
            with mock.patch.dict(C.BOUNDS, {name: 8 if aggregate else 4 for name in bounds}), \
                    mock.patch.object(D, 'provider', return_value=D.provider()), mock.patch.object(
                    D, 'sha', side_effect=AssertionError('oversized output hashed')), \
                    mock.patch.object(C, 'fingerprint', side_effect=AssertionError('early serialization')), \
                    mock.patch.object(C, 'physical', side_effect=AssertionError('oversized output read')), \
                    mock.patch.object(Path, 'mkdir', side_effect=AssertionError('oversized output written')):
                message = 'captured input output reservation' if aggregate else 'encoded body ceiling'
                with self.assertRaisesRegex(ValueError, message):
                    C.emit_fresh(self.output / 'encoded-bound-refusal', proposal)

    def test_output_uses_immutable_snapshot_after_public_encoded_replacement(self):
        class Hostile(dict):
            def items(self):
                return [('manifest.json', b'changed'), ('../escape', b'escaped')]
        proposal = C.contract_lock()
        expected = {name: bytes(raw) for name, raw in proposal.encoded.items()}
        physical = C.physical
        def replace_after_validation(name, maximum, seen):
            for raw in proposal.encoded.values():
                if type(raw) is bytearray:
                    raw[:] = b'changed public body'
            proposal.encoded = Hostile()
            return physical(name, maximum, seen)
        output = self.output / 'immutable-output-snapshot'
        with mock.patch.object(C, 'physical', side_effect=replace_after_validation):
            C.emit_fresh(output, proposal)
        self.assertEqual({path.name: path.read_bytes() for path in output.iterdir()}, expected)
        self.assertFalse((output.parent / 'escape').exists())
