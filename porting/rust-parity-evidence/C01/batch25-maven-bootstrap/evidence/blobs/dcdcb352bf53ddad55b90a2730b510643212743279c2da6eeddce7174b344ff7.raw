"""Mock HTTP and inspect synthetic ZIP/XML only; no network or acquired code runs."""
import copy
import hashlib
import io
import json
import os
from pathlib import Path
import tempfile
import time
from types import ModuleType
import unittest
from unittest.mock import patch
import zipfile


class BootstrapTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        path = Path(__file__).resolve().parents[1] / 'acquire_plugins.py'
        raw = path.read_bytes()
        if hashlib.sha256(raw).hexdigest() != os.environ['SYSON_C01_BOOTSTRAP_SOURCE_SHA256']:
            raise ValueError('reviewed bootstrap source hash required')
        cls.subject = ModuleType('reviewed_bootstrap')
        cls.subject.__file__ = str(path)
        exec(compile(raw, str(path), 'exec'), cls.subject.__dict__)
        cls.proposal = json.loads((path.parent / 'proposal.json').read_bytes())
        cls.temporary_root = Path(os.environ['SYSON_C01_BOOTSTRAP_TEST_TMP']).resolve()
        if not cls.temporary_root.is_relative_to('/workspace') or not cls.temporary_root.is_dir():
            raise ValueError('existing admitted Samsung test directory required')

    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(dir=self.temporary_root)
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        for name in ['downloads', 'inspection']:
            (self.root / name).mkdir()
        replacement = patch.object(self.subject, 'OUTPUT', self.root)
        replacement.start()
        self.addCleanup(replacement.stop)
        self.spec = copy.deepcopy(self.proposal)
        self.item = self.spec['artifacts'][0]

    def response(self, raw, declared=None, url=None, status=200):
        value = io.BytesIO(raw)
        value.status, value.url = status, url or self.item['url']
        value.headers = {} if declared is None else {'Content-Length': str(declared)}
        return value

    def fetch(self, response, expected):
        class Opener:
            def open(self, *args, **kwargs):
                return response
        self.record = {'requests': [], 'entity_bytes': 0}
        with patch.object(self.subject.signal, 'setitimer'):
            return self.subject.fetch(Opener(), self.item['url'], 'artifact', expected,
                                      self.spec, self.record, time.monotonic() + 10)

    def descriptor_bytes(self, goal='effective-pom'):
        return ('<plugin><groupId>org.apache.maven.plugins</groupId>'
                '<artifactId>maven-help-plugin</artifactId><version>3.5.1</version>'
                '<mojos><mojo><goal>' + goal + '</goal><implementation>Fixture</implementation>'
                '</mojo></mojos></plugin>').encode()

    def jar(self, entries):
        path = self.root / 'fixture.jar'
        with zipfile.ZipFile(path, 'w') as archive:
            for name, raw in entries:
                archive.writestr(name, raw)
        return path

    def test_only_twelve_literal_https_urls_are_admitted(self):
        for item in self.spec['artifacts']:
            for key in ['url', 'checksum_url']:
                self.subject.url_allowed(item[key], self.spec)
        self.assertEqual(len(self.spec['artifacts']), 6)
        for url in ['http://repo.maven.apache.org/a', self.item['url'] + '?token=fixture',
                    self.item['url'].replace('repo.maven.apache.org', 'unreviewed.example'),
                    self.item['url'].replace('https://', 'https://user:password@')]:
            with self.assertRaisesRegex(RuntimeError, 'literal approved'):
                self.subject.url_allowed(url, self.spec)
        with self.assertRaisesRegex(RuntimeError, 'redirects refused'):
            self.subject.NoRedirect().redirect_request(None, None, 302, '', {}, self.item['url'])

    def test_exact_body_and_eof_are_observed_and_hashed(self):
        path = self.fetch(self.response(b'abc', 3), 3)
        self.assertEqual(path.read_bytes(), b'abc')
        self.assertTrue(self.record['requests'][0]['complete'])
        self.assertEqual(self.record['requests'][0]['sha256'], hashlib.sha256(b'abc').hexdigest())

    def test_truncated_long_and_misdeclared_bodies_preserve_partial_attempt(self):
        for raw, declared in [(b'ab', None), (b'abcd', None), (b'abc', 4)]:
            with self.subTest(raw_length=len(raw), declared=declared):
                with self.assertRaises(RuntimeError):
                    self.fetch(self.response(raw, declared), 3)
                self.assertFalse(self.record['requests'][0]['complete'])
                self.assertIn('sha256', self.record['requests'][0])
                (self.root / 'downloads/artifact').unlink()

    def test_wrong_status_or_final_url_and_total_byte_cap_refuse(self):
        for status, url in [(403, self.item['url']), (200, 'https://unreviewed.example/')]:
            with self.assertRaisesRegex(RuntimeError, 'status/URL'):
                self.fetch(self.response(b'abc', 3, url, status), 3)
            (self.root / 'downloads/artifact').unlink()
        self.spec['bounds']['entity_bytes'] = 2
        with self.assertRaisesRegex(RuntimeError, 'body byte bound'):
            self.fetch(self.response(b'abc', 3), 3)

    def test_sidecar_pin_and_primary_digest_both_reject_changed_bytes(self):
        primary = self.root / 'primary'
        checksum = self.root / 'checksum'
        primary.write_bytes(b'fixture')
        item = {'sha1': hashlib.sha1(b'fixture').hexdigest()}
        checksum.write_bytes(item['sha1'].encode())
        item['checksum_sha256'] = self.subject.hashed(checksum)
        self.subject.verify_checksum(checksum, item)
        self.subject.verify_primary(primary, item)
        checksum.write_bytes(b'0' * 40)
        with self.assertRaisesRegex(RuntimeError, 'checksum metadata'):
            self.subject.verify_checksum(checksum, item)
        primary.write_bytes(b'changed')
        with self.assertRaisesRegex(RuntimeError, 'artifact digest'):
            self.subject.verify_primary(primary, item)

    def test_selected_descriptor_is_retained_as_data_without_extraction(self):
        raw = self.descriptor_bytes()
        path = self.jar([('META-INF/maven/plugin.xml', raw), ('Fixture.class', b'not executable')])
        result = self.subject.jar_inventory(path, self.item, self.spec)
        self.assertEqual(result['descriptor']['mojos'][0]['goal'], 'effective-pom')
        self.assertEqual((self.root / 'inspection/maven-help-plugin-plugin.xml').read_bytes(), raw)
        self.assertFalse((self.root / 'Fixture.class').exists())

    def test_duplicate_traversal_and_file_parent_paths_refuse(self):
        for entries in [[('../escape', b'x')], [('a', b'x'), ('a/b', b'y')],
                        [('a', b'x'), ('a/', b'')], [('a//', b'')]]:
            with self.subTest(names=[name for name, _ in entries]):
                path = self.jar(entries + [('META-INF/maven/plugin.xml', self.descriptor_bytes())])
                with self.assertRaisesRegex(RuntimeError, 'ZIP path/duplicate|ZIP file/directory'):
                    self.subject.jar_inventory(path, self.item, self.spec)

    def test_expansion_and_entry_count_bounds_apply_before_selected_reads(self):
        path = self.jar([('META-INF/maven/plugin.xml', self.descriptor_bytes())])
        for key in ['zip_entries', 'zip_entry_bytes', 'zip_uncompressed_bytes', 'selected_entry_bytes']:
            spec = copy.deepcopy(self.spec)
            spec['bounds'][key] = 0
            with self.subTest(bound=key), self.assertRaises(RuntimeError):
                self.subject.jar_inventory(path, self.item, spec)

    def test_symlink_zip_entry_is_not_admitted(self):
        entry = zipfile.ZipInfo('link')
        entry.create_system = 3
        entry.external_attr = 0o120777 << 16
        path = self.jar([(entry, b'outside')])
        with self.assertRaisesRegex(RuntimeError, 'file type'):
            self.subject.jar_inventory(path, self.item, self.spec)

    def test_source_archive_is_inventoried_without_source_or_class_execution(self):
        item = copy.deepcopy(self.item)
        item['name'] = 'maven-help-plugin-3.5.1-sources.jar'
        path = self.jar([('Fixture.java', b'public class Fixture {}')])
        result = self.subject.jar_inventory(path, item, self.spec)
        self.assertIsNone(result['descriptor'])
        self.assertEqual(result['selected'], [])
        self.assertFalse((self.root / 'Fixture.java').exists())

    def test_xml_dtd_entity_utf16_and_missing_or_wrong_goal_are_refused(self):
        for raw in [b'<!DOCTYPE plugin><plugin/>', b'<!ENTITY x "fixture"><plugin/>',
                    '<!DOCTYPE plugin><plugin/>'.encode('utf-16')]:
            with self.assertRaises((RuntimeError, UnicodeError)):
                self.subject.xml_data(raw)
        for raw in [self.descriptor_bytes('wrong'), self.descriptor_bytes().replace(b'3.5.1', b'wrong'),
                    self.descriptor_bytes().replace(b'<goal>', b'<goal>duplicate</goal><goal>')]:
            with self.assertRaises(RuntimeError):
                self.subject.descriptor(raw, self.item, self.spec)

    def test_output_bound_and_basename_containment_are_enforced(self):
        with self.assertRaisesRegex(RuntimeError, 'unsafe selected'):
            self.subject.write_data('../escape', b'data', self.spec)
        self.spec['bounds']['output_bytes'] = 0
        with self.assertRaisesRegex(RuntimeError, 'output byte bound'):
            self.subject.write_data('safe', b'data', self.spec)


if __name__ == '__main__':
    unittest.main()
