"""Finite-epoch controls; fake I/O never replaces immutable provider policies."""
from pathlib import Path
import copy
import hashlib
import importlib.util
import io
import json
import os
import tempfile
import types
import unittest
from unittest.mock import patch

SOURCE = Path(__file__).resolve().parent / 'acquire.py'
MANIFEST = SOURCE.parent / 'requests/models-01.json'
PIN = '682c4b8569beeb95defb30b315496b5f474d133d99170b4eb79827466c117f49'
SPEC = importlib.util.spec_from_file_location('fixed_core_acquire_tests', SOURCE)
fetch = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(fetch)


class Response(io.BytesIO):
    def __init__(self, body, url, code=200):
        super().__init__(body)
        self.url, self.code = url, code
        self.headers = {'Content-Length': str(len(body))}

    def geturl(self):
        return self.url


class AcquisitionTests(unittest.TestCase):
    def setUp(self):
        root = Path(os.environ['SYSON_BOOTSTRAP_TEST_ROOT'])
        self.assertTrue(root.is_absolute() and root.resolve() == root and root.is_dir())
        if os.environ.get('GITHUB_ACTIONS') == 'true':
            runner = Path(os.environ['RUNNER_TEMP'])
            self.assertTrue(runner.resolve() == runner and root.is_relative_to(runner))
        else:
            self.assertTrue(root.is_relative_to('/workspace/build/builderbob'))
        temporary = tempfile.TemporaryDirectory(prefix='finite-core-', dir=root)
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        block = patch('socket.create_connection', side_effect=AssertionError('network forbidden'))
        block.start()
        self.addCleanup(block.stop)

    def mutated(self, change, identifier='models-01'):
        manifest = json.loads((SOURCE.parent / fetch.epoch(identifier)['manifest']).read_bytes())
        change(manifest)
        raw = json.dumps(manifest).encode()
        registry = copy.deepcopy(fetch.EPOCHS)
        registry[identifier]['sha256'] = hashlib.sha256(raw).hexdigest()
        with patch.object(fetch, 'EPOCHS', registry), self.assertRaises(ValueError):
            fetch.requests(identifier, raw)

    def test_exact_registry_primary_order_roles_all_four_sidecars(self):
        self.assertEqual(hashlib.sha256(MANIFEST.read_bytes()).hexdigest(), PIN)
        self.assertEqual(set(fetch.EPOCHS), {'models-01', 'models-02'})
        rows = fetch.requests('models-01', MANIFEST.read_bytes())
        self.assertEqual(len(rows), 175)
        self.assertEqual(sum(r['role'] == 'pom' for r in rows), 34)
        self.assertEqual(sum(r['role'] == 'version_metadata' for r in rows), 1)
        self.assertEqual(sum(r['role'] == 'checksum' for r in rows), 105)
        self.assertEqual(sum(r['role'] == 'signature' for r in rows), 35)
        for index in range(35):
            group = rows[index * 5:index * 5 + 5]
            self.assertEqual([r['primary_index'] for r in group], [index] * 5)
            self.assertEqual([r['url'][len(group[0]['url']):] for r in group], fetch.base.SUFFIXES)

    def test_epoch_raw_hash_size_and_duplicate_keys_are_closed(self):
        for name in ['', 'models-03', '../models-01', None, True]:
            with self.subTest(epoch=name), self.assertRaises(ValueError):
                fetch.requests(name, MANIFEST.read_bytes())
        for raw in [MANIFEST.read_bytes() + b'\n', b'x' * (262144 + 1)]:
            with self.assertRaises(ValueError):
                fetch.requests('models-01', raw)
        self.mutated(lambda m: m.update(epoch='models-02'))
        self.mutated(lambda m: m.update(schema='other'))
        raw = MANIFEST.read_bytes().replace(b'"exact_primaries": 35',
                                            b'"exact_primaries": 35, "exact_primaries": 35')
        registry = copy.deepcopy(fetch.EPOCHS)
        registry['models-01']['sha256'] = hashlib.sha256(raw).hexdigest()
        with patch.object(fetch, 'EPOCHS', registry), self.assertRaisesRegex(ValueError, 'duplicate'):
            fetch.requests('models-01', raw)

    def test_independent_primary_seal_rejects_coherent_coordinates_and_order(self):
        def coherent(m):
            primary = m['new_primaries'][0]
            old = primary['coordinate']['version']
            primary['coordinate']['version'] = '99'
            for item in [primary] + m['requests'][:5]:
                for key in ['url', 'relative_artifact', 'relative_path']:
                    if key in item:
                        item[key] = item[key].replace('/' + old + '/', '/99/').replace('-' + old + '.pom', '-99.pom')
        self.mutated(coherent)
        self.mutated(lambda m: m['new_primaries'].reverse())
        self.mutated(lambda m: m['new_primaries'][0].update(kind='jar'))
        self.mutated(lambda m: m['new_primaries'].pop())

    def test_strict_types_limits_statuses_rows_and_destination_cannot_drift(self):
        changes = [lambda m: m['limits'].update(exact_primaries=True),
            lambda m: m['limits'].update(payload_and_sidecar_total_bytes=128 * fetch.MIB),
            lambda m: m['new_primaries'][0].update(primary_index=False),
            lambda m: m['new_primaries'][0]['coordinate'].update(groupId='wrong'),
            lambda m: m['requests'][0].update(primary_index=0.0),
            lambda m: m['requests'][1].update(algorithm='sha1'),
            lambda m: m['requests'][1].update(role='pom'),
            lambda m: m['requests'][0].update(relative_path='downloads/../escape'),
            lambda m: m['requests'].reverse(), lambda m: m['requests'].pop()]
        for change in changes:
            self.mutated(change)
        for statuses in [[200.0], [True], [200, 404]]:
            def coherent(m):
                m['new_primaries'][0]['allowed_status'] = statuses
                m['requests'][0]['allowed_status'] = statuses
            self.mutated(coherent)
        self.mutated(lambda m: m['requests'][1].update(allowed_status=[200.0, 404.0]))
        self.mutated(lambda m: m['requests'][1].update(maximum_bytes=65536.0))

    def test_provider_whole_source_binding_refuses_unknown_changed_and_symlink(self):
        self.assertEqual(hashlib.sha256(fetch.HELPER.read_bytes()).hexdigest(), fetch.HELPER_SHA)
        self.assertEqual(hashlib.sha256(fetch.TRANSPORT.read_bytes()).hexdigest(), fetch.TRANSPORT_SHA)
        self.assertEqual(fetch.base.PAYLOAD_LIMIT, 128 * fetch.MIB)
        self.assertEqual(fetch.helper.BODY_LIMIT, 16 * fetch.MIB)
        with self.assertRaisesRegex(ValueError, 'binding'):
            fetch.load_verified(fetch.HELPER, fetch.HELPER_SHA, 'arbitrary')
        original = Path.open
        def changed(path, *args, **kwargs):
            return io.BytesIO(b"raise AssertionError('unreviewed')") if path == fetch.HELPER else original(path, *args, **kwargs)
        with patch.object(Path, 'open', changed), self.assertRaisesRegex(ValueError, 'whole provider pin'):
            fetch.load_verified(fetch.HELPER, fetch.HELPER_SHA, 'core_metadata_primitives')
        link = self.root / 'linked'
        link.symlink_to(MANIFEST)
        with self.assertRaisesRegex(ValueError, 'canonical'):
            fetch.read_small(link, 262144)

    def run_main(self, mode=None, identifier='models-01'):
        manifest = SOURCE.parent / fetch.epoch(identifier)['manifest']
        pin = fetch.epoch(identifier)['sha256']
        out = self.root / 'attempt'
        out.mkdir(mode=0o700)
        lock = self.root / 'compiler.lock'
        lock.write_bytes(b'')
        paths = [str(p) for p in [SOURCE, manifest, fetch.HELPER, fetch.TRANSPORT, fetch.OWNER, lock]]
        spec = {'epoch': identifier, 'out': str(out), 'compiler_lock': str(lock),
            'ownership_helper': str(fetch.OWNER), 'proposal': str(manifest), 'proposal_sha256': pin,
            'inputs': paths, 'input_sha256': {p: fetch.base.digest(p) for p in paths}}
        if mode == 'lock':
            other = self.root / 'other.lock'
            other.write_bytes(b'')
            spec['compiler_lock'] = str(other)
        if mode == 'provider_missing':
            spec['input_sha256'].pop(str(fetch.HELPER))
        if mode == 'duplicate_input':
            spec['inputs'].append(str(lock))
        if mode == 'epoch':
            spec['epoch'] = 'unadmitted'
        if mode == 'proposal':
            spec['proposal'] = str(self.root / 'other.json')
        spec_path = self.root / 'spec.json'
        spec_path.write_text(json.dumps(spec))
        calls = []
        rows = fetch.requests(identifier, manifest.read_bytes())
        metadata_index = next((i for i, r in enumerate(rows) if r['role'] == 'version_metadata'), 0)
        body = b'opaque served XML/POM bytes: \xff\x00; no parser or normalization'
        def attest(unit, cookie, prefix, destination, environment):
            self.assertEqual(prefix, 'syson-b33-closure-')
            proof = {'unit': unit, 'cookie': cookie, 'control_group': '/synthetic-owned.scope'}
            destination.write_text(json.dumps(proof))
            return proof
        def loader(path, expected, name):
            self.assertEqual((path, expected, name), (fetch.OWNER, fetch.OWNER_SHA, 'core_scope_owner'))
            return types.SimpleNamespace(attest=attest)
        def fresh(path):
            self.assertEqual(path, out)
            self.assertTrue(path.resolve() == path and not path.is_symlink() and path.stat().st_mode & 0o077 == 0)
            self.assertEqual(list(path.iterdir()), [])
        def response(request, timeout):
            index = len(calls)
            row = rows[index]
            self.assertEqual(request.full_url, row['url'])
            self.assertEqual(timeout, 15)
            calls.append(request.full_url)
            content, status = body, 200
            if row['role'] == 'checksum':
                content = hashlib.new(row['algorithm'], body).hexdigest().encode()
                if mode in ['gnu', 'wrongname']:
                    filename = rows[index - index % 5]['url'].rsplit('/', 1)[-1]
                    content = content.upper() + b' ' + (b'*' if row['algorithm'] == 'sha256' else b' ') + b'/untrusted/' + filename.encode() + b'\n'
                    if mode == 'wrongname' and index == metadata_index + 1:
                        content = content.replace(filename.encode(), b'wrong.xml')
            if row['role'] == 'signature':
                content, status = b'unverified signature absent', 404
            if mode == 'badstrong' and index == metadata_index + 1:
                content = b'0' * 128
            if mode == 'sidecar404' and index == metadata_index + 1:
                content, status = b'optional checksum absent', 404
            if mode == 'missing' and index == metadata_index:
                content, status = b'missing mandatory metadata', 404
            if mode == 'metadata_body' and index == metadata_index:
                content = b'x' * (rows[metadata_index]['maximum_bytes'] + 1)
            if mode == 'body' and row['role'] == 'pom':
                content = b'x' * (2 * fetch.MIB)
            if mode == 'drift' and index == len(rows) - 1:
                lock.write_bytes(b'changed frozen bytes')
            result = Response(content, request.full_url, status)
            if mode in ['headers', 'header_overflow']:
                result.headers = {'X': '\x01' * (16379 if mode == 'headers' else 16380)}
            if mode == 'redirect':
                result.url += '?redirect'
            return result
        environment = {'HOME': os.environ['HOME'], 'SYSON_BOOTSTRAP_SOURCE_SHA256': fetch.base.digest(SOURCE)}
        home = types.SimpleNamespace(pw_name='builderbob', pw_dir=environment['HOME'])
        actual_controls = dict(fetch.base.CONTROLS)
        if mode == 'controls':
            actual_controls['pids.max'] = '33'
        error = None
        with lock.open('rb') as descriptor:
            argv = [str(SOURCE), str(spec_path), fetch.base.digest(spec_path), str(descriptor.fileno()),
                    'syson-b33-closure-' + 'a' * 32 + '.scope', 'b' * 32]
            with patch.object(fetch.sys, 'argv', argv), patch.dict(os.environ, environment), \
                 patch.object(fetch.pwd, 'getpwuid', return_value=home), \
                 patch.object(fetch, 'load_verified', side_effect=loader), \
                 patch.object(fetch, 'fresh_output', side_effect=fresh), \
                 patch.object(fetch, 'controls', return_value=actual_controls), \
                 patch.object(fetch, 'opener', return_value=types.SimpleNamespace(open=response)):
                clock = patch.object(fetch.time, 'monotonic', side_effect=[0, 901]) if mode == 'deadline' else None
                if clock:
                    clock.start()
                try:
                    fetch.main()
                except ValueError as failure:
                    error = str(failure)
                finally:
                    if clock:
                        clock.stop()
        receipt = out / 'receipt.json'
        return json.loads(receipt.read_text()) if receipt.exists() else None, calls, out, error, metadata_index

    def test_complete175_pipeline_preserves_raw_roles_all_algorithms_and_bounds(self):
        record, calls, out, error, _ = self.run_main()
        self.assertIsNone(error)
        self.assertEqual(record['status'], 'PASS_PASSIVE_ACQUISITION_ONLY')
        self.assertEqual((len(calls), len(record['requests']), len(record['checksums'])), (175, 175, 105))
        self.assertEqual(record['inputs_before'], record['inputs_after'])
        self.assertFalse(record['signature_verified'])
        self.assertFalse(record['closure_complete'])
        self.assertFalse(record['offline_compile_verified'])
        self.assertEqual(record['profile']['logical_total_bytes'], 80 * fetch.MIB)
        self.assertEqual(sum(r['role'] == 'version_metadata' for r in record['requests']), 1)
        for row in record['requests']:
            self.assertEqual(hashlib.sha256((out / row['relative_path']).read_bytes()).hexdigest(), row['sha256'])
            self.assertLessEqual(fetch.helper.encoded_size({k: v for k, v in row.items() if k != 'headers'}), 4096)
        self.assertIn(b'\xff\x00', (out / record['requests'][0]['relative_path']).read_bytes())
        self.assertLessEqual(fetch.helper.static_frame((out / 'scope-proof.json').stat().st_size, record), fetch.MIB)

    def test_complete_gnu_text_binary_uppercase_and_raw_sidecars(self):
        record, calls, out, error, _ = self.run_main('gnu')
        self.assertIsNone(error)
        self.assertEqual((len(calls), len(record['checksums'])), (175, 105))
        raw = (out / record['requests'][2]['relative_path']).read_bytes()
        self.assertIn(b' */untrusted/', raw)
        self.assertTrue(raw.endswith(b'\n'))

    def test_strong_metadata_checksum_and_wrong_filename_do_not_fallback(self):
        for mode in ['badstrong', 'wrongname']:
            with self.subTest(mode=mode):
                record, calls, out, error, index = self.run_main(mode)
                self.assertIsNotNone(error)
                self.assertEqual(len(calls), 175)
                self.assertIn('checksum', record['error'])
                self.assertEqual(record['requests'][index]['role'], 'version_metadata')
                self.assertTrue((out / record['requests'][index + 1]['relative_path']).exists())
                self.clear_attempt()

    def clear_attempt(self):
        import shutil
        for path in self.root.iterdir():
            shutil.rmtree(path) if path.is_dir() else path.unlink()

    def test_optional404_keeps_other_algorithms_mandatory404_stops(self):
        record, calls, out, error, index = self.run_main('sidecar404')
        self.assertIsNone(error)
        self.assertEqual(len(record['checksums']), 104)
        self.assertEqual(record['requests'][index + 1]['status'], 404)
        self.clear_attempt()
        record, calls, out, error, index = self.run_main('missing')
        self.assertIsNotNone(error)
        self.assertEqual(len(calls), index + 1)
        self.assertEqual(record['requests'][-1]['status'], 404)

    def test_body_partial_and_aggregate16mib_are_real_file_refusals(self):
        for mode in ['metadata_body', 'body']:
            record, calls, out, error, index = self.run_main(mode)
            self.assertIsNotNone(error)
            self.assertIn('stream byte limit', record['error'])
            if mode == 'metadata_body':
                self.assertEqual(len(calls), index + 1)
                self.assertEqual((out / record['requests'][0]['relative_path']).stat().st_size > 0, True)
                partial = next(p for p in out.rglob('*.partial') if p.name == 'maven-metadata.xml.partial')
                self.assertEqual(partial.stat().st_size, 256 * 1024)
            else:
                self.assertEqual(sum(p.stat().st_size for p in out.rglob('*.partial')), 16 * fetch.MIB)
            self.assertTrue(list(out.rglob('*.response.json')))
            self.clear_attempt()

    def test_real_escaped_header_metadata_charges_both_copies(self):
        record, calls, out, error, _ = self.run_main('headers')
        self.assertIsNone(error)
        self.assertEqual(len(calls), 175)
        files = list(out.rglob('*.json'))
        self.assertEqual(len(files), 177)
        written = sum(p.stat().st_size for p in files)
        self.assertGreater(written, 32 * fetch.MIB)
        self.assertLessEqual(written, 63 * fetch.MIB)
        self.assertIn(b'\\u0001', (out / 'receipt.json').read_bytes())

    def test_header_overflow_and_redirect_preserve_failure_without_fallback(self):
        for mode in ['header_overflow', 'redirect']:
            record, calls, out, error, _ = self.run_main(mode)
            self.assertIsNotNone(error)
            self.assertEqual(len(calls), 1)
            self.assertEqual(sum(p.stat().st_size for p in out.rglob('*.partial')), 0)
            self.assertTrue(list(out.rglob('*.response.json')))
            self.clear_attempt()

    def test_controls_lock_deadline_and_frozen_inputs_refuse_before_get(self):
        for mode in ['controls', 'lock', 'deadline', 'provider_missing', 'duplicate_input']:
            record, calls, out, error, _ = self.run_main(mode)
            self.assertIsNotNone(error)
            self.assertEqual(calls, [])
            self.assertEqual(record['status'], 'FAIL_PRESERVED')
            self.clear_attempt()

    def test_unknown_epoch_or_proposal_fails_before_output_or_owner(self):
        for mode in ['epoch', 'proposal']:
            record, calls, out, error, _ = self.run_main(mode)
            self.assertIsNotNone(error)
            self.assertIsNone(record)
            self.assertEqual(calls, [])
            self.assertEqual(list(out.iterdir()), [])
            self.clear_attempt()

    def test_post_pipeline_input_drift_keeps_all175_and_failed_receipt(self):
        record, calls, out, error, _ = self.run_main('drift')
        self.assertIsNotNone(error)
        self.assertEqual(len(calls), 175)
        self.assertEqual(record['status'], 'FAIL_INPUT_DRIFT')
        self.assertNotEqual(record['inputs_before'], record['inputs_after'])

    def test_checksum_grammar_rejects_all_malformed_records_without_filename_reads(self):
        for algorithm in ['sha512', 'sha256', 'sha1']:
            digest = hashlib.new(algorithm, b'body').hexdigest().encode()
            for suffix in [b'', b'  /remote/pkg.pom\n', b' */remote/pkg.pom\r\n']:
                self.assertEqual(fetch.helper.sidecar_digest(digest.upper() + suffix, algorithm, 'pkg.pom'),
                                 digest.decode().upper())
            for raw in [digest + b'\n' + digest, digest + b' pkg.pom', digest + b'  wrong.pom',
                        digest[:-1], b'g' + digest[1:], digest + b'  /bad\x00/pkg.pom',
                        digest + b'  /bad\t/pkg.pom', b'x' * 65537]:
                with self.subTest(algorithm=algorithm), self.assertRaises(ValueError):
                    fetch.helper.sidecar_digest(raw, algorithm, 'pkg.pom')

    def test_actual_metadata_ledger_preserves_failure_reserve_and_real_files(self):
        metadata = fetch.helper.Metadata()
        value = {'raw': 'x' * (fetch.MIB - 32)}
        written = []
        for index in range(62):
            path = self.root / ('row-' + str(index) + '.json')
            metadata.write(path, value)
            written.append(path)
        self.assertEqual(metadata.used, sum(p.stat().st_size for p in written))
        with self.assertRaisesRegex(ValueError, 'metadata byte limit'):
            metadata.write(self.root / 'refused.json', value)
        self.assertFalse((self.root / 'refused.json').exists())
        record = {'status': 'PASS_PASSIVE_ACQUISITION_ONLY', 'requests': [],
                  'too_large': 'x' * (2 * fetch.MIB), 'inputs_before': {'fixed': 'a' * 64}}
        receipt = self.root / 'receipt.json'
        metadata.finish(receipt, record)
        self.assertEqual(json.loads(receipt.read_text())['status'], 'FAIL_METADATA_LIMIT')
        self.assertEqual(record['inputs_before'], {'fixed': 'a' * 64})
        self.assertLessEqual(metadata.used, 63 * fetch.MIB)

    def test_spec_whole_bound_precedes_output_and_static_frame_counts_both_maps(self):
        spec = self.root / 'oversized-spec.json'
        spec.write_bytes(b' ' * (262144 + 1))
        argv = [str(SOURCE), str(spec), fetch.base.digest(spec), '0', 'unit', 'cookie']
        with patch.object(fetch.sys, 'argv', argv), patch.dict(os.environ,
                {'SYSON_BOOTSTRAP_SOURCE_SHA256': fetch.base.digest(SOURCE)}), \
             patch.object(fetch, 'fresh_output', side_effect=AssertionError('oversize output')):
            with self.assertRaisesRegex(ValueError, 'bounded input'):
                fetch.main()
        inputs = {'control-' + chr(1) * 100: 'a' * 64}
        record = {'requests': [{'headers': [['ignored', 'body']]}],
                  'inputs_before': inputs, 'inputs_after': inputs}
        static = {k: v for k, v in record.items() if k != 'requests'}
        self.assertEqual(fetch.helper.static_frame(17, record), 17 + fetch.helper.encoded_size(static))
        self.assertEqual(json.dumps(static).count('\\u0001'), 200)


    def test_models02_independent18_gavs_and_complete90_row_oracle(self):
        raw = (SOURCE.parent / 'requests/models-02.json').read_bytes()
        self.assertEqual(hashlib.sha256(raw).hexdigest(), '7434575a1030365ba75b4f0650d36183a1e930efd714c367cf4e810db4445ef4')
        gavs = [
            'org.apache:apache:23',
            'org.apache:apache:31',
            'org.apache:apache:39',
            'org.apache.maven:maven-parent:10',
            'org.apache.maven:maven-parent:16',
            'org.apache.maven:maven-parent:23',
            'org.apache.maven:maven-parent:44',
            'org.codehaus.plexus:plexus:25',
            'org.eclipse.sisu:sisu-plexus:0.9.0.M2',
            'org.junit:junit-bom:5.11.1',
            'org.junit:junit-bom:5.11.4',
            'org.junit:junit-bom:5.12.2',
            'org.junit:junit-bom:5.13.1',
            'org.junit:junit-bom:5.14.4',
            'org.junit:junit-bom:5.9.3',
            'org.slf4j:slf4j-bom:2.0.18',
            'org.sonatype.forge:forge-parent:10',
            'org.sonatype.forge:forge-parent:5']
        manifest, rows = json.loads(raw), fetch.requests('models-02', raw)
        self.assertEqual(len(rows), 90)
        self.assertEqual([':'.join(p['coordinate'][k] for k in ['groupId', 'artifactId', 'version'])
                          for p in manifest['new_primaries']], gavs)
        for index, gav in enumerate(gavs):
            group, artifact, version = gav.split(':')
            relative = group.replace('.', '/') + '/' + artifact + '/' + version + '/' + artifact + '-' + version + '.pom'
            for offset, suffix in enumerate(fetch.base.SUFFIXES):
                wanted = {'url': fetch.base.BASE + relative + suffix,
                    'relative_path': 'downloads/' + relative + suffix + '.partial',
                    'maximum_bytes': 65536 if suffix else 2 * fetch.MIB,
                    'role': 'pom' if not suffix else 'signature' if suffix == '.asc' else 'checksum',
                    'allowed_status': [200, 404] if suffix else [200], 'primary_index': index}
                if suffix in ['.sha512', '.sha256', '.sha1']:
                    wanted['algorithm'] = suffix[1:]
                self.assertEqual(rows[index * 5 + offset], wanted)

    def test_models02_coherent_primary_and_cross_epoch_corruptions_refuse(self):
        def coherent(m):
            m['new_primaries'][0]['coordinate']['version'] = '99'
            for item in [m['new_primaries'][0]] + m['requests'][:5]:
                for key in ['url', 'relative_artifact', 'relative_path']:
                    if key in item:
                        item[key] = item[key].replace('/23/', '/99/').replace('-23.pom', '-99.pom')
        for change in [coherent, lambda m: m.update(epoch='models-01'),
                lambda m: m['new_primaries'].reverse(), lambda m: m['requests'][0].update(role='jar'),
                lambda m: m['limits'].update(exact_primaries=True),
                lambda m: m['requests'][1].update(allowed_status=[200.0, 404.0])]:
            self.mutated(change, 'models-02')

    def test_models02_full90_pipeline_gnu_headers_and_optional404(self):
        for mode in [None, 'gnu', 'headers', 'sidecar404']:
            record, calls, out, error, _ = self.run_main(mode, 'models-02')
            self.assertIsNone(error)
            self.assertEqual((len(calls), len(record['requests']), len(record['checksums'])),
                             (90, 90, 53 if mode == 'sidecar404' else 54))
            self.assertEqual(record['status'], 'PASS_PASSIVE_ACQUISITION_ONLY')
            self.assertEqual(record['inputs_before'], record['inputs_after'])
            for row in record['requests']:
                self.assertEqual(hashlib.sha256((out / row['relative_path']).read_bytes()).hexdigest(), row['sha256'])
            self.assertLessEqual(sum(p.stat().st_size for p in out.rglob('*.json')), 63 * fetch.MIB)
            self.assertFalse(any(r['role'] == 'version_metadata' for r in record['requests']))
            self.clear_attempt()

    def test_models02_all_failure_modes_preserve_real_files_and_refuse(self):
        for mode in ['badstrong', 'wrongname', 'missing', 'metadata_body', 'body', 'header_overflow',
                     'redirect', 'controls', 'lock', 'deadline', 'provider_missing', 'duplicate_input',
                     'epoch', 'proposal', 'drift']:
            record, calls, out, error, _ = self.run_main(mode, 'models-02')
            self.assertIsNotNone(error, mode)
            if mode in ['badstrong', 'wrongname']:
                self.assertEqual(len(calls), 90)
                self.assertIn('checksum', record['error'])
            if mode == 'missing':
                self.assertEqual((len(calls), len(record['requests'])), (1, 1))
                self.assertEqual(calls, [fetch.base.BASE + 'org/apache/apache/23/apache-23.pom'])
                self.assertEqual(record['status'], 'FAIL_PRESERVED')
                self.assertEqual(record['error'], "ValueError('HTTP failure retained; no fallback')")
                self.assertNotIn('checksums', record)
                retained = record['requests'][0]
                self.assertEqual((retained['role'], retained['status']), ('pom', 404))
                self.assertEqual(retained['url'], calls[0])
                self.assertEqual(retained['relative_path'],
                                 'downloads/org/apache/apache/23/apache-23.pom.partial')
                payload = out / retained['relative_path']
                self.assertEqual(payload.read_bytes(), b'missing mandatory metadata')
                self.assertEqual((retained['bytes'], retained['sha256']),
                                 (len(b'missing mandatory metadata'),
                                  hashlib.sha256(payload.read_bytes()).hexdigest()))
                response_path = payload.with_suffix(payload.suffix + '.response.json')
                response = json.loads(response_path.read_bytes())
                self.assertEqual((response['status'], response['relative_path']),
                                 (404, retained['relative_path']))
                self.assertEqual(response, retained)
            if mode in ['metadata_body', 'body']:
                self.assertIn('stream byte limit', record['error'])
                self.assertTrue(list(out.rglob('*.response.json')))
            if mode == 'drift':
                self.assertEqual(record['status'], 'FAIL_INPUT_DRIFT')
            self.clear_attempt()


if __name__ == '__main__':
    unittest.main()
