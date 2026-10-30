"""Fake-response controls only: no acquired code, sockets, Maven or JVM."""
import copy
import hashlib
import importlib.util
import io
import json
import os
import signal
import time
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import urllib.error

SOURCE = Path(__file__).resolve().parents[1] / 'fetch.py'
SPEC = importlib.util.spec_from_file_location('passive_fetch_under_test', SOURCE)
fetch = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(fetch)
BASE = 'https://repo.maven.apache.org/maven2/'
MANIFEST = SOURCE.parent / 'requests.json'
MANIFEST_SHA = '9c710bb285eff7ffb52c1608a2e8497451dd86bbd4057ddb444c04d4fcc415a0'
NAMES = [item['relative_artifact'] for item in json.loads(MANIFEST.read_bytes())['new_payloads']]


def proposal():
    return MANIFEST.read_bytes()


def deny_manifest(test, mutate):
    value = json.loads(proposal())
    mutate(value)
    raw = json.dumps(value).encode()
    # Production has no override: patch only to reach independent structural checks.
    with patch.object(fetch, 'REQUEST_SET_SHA', hashlib.sha256(raw).hexdigest()):
        with test.assertRaises(ValueError):
            fetch.requests(raw)


class Response(io.BytesIO):
    def __init__(self, body, url, code=200, fail_after=None):
        super().__init__(body)
        self.url, self.code = url, code
        self.headers = {'Content-Length': str(len(body)), 'X-Fixture': 'synthetic'}
        self.reads, self.fail_after = [], fail_after

    def geturl(self):
        return self.url

    def read(self, size=-1):
        if self.fail_after is not None and len(self.reads) >= self.fail_after:
            raise OSError('synthetic interrupted body')
        self.reads.append(size)
        return super().read(size)


class Opener:
    def __init__(self, response):
        self.response, self.calls = response, []

    def open(self, request, timeout):
        self.calls.append((request.full_url, timeout))
        if isinstance(self.response, Exception):
            raise self.response
        return self.response


class PassiveFetchControls(unittest.TestCase):
    def setUp(self):
        self.assertEqual(fetch.OUTPUT_ROOT, Path('/workspace/build/builderbob'))
        self.assertEqual(fetch.REQUEST_SET_SHA, MANIFEST_SHA)
        self.assertEqual(hashlib.sha256(proposal()).hexdigest(), MANIFEST_SHA)
        root = Path(os.environ['SYSON_BOOTSTRAP_TEST_ROOT'])
        if not root.is_absolute() or root.resolve() != root or not root.is_dir():
            raise AssertionError('existing canonical test fixture root required')
        if os.environ.get('GITHUB_ACTIONS') == 'true':
            runner = Path(os.environ['RUNNER_TEMP'])
            if not runner.is_absolute() or runner.resolve() != runner or not runner.is_dir():
                raise AssertionError('canonical GitHub runner temporary directory required')
            if not root.is_relative_to(runner):
                raise AssertionError('hosted fixtures must stay beneath runner.temp')
        elif not root.is_relative_to('/workspace/build/builderbob'):
            raise AssertionError('local test data must remain on admitted workspace')
        self.temporary = tempfile.TemporaryDirectory(prefix='fetch-controls-', dir=root)
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        fixture_root = patch.object(fetch, 'OUTPUT_ROOT', self.root)
        fixture_root.start()
        self.addCleanup(fixture_root.stop)
        self.socket_guard = patch('socket.create_connection', side_effect=AssertionError('network forbidden'))
        self.socket_guard.start()
        self.addCleanup(self.socket_guard.stop)

    def test_frozen_request_set_has_exact_395_urls_and_destinations(self):
        rows = fetch.requests(proposal())
        self.assertEqual(len(rows), 395)
        self.assertEqual(rows[0]['url'], BASE + NAMES[0])
        self.assertEqual(rows[-1]['relative_path'], 'downloads/' + NAMES[-1] + '.asc.partial')

    def test_request_mutations_refuse_url_order_count_path_and_caps(self):
        changes = [
            lambda p: p['requests'][0].update(url=BASE.replace('https:', 'http:') + NAMES[0]),
            lambda p: p['requests'][0].update(url=BASE.replace('repo.maven.apache.org', 'example.invalid') + NAMES[0]),
            lambda p: p['requests'][0].update(url=BASE + NAMES[0] + '?extra=1'),
            lambda p: p['requests'][0].update(relative_path='../escape'),
            lambda p: p['requests'][0].update(maximum_bytes=2 * 1024**2 + 1),
            lambda p: p['requests'][1].update(maximum_bytes=65537),
            lambda p: p['limits'].update(exact_requests=396),
            lambda p: p['limits'].update(payload_and_sidecar_total_bytes=128 * 1024**2 + 1),
            lambda p: p['requests'].reverse(),
            lambda p: p['requests'].pop(),
        ]
        for index, change in enumerate(changes):
            with self.subTest(mutation=index):
                deny_manifest(self, change)

    def test_fresh_output_rejects_occupied_symlink_and_noncanonical_path(self):
        empty = self.root / 'empty'
        empty.mkdir(mode=0o700)
        fetch.fresh_output(empty)
        link = self.root / 'escaping-output-link'
        link.symlink_to(empty, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, 'fresh owned output'):
            fetch.fresh_output(link)
        with self.assertRaisesRegex(ValueError, 'fresh owned output'):
            fetch.fresh_output(self.root / 'empty' / '..' / 'empty')
        (empty / 'existing').write_bytes(b'preserved')
        with self.assertRaisesRegex(ValueError, 'fresh owned output'):
            fetch.fresh_output(empty)
        self.assertEqual((empty / 'existing').read_bytes(), b'preserved')

    def row(self):
        return fetch.requests(proposal())[1]  # .pom.sha512: 64KiB sidecar cap

    def test_success_preserves_exact_body_hash_and_status(self):
        row, body = self.row(), b'synthetic served bytes\x00\xff'
        response = Response(body, row['url'])
        opener, path = Opener(response), self.root / 'served.partial'
        result = fetch.transfer(opener, row, path, 40 * 1024**2)
        self.assertEqual(path.read_bytes(), body)
        self.assertEqual(result['sha256'], hashlib.sha256(body).hexdigest())
        self.assertEqual(result['status'], 200)
        self.assertEqual(opener.calls, [(row['url'], 15)])
        self.assertTrue(response.closed)

    def check_header_fixture(self, label, headers, expected_prefix, refused):
        row, body = self.row(), b'body must remain unread on metadata refusal'
        path = self.root / (label + '.partial')
        response = Response(body, row['url'])
        response.headers = headers
        if refused:
            with self.assertRaisesRegex(ValueError, 'header metadata limit'):
                fetch.transfer(Opener(response), row, path, 1000)
            self.assertEqual(response.reads, [])
            self.assertEqual(path.read_bytes(), b'')
        else:
            fetch.transfer(Opener(response), row, path, 1000)
            self.assertTrue(response.reads)
            self.assertEqual(path.read_bytes(), body)
        self.assertTrue(response.closed)
        record = json.loads(path.with_suffix('.partial.response.json').read_text())
        self.assertEqual(record['headers'], [list(pair) for pair in expected_prefix])
        self.assertEqual(record['bytes'], 0 if refused else len(body))
        self.assertEqual(record['sha256'], hashlib.sha256(b'' if refused else body).hexdigest())
        self.assertLessEqual(len(record['headers']), 64)
        self.assertLessEqual(sum(len(k.encode('utf-8')) + len(v.encode('utf-8')) + 4
                                 for k, v in record['headers']), 16384)

    def test_header_count_exact_boundary_and_overflow_keep_only_bounded_prefix(self):
        headers = {f'X-{i}': 'v' for i in range(64)}
        self.check_header_fixture('headers64', headers, list(headers.items()), False)
        expanded = dict(headers, Extra='never retained')
        self.check_header_fixture('headers65', expanded, list(headers.items()), True)

    def test_header_byte_exact_boundary_and_one_byte_overflow(self):
        # First pair costs 6bytes; second pair costs5bytes plus its value.
        prefix = [('A', 'B')]
        exact = dict(prefix + [('X', 'x' * 16373)])
        self.check_header_fixture('header-bytes-exact', exact, list(exact.items()), False)
        overflow = dict(prefix + [('X', 'x' * 16374)])
        self.check_header_fixture('header-bytes-overflow', overflow, prefix, True)

    def test_header_unicode_budget_counts_utf8_keys_values_and_crlf(self):
        prefix = [('A', 'B')]  # 6bytes,including key/value separator and CRLF.
        value = '😀' * 4093  # 16372 UTF8bytes,not4093.
        exact = dict(prefix + [('é', value)])  # 6+2+16372+4 ==16384.
        self.assertLess(sum(len(k) + len(v) + 4 for k, v in exact.items()), 16384)
        self.check_header_fixture('header-unicode-exact', exact, list(exact.items()), False)
        overflow = dict(prefix + [('é', value + 'x')])
        self.check_header_fixture('header-unicode-overflow', overflow, prefix, True)

    def test_redirect_handler_and_changed_final_url_never_follow(self):
        self.assertIsNone(fetch.NoRedirect().redirect_request(None, None, 302, '', {}, 'https://example.invalid/'))
        row, path = self.row(), self.root / 'redirect.partial'
        response = Response(b'not an artifact', 'https://example.invalid/')
        with self.assertRaisesRegex(ValueError, 'redirect refused'):
            fetch.transfer(Opener(response), row, path, 100)
        self.assertEqual(path.read_bytes(), b'')
        self.assertTrue(response.closed)
        record = json.loads(path.with_suffix('.partial.response.json').read_text())
        self.assertEqual(record['final_url'], 'https://example.invalid/')
        self.assertEqual(record['sha256'], hashlib.sha256(b'').hexdigest())

    def test_non200_body_and_headers_are_retained_without_becoming_200(self):
        row, path = self.row(), self.root / 'failure.partial'
        error = urllib.error.HTTPError(row['url'], 403, 'Forbidden', {'X-Fixture': 'denied'}, io.BytesIO(b'denied body'))
        result = fetch.transfer(Opener(error), row, path, 100)
        self.assertEqual(result['status'], 403)
        self.assertEqual(path.read_bytes(), b'denied body')
        self.assertIn(('X-Fixture', 'denied'), result['headers'])

    def test_open_failure_still_retains_failure_metadata_without_an_artifact(self):
        row, path = self.row(), self.root / 'open-failure.partial'
        with self.assertRaisesRegex(OSError, 'synthetic connection refused'):
            fetch.transfer(Opener(OSError('synthetic connection refused')), row, path, 100)
        self.assertFalse(path.exists())
        record = json.loads(path.with_suffix('.partial.response.json').read_text())
        self.assertEqual(record['url'], row['url'])
        self.assertIsNone(record['status'])
        self.assertIsNone(record['sha256'])
        self.assertEqual(record['bytes'], 0)

    def test_file_and_remaining_aggregate_limits_keep_exact_partial_prefix(self):
        row = self.row()
        for label, available, retained in [('file', 100000, 65536), ('aggregate', 3, 3)]:
            with self.subTest(limit=label):
                path = self.root / (label + '.partial')
                response = Response(b'x' * 65537, row['url'])
                with self.assertRaisesRegex(ValueError, 'stream byte limit'):
                    fetch.transfer(Opener(response), row, path, available)
                self.assertEqual(path.read_bytes(), b'x' * retained)
                record = json.loads(path.with_suffix('.partial.response.json').read_text())
                self.assertEqual(record['bytes'], retained)
                self.assertEqual(record['sha256'], hashlib.sha256(b'x' * retained).hexdigest())
                self.assertTrue(response.closed)

    def test_interrupted_stream_keeps_partial_body_and_response_record(self):
        row, path = self.row(), self.root / 'interrupted.partial'
        response = Response(b'partial bytes', row['url'], fail_after=1)
        with self.assertRaisesRegex(OSError, 'synthetic interrupted body'):
            fetch.transfer(Opener(response), row, path, 100)
        self.assertEqual(path.read_bytes(), b'partial bytes')
        record = json.loads(path.with_suffix('.partial.response.json').read_text())
        self.assertEqual(record['bytes'], 13)
        self.assertTrue(response.closed)

    def test_deadline_interrupts_a_blocked_read_and_preserves_partial_metadata(self):
        row, path = self.row(), self.root / 'blocked.partial'
        class BlockingResponse(Response):
            def read(self, size=-1):
                if self.reads:
                    time.sleep(1)  # Actual blocking operation interrupted by the signal.
                return super().read(size)
        response = BlockingResponse(b'prefix', row['url'])
        original = fetch.request_deadline
        previous = signal.getsignal(signal.SIGALRM)
        with patch.object(fetch, 'request_deadline', side_effect=lambda _: original(0.02)):
            with self.assertRaisesRegex(TimeoutError, 'request wall deadline'):
                fetch.transfer(Opener(response), row, path, 100)
        self.assertEqual(path.read_bytes(), b'prefix')
        self.assertEqual(json.loads(path.with_suffix('.partial.response.json').read_text())['bytes'], 6)
        self.assertTrue(response.closed)
        self.assertEqual(signal.getitimer(signal.ITIMER_REAL), (0.0, 0.0))
        self.assertIs(signal.getsignal(signal.SIGALRM), previous)

    def test_foreign_timer_is_refused_without_replacing_timer_or_handler(self):
        previous = signal.getsignal(signal.SIGALRM)
        self.assertEqual(signal.getitimer(signal.ITIMER_REAL), (0.0, 0.0))
        def foreign_handler(signum, frame):
            raise AssertionError('foreign timer unexpectedly fired')
        signal.signal(signal.SIGALRM, foreign_handler)
        signal.setitimer(signal.ITIMER_REAL, 60)
        try:
            with self.assertRaisesRegex(ValueError, 'existing/invalid request timer'):
                with fetch.request_deadline(0.02):
                    self.fail('foreign timer must prevent entering request')
            self.assertIs(signal.getsignal(signal.SIGALRM), foreign_handler)
            self.assertGreater(signal.getitimer(signal.ITIMER_REAL)[0], 0)
        finally:
            signal.setitimer(signal.ITIMER_REAL, 0)
            signal.signal(signal.SIGALRM, previous)

    def test_deadline_restores_handler_and_clears_timer_when_body_raises(self):
        previous = signal.getsignal(signal.SIGALRM)
        with self.assertRaisesRegex(RuntimeError, 'synthetic body failure'):
            with fetch.request_deadline(60):
                raise RuntimeError('synthetic body failure')
        self.assertEqual(signal.getitimer(signal.ITIMER_REAL), (0.0, 0.0))
        self.assertIs(signal.getsignal(signal.SIGALRM), previous)

    def test_raw_manifest_hash_and_duplicate_json_keys_refuse(self):
        with self.assertRaisesRegex(ValueError, 'fixed manifest hash'):
            fetch.requests(proposal() + b'\n')
        raw = b'{"limits":{},"limits":{}}'
        with patch.object(fetch, 'REQUEST_SET_SHA', hashlib.sha256(raw).hexdigest()):
            with self.assertRaisesRegex(ValueError, 'duplicate manifest key'):
                fetch.requests(raw)

    def test_manifest_payload_duplicates_coordinates_and_metadata_profile_refuse(self):
        def duplicate_payload_and_its_five_requests(p):
            p['new_payloads'][1] = copy.deepcopy(p['new_payloads'][0])
            p['requests'][5:10] = copy.deepcopy(p['requests'][:5])
        changes = [
            duplicate_payload_and_its_five_requests,
            lambda p: p['new_payloads'][1]['coordinate'].update(version='wrong'),
            lambda p: p['new_payloads'][1].update(relative_artifact='org/ow2/asm/asm/9.9/wrong.pom'),
            lambda p: p['new_payloads'].pop(),
            lambda p: p['requests'][0].update(relative_path='/absolute/file'),
            lambda p: p['limits'].update(all_metadata_and_log_bytes_proposed=96 * 1024**2 + 1),
        ]
        for index, change in enumerate(changes):
            with self.subTest(mutation=index):
                deny_manifest(self, change)

    def test_nested_destination_preserves_existing_files_and_refuses_symlinks(self):
        row = fetch.requests(proposal())[0]
        path = fetch.destination(self.root, row)
        self.assertEqual(path.relative_to(self.root).as_posix(), row['relative_path'])
        self.assertEqual(path.parent.resolve(), path.parent)
        path.write_bytes(b'preserve existing bytes')
        with self.assertRaisesRegex(ValueError, 'existing destination'):
            fetch.destination(self.root, row)
        self.assertEqual(path.read_bytes(), b'preserve existing bytes')
        unsafe, outside = self.root / 'unsafe', self.root / 'outside'
        unsafe.mkdir(mode=0o700)
        outside.mkdir(mode=0o700)
        (unsafe / 'downloads').symlink_to(outside, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, 'nested symlink'):
            fetch.destination(unsafe, row)
        self.assertEqual(list(outside.iterdir()), [])
        for bad in ['downloads/../escape', 'downloads//a', '/downloads/a']:
            with self.subTest(path=bad), self.assertRaises(ValueError):
                fetch.destination(self.root, dict(row, relative_path=bad))

    def test_exact_128mib_aggregate_boundary_refuses_without_large_allocation(self):
        self.assertEqual(fetch.PAYLOAD_LIMIT, 134217728)
        self.assertEqual(fetch.remaining(134217728), 0)
        for total in [-1, 134217729, True]:
            with self.subTest(total=total), self.assertRaises(ValueError):
                fetch.remaining(total)
        row, path = self.row(), self.root / 'aggregate-last-byte.partial'
        response = Response(b'xy', row['url'])
        with self.assertRaisesRegex(ValueError, 'stream byte limit'):
            fetch.transfer(Opener(response), row, path, fetch.remaining(134217727))
        self.assertEqual(path.read_bytes(), b'x')
        self.assertTrue(response.closed)

    def test_metadata_charges_existing_proof_and_actual_serialized_bytes(self):
        proof = self.root / 'scope-proof.json'
        proof.write_bytes(b'fixed synthetic invocation proof')
        metadata = fetch.Metadata(used=proof.stat().st_size)
        path = self.root / 'response.json'
        metadata.response(path, {'header': 'é😀'})
        self.assertEqual(metadata.used, proof.stat().st_size + path.stat().st_size)
        self.assertEqual(json.loads(path.read_text()), {'header': 'é😀'})

    def test_96mib_metadata_profile_counts_proof_and_reserves_failure_receipt(self):
        self.assertEqual(fetch.METADATA_LIMIT, 100663296)
        early = {'bounded': 'retained'}
        encoded = (json.dumps(early, indent=2) + '\n').encode()
        metadata = fetch.Metadata(used=94 * 1024**2 - len(encoded))
        metadata.response(self.root / 'earlier.json', early)
        self.assertEqual(metadata.used, 94 * 1024**2)
        failed = {'url': 'fixed synthetic response', 'headers': [['X', 'bounded prefix']]}
        with self.assertRaisesRegex(ValueError, 'metadata byte limit'):
            metadata.response(self.root / 'overflow.json', failed)
        self.assertFalse((self.root / 'overflow.json').exists())
        record = {'status': 'PASS_PASSIVE_ACQUISITION_ONLY', 'framing': 'x' * 1024**2,
                  'inputs_before': {'fixed': 'before'}, 'inputs_after': {'fixed': 'after'}}
        metadata.finish(self.root / 'receipt.json', record)
        self.assertEqual(record['status'], 'FAIL_METADATA_LIMIT')
        self.assertEqual(record['failed_response'], failed)
        self.assertEqual(record['inputs_before'], {'fixed': 'before'})
        self.assertEqual(record['inputs_after'], {'fixed': 'after'})
        self.assertLessEqual(metadata.used, 95 * 1024**2)
        self.assertEqual(json.loads((self.root / 'earlier.json').read_text()), early)
        self.assertEqual(json.loads((self.root / 'receipt.json').read_text()), record)
        with self.assertRaises(ValueError):
            fetch.Metadata(limit=96 * 1024**2)

    def test_actual_transfer_records_feed_checksum_validation_without_losing_paths(self):
        rows = fetch.requests(proposal())[:5]
        body = b'opaque passive payload'
        bodies = [body, hashlib.sha512(body).hexdigest().encode(),
                  hashlib.sha256(body).hexdigest().encode(), hashlib.sha1(body).hexdigest().encode(),
                  b'unverified synthetic signature']
        records, total, metadata = [], 0, fetch.Metadata()
        for row, content in zip(rows, bodies):
            response = Response(content, row['url'])
            value = fetch.transfer(Opener(response), row, fetch.destination(self.root, row),
                                   fetch.remaining(total), metadata)
            records.append(value)
            total += value['bytes']
            self.assertEqual(value['relative_path'], row['relative_path'])
            self.assertEqual(value['role'], row['role'])
        expected = [{'artifact': NAMES[0], 'algorithm': algorithm, 'match': True}
                    for algorithm in ['sha512', 'sha256', 'sha1']]
        self.assertEqual(fetch.checksums(self.root, records), expected)

    def checksum_fixture(self):
        downloads = self.root / 'downloads'
        downloads.mkdir()
        for name in NAMES:
            artifact = downloads / (name + '.partial')
            artifact.parent.mkdir(parents=True, exist_ok=True)
            artifact.write_bytes(b'opaque synthetic artifact')
        rows = [dict(row, status=200 if row['role'] in ('pom', 'jar') else 404)
                for row in fetch.requests(proposal())]
        return downloads, rows

    def enable_checksum(self, downloads, rows, suffix, content):
        url = BASE + NAMES[0] + suffix
        next(row for row in rows if row['url'] == url)['status'] = 200
        (downloads / (NAMES[0] + suffix + '.partial')).write_text(content)

    def test_strong_checksum_matches_uppercase_and_legacy_is_labelled_sha1(self):
        downloads, rows = self.checksum_fixture()
        body = b'opaque synthetic artifact'
        self.enable_checksum(downloads, rows, '.sha512', hashlib.sha512(body).hexdigest().upper() + '\n')
        self.enable_checksum(downloads, rows, '.sha1', hashlib.sha1(body).hexdigest())
        self.assertEqual(fetch.checksums(self.root, rows), [
            {'artifact': NAMES[0], 'algorithm': 'sha512', 'match': True},
            {'artifact': NAMES[0], 'algorithm': 'sha1', 'match': True}])

    def test_bad_strong_checksum_cannot_fall_back_to_valid_legacy(self):
        downloads, rows = self.checksum_fixture()
        self.enable_checksum(downloads, rows, '.sha512', '0' * 128)
        self.enable_checksum(downloads, rows, '.sha1', hashlib.sha1(b'opaque synthetic artifact').hexdigest())
        with self.assertRaisesRegex(ValueError, 'checksum mismatch'):
            fetch.checksums(self.root, rows)

    def test_malformed_checksum_refuses_and_absent_checksums_grant_no_match(self):
        downloads, rows = self.checksum_fixture()
        self.assertEqual(fetch.checksums(self.root, rows), [])
        self.enable_checksum(downloads, rows, '.sha256', 'not a checksum')
        with self.assertRaisesRegex(ValueError, 'invalid sidecar'):
            fetch.checksums(self.root, rows)


if __name__ == '__main__':
    unittest.main()
