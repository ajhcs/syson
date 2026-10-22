# SPDX-License-Identifier: EPL-2.0
"""Synthetic evidence format tests. Generated reports are NOT product run evidence."""
import copy
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / 'scripts/check_behavior_ledger.py'
spec = importlib.util.spec_from_file_location('gate', SCRIPT)
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)
SCHEMA = json.loads((SCRIPT.parents[3] / 'porting/ledger-schema/behavior-ledger.schema.json').read_text())


class LedgerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory(dir=os.environ['SYSON_BUILD_DIR'])
        cls.root = Path(cls.temp.name)
        cls.git('init', '-q')
        (cls.root / 'source.txt').write_text('upstream source\nimplementation projection\n')
        (cls.root / 'fixture.json').write_text(json.dumps({'synthetic': True}))
        cls.git('add', 'source.txt', 'fixture.json')
        cls.git('-c', 'user.name=Synthetic Test', '-c', 'user.email=test@example.invalid',
                '-c', 'core.hooksPath=/dev/null', '-c', 'commit.gpgsign=false', 'commit', '-qm', 'synthetic fixture')
        cls.revision = cls.git('rev-parse', 'HEAD').decode().strip()
        cls.source_hash = hashlib.sha256((cls.root / 'source.txt').read_bytes()).hexdigest()

    @classmethod
    def git(cls, *args):
        return subprocess.check_output(['git', '-C', str(cls.root), *args], stderr=subprocess.DEVNULL)

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def artifact(self, path, value):
        raw = value.encode() if isinstance(value, str) else json.dumps(value).encode()
        (self.root / path).write_bytes(raw)
        return {'path': path, 'sha256': hashlib.sha256(raw).hexdigest()}

    def setUp(self):
        fixture = self.artifact('fixture.json', {'synthetic': True})
        transcript = self.artifact('transcript.txt', 'Synthetic transcript for format validation only.')
        self.cases = [f'SYSON-CASE-{n:06}' for n in range(1, 4)]
        self.run_reports = {}
        self.run_refs = {}
        for side in ('upstream', 'rust'):
            report = {'behavior_id': 'SYSON-BEH-000001', 'side': side, 'revision': self.revision,
                      'executed': True, 'command': ['synthetic-format-fixture'],
                      'started_at': '2026-10-22T00:00:00Z', 'finished_at': '2026-10-22T00:00:01Z',
                      'exit_code': 0, 'result': 'passed', 'case_ids': self.cases,
                      'fixture_hashes': [fixture['sha256']], 'transcript': transcript,
                      'harness_revision': self.revision,
                      'harness_files': [{'path': 'source.txt', 'sha256': self.source_hash}, fixture]}
            self.run_reports[side] = report
            self.run_refs[side] = [self.artifact(f'{side}.json', report)]
        review_report = {'behavior_id': 'SYSON-BEH-000001', 'reviewer': 'independent-test-reviewer',
                         'independent': True, 'implementation_revision': self.revision,
                         'disposition': 'accepted', 'case_ids': self.cases,
                         'notes': 'Synthetic format validation only; no product evidence.'}
        row = {'id': 'SYSON-BEH-000001', 'title': 'Synthetic format test', 'required': True,
               'status': 'verified', 'owner': 'test', 'dependency_ids': ['syson'],
               'source_evidence': [{'dependency_id': 'syson', 'revision': self.revision,
                                    'path': 'source.txt', 'sha256': self.source_hash,
                                    'line_start': 1, 'line_end': 2}],
               'expected': {'state': 'synthetic', 'result': 'synthetic'},
               'implementation': {'revision': self.revision, 'authors': ['test-author'],
                                  'files': [{'path': 'source.txt', 'sha256': self.source_hash}]},
               'cases': {group: [{'id': case_id, 'description': 'synthetic case', 'fixtures': [fixture]}]
                         for group, case_id in zip(('positive', 'negative', 'failure'), self.cases)},
               'runs': self.run_refs, 'review': {'reviewer': 'independent-test-reviewer',
                                               'independent': True, 'disposition': 'accepted',
                                               'artifact': self.artifact('review.json', review_report)},
               'disposition': 'Synthetic fixture, not product evidence.'}
        self.ledger = {'schema_version': 1, 'upstream_revision': self.revision,
                       'scope': {'description': 'synthetic test', 'inventory_closed': False,
                                 'dependency_closure_closed': False, 'closure_evidence': []},
                       'dependencies': [{'id': 'syson', 'repository': 'https://example.invalid/test',
                                         'revision': self.revision, 'owner': 'test', 'status': 'pinned',
                                         'disposition': 'synthetic'}], 'behaviors': [row]}
        self.row = row

    def check(self, complete=False):
        return gate.validate(self.root, self.ledger, SCHEMA, complete=complete)

    def reject(self, complete=False):
        with self.assertRaises(gate.Invalid):
            self.check(complete)

    def change_run(self, key, value, side='upstream'):
        self.run_reports[side][key] = value
        self.row['runs'][side] = [self.artifact(f'{side}.json', self.run_reports[side])]

    def test_synthetic_verified_shape(self):
        self.assertEqual(self.check()['verified'], 1)

    def test_pending_shape_does_not_assert_verification(self):
        self.row.update(status='unrun', implementation=None, review=None)
        self.row['runs'] = {'upstream': [], 'rust': []}
        self.assertEqual(self.check()['verified'], 0)

    def test_unknown_property(self):
        self.row['pretend_verified'] = True
        self.reject()

    def test_duplicate_behavior(self):
        self.ledger['behaviors'].append(copy.deepcopy(self.row))
        self.reject()

    def test_duplicate_json_keys(self):
        with self.assertRaises(gate.Invalid):
            gate.strict_json('{"x":1,"x":2}')

    def test_non_json_nan(self):
        with self.assertRaises(gate.Invalid):
            gate.strict_json('{"x":NaN}')

    def test_wrong_source_hash(self):
        self.row['source_evidence'][0]['sha256'] = '0' * 64
        self.reject()

    def test_source_revision_differs(self):
        self.row['source_evidence'][0]['revision'] = '0' * 40
        self.reject()

    def test_source_lines_invalid(self):
        self.row['source_evidence'][0]['line_end'] = 999
        self.reject()

    def test_missing_source(self):
        self.row['source_evidence'] = []
        self.reject()

    def test_wrong_implementation_hash(self):
        self.row['implementation']['files'][0]['sha256'] = '0' * 64
        self.reject()

    def test_missing_implementation(self):
        self.row['implementation'] = None
        self.reject()

    def test_missing_owner(self):
        del self.row['owner']
        self.reject()

    def test_unresolved_dependency(self):
        self.ledger['dependencies'][0]['status'] = 'unresolved'
        self.reject()

    def test_no_positive_negative_or_failure(self):
        for group in ('positive', 'negative', 'failure'):
            saved = self.row['cases'][group]
            self.row['cases'][group] = []
            self.reject()
            self.row['cases'][group] = saved

    def test_missing_fixture(self):
        self.row['cases']['negative'][0]['fixtures'][0]['path'] = 'absent.json'
        self.reject()

    def test_fixture_without_hash(self):
        del self.row['cases']['failure'][0]['fixtures'][0]['sha256']
        self.reject()

    def test_artifact_path_escape(self):
        self.row['runs']['upstream'][0]['path'] = '../outside.json'
        self.reject()

    def test_symlink_escape(self):
        link = self.root / 'outside-link'
        if link.is_symlink():
            link.unlink()
        link.symlink_to(self.root.parent)
        self.row['runs']['upstream'][0]['path'] = 'outside-link/unknown'
        self.reject()

    def test_empty_upstream_and_rust_runs(self):
        for side in ('upstream', 'rust'):
            saved = self.row['runs'][side]
            self.row['runs'][side] = []
            self.reject()
            self.row['runs'][side] = saved

    def test_tampered_run_file(self):
        (self.root / 'upstream.json').write_text('{}')
        self.reject()

    def test_unexecuted_run(self):
        self.change_run('executed', False)
        self.reject()

    def test_failed_run(self):
        self.change_run('result', 'failed')
        self.reject()

    def test_nonzero_exit(self):
        self.change_run('exit_code', 1)
        self.reject()

    def test_wrong_run_revision(self):
        self.change_run('revision', '0' * 40)
        self.reject()

    def test_missing_run_case(self):
        self.change_run('case_ids', self.cases[:-1])
        self.reject()

    def test_unknown_run_case(self):
        self.change_run('case_ids', ['SYSON-CASE-999999'])
        self.reject()

    def test_missing_fixture_coverage(self):
        self.change_run('fixture_hashes', ['0' * 64])
        self.reject()

    def test_missing_review(self):
        self.row['review'] = None
        self.reject()

    def test_reviewer_authored_code(self):
        self.row['implementation']['authors'].append('independent-test-reviewer')
        self.reject()

    def test_review_not_independent(self):
        self.row['review']['independent'] = False
        self.reject()

    def test_review_disposition_not_accepted(self):
        self.row['review']['disposition'] = 'changes_requested'
        self.reject()

    def test_review_report_wrong_revision(self):
        report = json.loads((self.root / 'review.json').read_text())
        report['implementation_revision'] = '0' * 40
        self.row['review']['artifact'] = self.artifact('review.json', report)
        self.reject()

    def test_open_inventory_completion(self):
        self.reject(complete=True)

    def test_closure_needs_evidence(self):
        self.ledger['scope'].update(inventory_closed=True, dependency_closure_closed=True)
        self.reject(complete=True)

    def test_pinned_dependency_without_revision(self):
        self.ledger['dependencies'].append({'id': 'external', 'repository': 'https://example.invalid/external', 'revision': None, 'owner': 'test', 'status': 'pinned', 'disposition': 'synthetic'})
        self.reject()

    def test_invalid_calendar_date(self):
        self.change_run('started_at', '2026-99-22T00:00:00Z')
        self.reject()

    def test_reversed_run_dates(self):
        self.change_run('started_at', '2026-10-23T00:00:00Z')
        self.reject()

    def test_empty_run_transcript(self):
        self.change_run('transcript', self.artifact('empty.txt', ''))
        self.reject()

    def test_review_report_missing_case(self):
        report = json.loads((self.root / 'review.json').read_text())
        report['case_ids'] = self.cases[:-1]
        self.row['review']['artifact'] = self.artifact('review.json', report)
        self.reject()

    def test_wrong_harness_hash(self):
        self.change_run('harness_files', [{'path': 'source.txt', 'sha256': '0' * 64}])
        self.reject()

    def test_fixture_not_pinned_in_harness(self):
        self.change_run('harness_files', [{'path': 'source.txt', 'sha256': self.source_hash}])
        self.reject()

    def test_unrun_row_still_validates_native_report(self):
        self.row['status'] = 'unrun'
        self.row['runs']['upstream'] = []
        self.change_run('revision', '0' * 40, side='rust')
        self.reject()

    def test_unrun_row_can_retain_honest_failed_native_report(self):
        self.row['status'] = 'unrun'
        self.row['runs']['upstream'] = []
        self.change_run('result', 'failed', side='rust')
        self.change_run('exit_code', 1, side='rust')
        self.assertEqual(self.check()['verified'], 0)

    def test_unrun_row_rejects_inconsistent_native_result(self):
        self.row['status'] = 'unrun'
        self.row['runs']['upstream'] = []
        self.change_run('exit_code', 1, side='rust')
        self.reject()

    def closed(self):
        self.ledger['scope'].update(inventory_closed=True, dependency_closure_closed=True,
                                    closure_evidence=[self.artifact('closure.json', {'synthetic': True})])

    def test_closed_synthetic_completion(self):
        self.closed()
        self.assertTrue(self.check(complete=True)['complete'])

    def test_all_incomplete_required_statuses_reject_completion(self):
        self.closed()
        for status in ('unassessed', 'pending', 'implemented_bounded', 'blocked', 'incompatible', 'unrun'):
            self.row['status'] = status
            self.reject(complete=True)


if __name__ == '__main__':
    unittest.main()
