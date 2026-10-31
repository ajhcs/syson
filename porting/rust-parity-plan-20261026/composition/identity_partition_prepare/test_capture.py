"""Capture controls use staged physical inputs, not actual adoption evidence."""
import copy
import hashlib
import json
import re
import sys
import types
import test_derive as bootstrap
import zlib
import unittest
from contextlib import contextmanager
from pathlib import Path
from unittest import mock
from test_derive import D, G, C, H, PINS, source
from test_construct import PhysicalFixture


class CaptureTests(PhysicalFixture, unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        super().setUpClass()
        cls.oldroot = cls.output / 'original66'
        cls.oldroot.mkdir()
        for name, raw in dict(cls.files, **cls.tails).items():
            path = cls.oldroot / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(raw)

    def setUp(self):
        super().setUp()
        patcher = mock.patch.multiple(H, ROOT=self.oldroot, OUTPUT_ROOT=self.output)
        patcher.start()
        self.addCleanup(patcher.stop)

    @contextmanager
    def old_changed(self, name, raw):
        path = self.oldroot / name
        before = path.read_bytes()
        path.write_bytes(raw)
        try:
            yield
        finally:
            path.write_bytes(before)

    def test_real_producer66_whole_aliases_members_and_unsigned_readback(self):
        output = self.output / 'capture66'
        manifest = H.produce(output, [])
        self.assertEqual(manifest['status'], 'UNSEALED_PROPOSAL_ONLY')
        catalogue = json.loads((output / C.CATALOGUE).read_bytes())
        self.assertEqual((len(catalogue['files']), catalogue['proof_inputs']), (66, []))
        self.assertEqual(sum(row['encoding'] == 'raw' for row in catalogue['files'].values()), 50)
        self.assertEqual({n for n, row in catalogue['files'].items() if row['encoding'] == 'gzip'}, C.CAPTURED)
        for name, row in catalogue['files'].items():
            raw = (self.oldroot / name).read_bytes()
            self.assertEqual((row['bytes'], row['sha256']), (len(raw), D.sha(raw)))
            self.assertEqual(row['git_blob_sha1'], hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest())
            if row['encoding'] == 'raw':
                self.assertEqual(row['source'], name)
                self.assertFalse((output / name).exists())
                stored = raw
            else:
                stored = (output / row['source']).read_bytes()
                decoder = zlib.decompressobj(16 + zlib.MAX_WBITS)
                self.assertEqual(decoder.decompress(stored, len(raw) + 1), raw)
                self.assertTrue(decoder.eof)
                self.assertEqual((decoder.unused_data, decoder.unconsumed_tail), (b'', b''))
            self.assertEqual((row['stored_bytes'], row['stored_sha256']), (len(stored), D.sha(stored)))
        for name, pin in manifest['output_sha256'].items():
            self.assertEqual(D.sha((output / name).read_bytes()), pin)
        self.assertEqual(json.loads((output / 'manifest.json').read_bytes()), manifest)

    def test_exact_original_reader_and_verified_future_historical_view(self):
        with mock.patch.object(C, 'ROOT', self.oldroot), mock.patch.object(self.legacy, 'ROOT', self.oldroot):
            self.assertEqual(H.historical(self.oldroot), dict(self.files, **self.tails))
        self.install_lock()
        self.assertEqual(H.historical(self.stage), dict(self.files, **self.tails))
        with mock.patch.object(C, 'CATALOGUE_SHA256', 'PENDING_ROOT_FREEZE'):
            with self.assertRaisesRegex(ValueError, 'catalogue or policy pending'):
                H.historical(self.stage)
        with mock.patch.object(H, 'originals', side_effect=AssertionError('permissive old fallback')):
            with self.changed('completion-contract.lock.json', b'{"contract_id":"SYSON-RUST-COMPLETION/1.0.13"}'):
                with self.assertRaisesRegex(ValueError, 'unknown historical reader version'):
                    H.historical(self.stage)

    def test_source_normalization_returns_actual_bytes_and_rejects_other_changes(self):
        for name, field in [('guard', 'POLICY_SHA256'), ('construct', 'CATALOGUE_SHA256')]:
            raw = Path(getattr(G if name == 'guard' else C, '__file__')).read_bytes()
            needle = (field + " = 'PENDING_ROOT_FREEZE'").encode()
            normalized = re.sub(rb"(?m)^" + field.encode() + rb" = '[^']*'$", needle, raw)
            path = self.output / (name + '-source.py')
            sealed = normalized.replace(needle, (field + " = '" + '1' * 64 + "'").encode())
            path.write_bytes(sealed)
            self.assertEqual(source(path, name), sealed)
            for changed in [sealed + b'\n# changed program\n', sealed + needle + b'\n',
                            normalized.replace(needle, (field + " = 'bad'").encode())]:
                path.write_bytes(changed)
                with self.assertRaises(ValueError):
                    source(path, name)
        path = self.output / 'foreign-source.py'
        path.write_bytes(b'pass\n')
        with self.assertRaisesRegex(ValueError, 'whole actual'):
            source(path, 'derive')
        self.assertEqual(PINS['capture'], D.sha(Path(H.__file__).read_bytes()))

    def test_bad_proof_shape_namespace_and_duplicates_refuse_before_old_reads(self):
        class Hostile(list):
            def __iter__(self):
                raise AssertionError('unadmitted proof container')
        for proofs in [Hostile(), [object()], ['../escape'], [H.PRIOR + 'wrong'],
                       [H.PRIOR.replace('prior/', 'proof/') + 'x'] * 2, ['x'] * 65]:
            with mock.patch.object(H, 'originals', side_effect=AssertionError('proof preflight bypassed')):
                with self.assertRaises(ValueError):
                    H.produce(self.output / 'bad-proof', proofs)

    def test_original_alias_and_tail_corruption_refuse_without_output(self):
        for name in ['tools/team_policy.py', 'remaining-work/units.json', 'team-dispatch.json']:
            with self.old_changed(name, b'changed original bytes'):
                with self.assertRaises(ValueError):
                    H.produce(self.output / 'bad-original', [])
        self.assertFalse((self.output / 'bad-original').exists())

    def test_original_aggregate_stat_bound_precedes_large_body_read(self):
        original_stat = Path.stat
        def claimed(path, *args, **kwargs):
            result = original_stat(path, *args, **kwargs)
            if path.parent == self.oldroot / 'lanes':
                values = list(result)
                values[6] = 8 * C.MIB
                return type(result)(values)
            return result
        original_read = H.read
        def observed(root, name, maximum):
            if name != 'completion-contract.lock.json':
                raise AssertionError('body read before phase admission')
            return original_read(root, name, maximum)
        with mock.patch.object(Path, 'stat', claimed), mock.patch.object(H, 'read', side_effect=observed):
            with self.assertRaisesRegex(ValueError, 'historical phase ceiling'):
                H.originals(self.oldroot)

    def test_member_exact_type_determinism_and_small_output_threshold(self):
        raw = bytes(range(256)) * 1024
        self.assertEqual(H.member(raw), H.member(raw))
        with self.assertRaisesRegex(ValueError, 'raw member input'):
            H.member(bytearray(raw))
        class Encoder:
            def compress(self, value):
                return b'123456789'
            def flush(self):
                raise AssertionError('overflow continued to flush')
        with mock.patch.object(H, 'MEMBER_MAX', 8), mock.patch.object(H.zlib, 'compressobj', return_value=Encoder()):
            with self.assertRaisesRegex(ValueError, 'gzip member output'):
                H.member(b'bounded fixture')

    def test_final_source_and_program_drift_refuse_before_directory_creation(self):
        original_member, original_read = H.member, H.read
        before = (self.oldroot / 'TEAM-EXECUTION.md').read_bytes()
        def drift(raw):
            encoded = original_member(raw)
            (self.oldroot / 'TEAM-EXECUTION.md').write_bytes(b'changed while compressing')
            return encoded
        try:
            with mock.patch.object(H, 'member', side_effect=drift), mock.patch.object(
                    Path, 'mkdir', side_effect=AssertionError('output before physical freshness')):
                with self.assertRaisesRegex(ValueError, 'physical source changed'):
                    H.produce(self.output / 'changed-source', [])
        finally:
            (self.oldroot / 'TEAM-EXECUTION.md').write_bytes(before)
        def program_drift(root, name, maximum):
            raw = original_read(root, name, maximum)
            return raw + b'changed' if root == Path(H.__file__).parent and name == 'capture.py' else raw
        with mock.patch.object(H, 'read', side_effect=program_drift), mock.patch.object(
                Path, 'mkdir', side_effect=AssertionError('output before actual program freshness')):
            with self.assertRaisesRegex(ValueError, 'actual program changed'):
                H.produce(self.output / 'changed-program', [])

    def test_output_policy_and_existing_directory_preserve_owned_bytes(self):
        output = self.output / 'owned-capture'
        output.mkdir()
        marker = output / 'retained'
        marker.write_bytes(b'prior owned evidence')
        with self.assertRaisesRegex(ValueError, 'fresh owned capture output'):
            H.produce(output, [])
        self.assertEqual(marker.read_bytes(), b'prior owned evidence')
        with self.assertRaisesRegex(ValueError, 'fresh owned capture output'):
            H.produce(self.output.parent / 'outside-output-policy', [])

    def test_symlink_original_and_proof_origins_refuse(self):
        target = self.oldroot / 'TEAM-EXECUTION.md'
        raw = target.read_bytes()
        target.unlink()
        target.symlink_to(self.oldroot / 'teams.json')
        try:
            with self.assertRaisesRegex(ValueError, 'canonical capture origin'):
                H.produce(self.output / 'symlink-original', [])
        finally:
            target.unlink()
            target.write_bytes(raw)
        proof = H.PRIOR.replace('prior/', 'proof/') + 'symlink.json'
        path = self.oldroot / proof
        path.parent.mkdir(parents=True, exist_ok=True)
        path.symlink_to(self.oldroot / 'teams.json')
        try:
            with self.assertRaisesRegex(ValueError, 'canonical proof origin'):
                H.produce(self.output / 'symlink-proof', [proof])
        finally:
            path.unlink()

    def test_complete_proof_claim_bound_precedes_proof_body_read(self):
        proof = H.PRIOR.replace('prior/', 'proof/') + 'budget.json'
        path = self.oldroot / proof
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(b'bounded proof fixture')
        original_stat = Path.stat
        def claimed(target, *args, **kwargs):
            result = original_stat(target, *args, **kwargs)
            if target == path:
                values = list(result)
                values[6] = 8 * C.MIB + 1
                return type(result)(values)
            return result
        with mock.patch.object(H, 'originals', return_value=dict(self.files, **self.tails)), \
                mock.patch.object(Path, 'stat', claimed), mock.patch.object(
                H, 'read', side_effect=AssertionError('proof body before complete admission')):
            with self.assertRaisesRegex(ValueError, 'complete capture proof budget'):
                H.produce(self.output / 'proof-bound-refusal', [proof])
        path.unlink()

    def test_fresh_output_readback_mismatch_is_not_reported_success(self):
        original = H.read
        output = self.output / 'readback-refusal'
        def corrupted(root, name, maximum):
            raw = original(root, name, maximum)
            return b'wrong physical readback' if root == output else raw
        with mock.patch.object(H, 'read', side_effect=corrupted):
            with self.assertRaisesRegex(ValueError, 'fresh capture readback changed'):
                H.produce(output, [])
        self.assertTrue(output.exists())
        self.assertFalse((output / 'manifest.json').exists())

    def test_proof_authority_is_immutable_after_admission(self):
        proofs = []
        original = H.originals
        def mutate_after_preflight(root):
            proofs.append('../unchecked-late-proof')
            return original(root)
        output = self.output / 'immutable-proof-authority'
        with mock.patch.object(H, 'originals', side_effect=mutate_after_preflight):
            manifest = H.produce(output, proofs)
        self.assertEqual(proofs, ['../unchecked-late-proof'])
        self.assertEqual(json.loads((output / C.CATALOGUE).read_bytes())['proof_inputs'], [])
        self.assertNotIn('../unchecked-late-proof', manifest['input_sha256'])

    def test_exact_three_new_tool_origins_and_complete_old_partition(self):
        added = {'tools/verify_plan.py', 'tools/team_policy.py', 'tools/test_team_policy.py'}
        previous = {'completion-contract.json', 'acceptance-packet.schema.json', 'COMPLETION-CONTRACT.md',
                    'tools/build_plan.py', 'tools/completion_contract.py', 'tools/source_next_splits.py',
                    'plan.json', 'team-dispatch.json', 'completion-contract.lock.json',
                    'remaining-work/snapshot.lock.json', 'remaining-work/units.json'}
        self.assertEqual(C.CAPTURED, previous | added | {
            'composition/source_next_prepare/test_guard.py', 'composition/source_next_prepare/test_construct.py'})
        output = self.output / 'fourteen-capture-partition'
        manifest = H.produce(output, [])
        catalogue = json.loads((output / C.CATALOGUE).read_bytes())
        self.assertEqual((manifest['raw_alias_count'], len(C.CAPTURED)), (50, 16))
        for name in added:
            row = catalogue['files'][name]
            self.assertEqual((row['encoding'], row['source']), ('gzip', H.PRIOR + 'captures/' + name + '.gz'))
            stored = (output / row['source']).read_bytes()
            self.assertEqual(zlib.decompress(stored, 16 + zlib.MAX_WBITS), self.files[name])
            self.assertFalse((output / name).exists())

    def test_three_future_live_tool_changes_do_not_replace_historical_bytes(self):
        self.install_lock()
        for name in ['tools/verify_plan.py', 'tools/team_policy.py', 'tools/test_team_policy.py']:
            raw = b"raise AssertionError('passive tool bytes must never execute')\n"
            with self.changed(name, raw):
                historical = H.historical(self.stage)
                self.assertEqual(historical[name], self.files[name])
                self.assertNotEqual((self.stage / name).read_bytes(), historical[name])

    def test_fixed_fourteen_partition_rejects_missing_or_extra_gzip_origin(self):
        for name in ['tools/verify_plan.py', 'tools/team_policy.py', 'tools/test_team_policy.py']:
            def demote(catalogue):
                raw = self.files[name]
                catalogue['files'][name].update(source=name, encoding='raw', stored_bytes=len(raw),
                                                stored_sha256=D.sha(raw))
            with self.altered_catalogue(demote), self.assertRaisesRegex(ValueError, 'sixteen capture origins'):
                C.prior()
        def promote(catalogue):
            catalogue['files']['tools/preparation_splits.py'].update(
                source=H.PRIOR + 'extra.gz', encoding='gzip', stored_bytes=8, stored_sha256='0' * 64)
        with self.altered_catalogue(promote), self.assertRaisesRegex(ValueError, 'sixteen capture origins'):
            C.prior()

    def test_growing_source_read_is_capped_before_hashing(self):
        path = self.output / 'source-grow.py'
        path.write_bytes(b'x' * 32)
        stat = Path.stat
        def admitted(target, *args, **kwargs):
            result = stat(target, *args, **kwargs)
            if target == path:
                fields = list(result)
                fields[6] = 8
                return type(result)(fields)
            return result
        stream = mock.mock_open(read_data=b'x' * 32)
        with mock.patch.object(bootstrap, 'SOURCE_LIMIT', 8), mock.patch.object(Path, 'stat', admitted), \
                mock.patch.object(Path, 'open', stream), mock.patch.object(
                bootstrap.hashlib, 'sha256', side_effect=AssertionError('growth hashed before bound')):
            with self.assertRaisesRegex(ValueError, 'bounded actual source grew'):
                source(path, 'derive')
        stream().read.assert_called_once_with(9)

    def test_child_only_namespace_collision_refuses_before_source_io_or_exec(self):
        namespace = D.__package__
        foreign = types.ModuleType(namespace + '.foreign')
        existing = [name for name in sys.modules if name == namespace or name.startswith(namespace + '.')]
        with mock.patch.dict(sys.modules):
            for name in existing:
                sys.modules.pop(name)
            sys.modules[foreign.__name__] = foreign
            with mock.patch.object(bootstrap, 'source', side_effect=AssertionError('foreign prefix read')), \
                    mock.patch('builtins.exec', side_effect=AssertionError('foreign prefix executed')):
                with self.assertRaisesRegex(ValueError, 'test namespace collision'):
                    bootstrap.modules()
            self.assertIs(sys.modules[foreign.__name__], foreign)
            self.assertNotIn(namespace, sys.modules)
            self.assertNotIn(namespace + '.derive', sys.modules)

    def test_future_current_outputs_and_immutable_history_have_separate_origins(self):
        class Future(PhysicalFixture, unittest.TestCase):
            pass
        mutable = {'prs/M06a2.md', 'PR-INDEX.md', 'dispatch.json'}
        history = set(self.previous_snapshot['progress_inputs']) - {'team-dispatch.json'}
        immutable = next(name for name in sorted(history) if name not in self.files)
        original = H.read
        def regenerated(root, name, maximum):
            if root == self.actual and name in mutable:
                return original(root, name, maximum) + b'\n'
            return original(root, name, maximum)
        try:
            with mock.patch.object(H, 'read', side_effect=regenerated):
                Future.setUpClass()
            for name in mutable:
                raw = (Future.stage / name).read_bytes()
                self.assertEqual(raw, original(self.actual, name, 8 * C.MIB) + b'\n')
                self.assertEqual(Future.stage_origins[name], {
                    'kind': 'CURRENT_ROOT_UNIT_FIXTURE_BYTES', 'sha256': D.sha(raw)})
            self.assertEqual(Future.stage_origins[immutable]['kind'], 'VERIFIED_HISTORICAL_BYTES')
            self.assertEqual(Future.stage_origins['team-dispatch.json']['kind'], 'EXPLICIT_CURRENT_UNIT_FIXTURE_BYTES')
            self.assertEqual(D.sha(Future.tails['team-dispatch.json']),
                             self.previous_snapshot['sha256']['team-dispatch.json'])
        finally:
            Future.doClassCleanups()
        class CorruptHistory(PhysicalFixture, unittest.TestCase):
            pass
        def corrupted(root, name, maximum):
            return b'changed immutable historical body' if root == self.actual and name == immutable else original(root, name, maximum)
        try:
            with mock.patch.object(H, 'read', side_effect=corrupted):
                with self.assertRaisesRegex(ValueError, 'historical physical fixture source drift'):
                    CorruptHistory.setUpClass()
        finally:
            CorruptHistory.doClassCleanups()

    def assert_old_test_origin_survives_current_change(self, name, suffix):
        output = self.output / ('old-test-origin-' + suffix)
        manifest = H.produce(output, [])
        catalogue = json.loads((output / C.CATALOGUE).read_bytes())
        row = catalogue['files'][name]
        self.assertEqual((row['encoding'], row['source']),
                         ('gzip', H.PRIOR + 'captures/' + name + '.gz'))
        original = (self.oldroot / name).read_bytes()
        self.assertEqual(zlib.decompress((output / row['source']).read_bytes(), 31), original)
        self.assertEqual((manifest['input_sha256'][name], manifest['raw_alias_count']), (D.sha(original), 50))
        self.install_lock()
        with self.changed(name, b"raise AssertionError('passive historical test must not execute')\n"):
            self.assertEqual(H.historical(self.stage)[name], original)
            self.assertNotEqual((self.stage / name).read_bytes(), original)
        def demote(value):
            value['files'][name].update(source=name, encoding='raw', stored_bytes=len(original),
                                       stored_sha256=D.sha(original))
        with self.altered_catalogue(demote), self.assertRaisesRegex(ValueError, 'sixteen capture origins'):
            C.prior()

    def test_original_guard_test_is_captured_and_survives_future_current_change(self):
        self.assert_old_test_origin_survives_current_change(
            'composition/source_next_prepare/test_guard.py', 'guard')

    def test_original_constructor_test_is_captured_and_survives_future_current_change(self):
        self.assert_old_test_origin_survives_current_change(
            'composition/source_next_prepare/test_construct.py', 'construct')
