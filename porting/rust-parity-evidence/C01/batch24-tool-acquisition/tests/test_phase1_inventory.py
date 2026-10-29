"""Inventory-only controls; never download, decompress, extract or run tool code."""
import copy
import hashlib
import io
import json
from pathlib import Path
import tarfile
import types
import unittest
from unittest.mock import patch

BASE = Path(__file__).resolve().parents[1]
RUNNER = BASE / 'phase1-acquire-attempt02.py'
RUNNER_SHA256 = '50e7913977c7c2f599c20ed84818a62907724fbfd8de7733cfba4f999b7c15b3'
PRIOR = BASE.parent / 'c01-native-build-01'
ROOT = 'apache-maven-3.9.16'


def load_runner():
    source = RUNNER.read_bytes()
    if hashlib.sha256(source).hexdigest() != RUNNER_SHA256:
        raise RuntimeError('reviewed inventory runner changed')
    module = types.ModuleType('c01_attempt02_inventory_controls')
    module.__file__ = str(RUNNER)
    exec(compile(source, str(RUNNER), 'exec'), module.__dict__)
    return module


def member(name=None, kind=tarfile.DIRTYPE, **metadata):
    item = tarfile.TarInfo(name or ROOT + '/d')
    item.type, item.mode, item.mtime = kind, 0o755, 1778708165
    item.uid = item.gid = 0
    item.uname = item.gname = 'root'
    for key, value in metadata.items():
        setattr(item, key, value)
    return item


def encoded(item, body=b''):
    if item.isfile():
        item = copy.copy(item)
        item.size = len(body)
    header = item.tobuf(format=tarfile.PAX_FORMAT)
    return header + body + bytes((-len(body)) % 512)


def archive_bytes(*items):
    return b''.join(encoded(item, body) for item, body in items) + bytes(1024)


class InventoryControls(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.subject = load_runner()
        cls.proposal = json.loads((BASE / 'proposal-attempt02.json').read_bytes())
        cls.tool = next(t for t in cls.proposal['tools'] if t['id'] == 'maven')

    def admit(self, raw, maximum=None):
        tool = dict(self.tool)
        if maximum is not None:
            tool['max_members'] = maximum
        with patch.object(tarfile.TarFile, 'extractall', side_effect=RuntimeError('extraction forbidden')):
            with patch.object(self.subject, 'extract_and_hash', side_effect=RuntimeError('extraction forbidden')):
                with tarfile.open(fileobj=io.BytesIO(raw), mode='r:') as archive:
                    return self.subject.inventory(archive, tool)

    def refused(self, raw, maximum=None):
        with self.assertRaises(self.subject.Refusal):
            self.admit(raw, maximum)

    def test_proposal_pairing_and_original_files_unchanged(self):
        proposal = (BASE / 'proposal-attempt02.json').read_bytes()
        self.assertEqual(self.subject.PROPOSAL_SHA256, hashlib.sha256(proposal).hexdigest())
        expected = {
            'proposal.json': 'fe59578617ea153e8fce8689c8c3612d1e098efc03bdd30d6f1cfab79814dc46',
            'phase1-acquire.py': 'c0fcc5c77e1fef3fbb2928e4dad96c34a607bddec7eb9531fa2d899b8062f0d8',
        }
        for name, digest in expected.items():
            self.assertEqual(hashlib.sha256((BASE / name).read_bytes()).hexdigest(), digest)
        self.assertEqual([t['id'] for t in self.proposal['tools']], ['maven', 'node-npm'])
        self.assertEqual(self.proposal['reused_JDK']['inventory']['members'], 542)
        self.assertTrue(self.proposal['destinations']['fresh_task_root'].endswith('build-02'))

    def test_actual_pinned_maven_tar_retains_all_records_and_first_entry(self):
        compressed = PRIOR / 'downloads/apache-maven-3.9.16-bin.tar.gz.partial'
        with compressed.open('rb') as stream:
            self.assertEqual(hashlib.file_digest(stream, 'sha512').hexdigest(), self.tool['checksum'])
        raw = (PRIOR / 'downloads/maven.tar.partial').read_bytes()
        self.assertEqual(hashlib.sha256(raw).hexdigest(),
                         'fe51dcb0f38d5f5079014979d4fc7dd0ec49429eb206ccd4edf422089e7fc090')
        members, rows, total, targets = self.admit(raw)
        self.assertEqual((len(members), len(rows), total, targets), (108, 109, 10869833, {}))
        duplicates = [row for row in rows if row['duplicate_directory']]
        self.assertEqual(len(duplicates), 1)
        row = duplicates[0]
        self.assertEqual((row['record_index'], row['offset'], row['canonical_offset']), (53, 703488, 326656))
        self.assertEqual(row['normalized_name'], ROOT + '/lib/jansi-native')
        self.assertEqual(members[row['normalized_name']].offset, 326656)
        self.assertEqual([r['record_index'] for r in rows], list(range(1, 110)))
        self.assertEqual(raw[326656:327168], raw[703488:704000])
        with self.assertRaises(self.subject.Refusal):
            self.admit(raw, maximum=108)

    def test_exact_directory_repeat_is_admitted_but_remains_two_records(self):
        directory = member()
        members, rows, total, targets = self.admit(archive_bytes((directory, b''), (directory, b'')))
        self.assertEqual((len(members), len(rows), total, targets), (1, 2, 0, {}))
        self.assertEqual([r['duplicate_directory'] for r in rows], [False, True])
        self.assertEqual(members[ROOT + '/d'].offset, 0)
        self.assertEqual(rows[1]['canonical_offset'], 0)

    def test_each_directory_metadata_change_is_refused(self):
        changes = {'mode': 0o700, 'mtime': 1778708166, 'uid': 1, 'gid': 1,
                   'uname': 'other', 'gname': 'other', 'linkname': 'other'}
        for field, value in changes.items():
            with self.subTest(field=field):
                self.refused(archive_bytes((member(), b''), (member(**{field: value}), b'')))

    def test_directory_type_and_nonzero_payload_conflicts_are_refused(self):
        for kind in (tarfile.REGTYPE, tarfile.SYMTYPE, tarfile.LNKTYPE):
            with self.subTest(kind=kind):
                self.refused(archive_bytes((member(), b''), (member(kind=kind), b'')))
        self.refused(archive_bytes((member(), b''), (member(size=1), b'x')))
        self.refused(archive_bytes((member(kind=tarfile.REGTYPE), b'x'), (member(), b'')))

    def test_identical_regular_files_and_links_are_still_refused(self):
        for kind in (tarfile.REGTYPE, tarfile.SYMTYPE, tarfile.LNKTYPE):
            item = member(kind=kind, linkname=ROOT + '/target' if kind == tarfile.LNKTYPE else 'target')
            body = b'x' if item.isfile() else b''
            with self.subTest(kind=kind):
                self.refused(archive_bytes((member(ROOT + '/target', tarfile.REGTYPE), b'x'),
                                          (item, body), (item, body)))

    def test_pax_directory_duplicates_are_not_relaxed(self):
        directory = member(pax_headers={'comment': 'same extension metadata'})
        self.refused(archive_bytes((directory, b''), (directory, b'')))
        self.refused(archive_bytes((member(), b''), (directory, b'')))

    def test_raw_header_difference_is_refused_even_when_parsed_metadata_matches(self):
        first = bytearray(encoded(member()))
        first[500:502] = b'AB'
        first[148:156] = b'        '
        first[148:156] = ('%06o' % sum(first)).encode() + bytes([0, 32])
        second = bytearray(first)
        second[500:502] = b'BA'
        self.assertEqual(tarfile.TarInfo.frombuf(first, 'utf-8', 'strict').get_info(),
                         tarfile.TarInfo.frombuf(second, 'utf-8', 'strict').get_info())
        self.refused(bytes(first) + bytes(second) + bytes(1024))

    def test_duplicate_flood_counts_records_not_unique_names(self):
        raw = archive_bytes(*[(member(), b'')] * 5)
        self.assertEqual(len(self.admit(raw, maximum=5)[1]), 5)
        self.refused(raw, maximum=4)

    def test_normalized_name_alias_cannot_replace_original_directory(self):
        self.refused(archive_bytes((member(), b''), (member(ROOT + '/./d'), b'')))
        self.refused(archive_bytes((member(), b''), (member(ROOT + '//d'), b'')))

    def test_traversal_and_absolute_or_foreign_root_remain_refused(self):
        for name in (ROOT + '/../escape', '/' + ROOT + '/d', 'other/d', ROOT + '/d\\escape'):
            with self.subTest(name=name):
                self.refused(archive_bytes((member(name), b'')))

    def test_nondirectory_ancestor_remains_refused(self):
        self.refused(archive_bytes((member(ROOT + '/d', tarfile.REGTYPE), b'x'),
                                  (member(ROOT + '/d/child', tarfile.REGTYPE), b'y')))

    def test_link_cycles_escaping_links_and_dangling_links_remain_refused(self):
        self.refused(archive_bytes((member(ROOT + '/a', tarfile.SYMTYPE, linkname='b'), b''),
                                  (member(ROOT + '/b', tarfile.SYMTYPE, linkname='a'), b'')))
        for target in ('../../escape', '/outside', 'missing'):
            with self.subTest(target=target):
                self.refused(archive_bytes((member(ROOT + '/a', tarfile.SYMTYPE, linkname=target), b'')))
        self.refused(archive_bytes((member(ROOT + '/a', tarfile.LNKTYPE, linkname='outside'), b'')))

    def test_regular_byte_cap_and_special_types_remain_refused(self):
        raw = archive_bytes((member(ROOT + '/file', tarfile.REGTYPE), b'ab'))
        tool = self.tool
        self.tool = dict(tool, max_expanded_regular_bytes=1)
        try:
            self.refused(raw)
        finally:
            self.tool = tool
        for kind in (tarfile.FIFOTYPE, tarfile.CHRTYPE, tarfile.BLKTYPE):
            with self.subTest(kind=kind):
                self.refused(archive_bytes((member(kind=kind), b'')))


if __name__ == '__main__':
    unittest.main()
