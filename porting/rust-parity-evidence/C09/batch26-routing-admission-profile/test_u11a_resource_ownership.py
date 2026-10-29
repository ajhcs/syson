"""Exercise actual cleanup/query definitions with fake systemd and owned fixtures."""
import ast
import copy
import hashlib
import json
import os
import re
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest


class OwnershipTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.source = Path(__file__).parent / 'run_u11a_resource_01.py'
        raw = cls.source.read_bytes()
        if hashlib.sha256(raw).hexdigest() != os.environ['SYSON_U11A_RESOURCE_SOURCE_SHA256']:
            raise ValueError('reviewed resource controller hash required')
        cls.definitions = ast.Module(body=[node for node in ast.parse(raw).body
            if isinstance(node, ast.FunctionDef) and node.name in ['scope_state', 'cleanup_scope']], type_ignores=[])
        cls.fixture_parent = Path(os.environ['SYSON_U11A_RESOURCE_TEST_TMP']).resolve()
        if not cls.fixture_parent.is_relative_to('/workspace') or not cls.fixture_parent.is_dir():
            raise ValueError('existing admitted Samsung fixtures required')

    def setUp(self):
        temporary = tempfile.TemporaryDirectory(dir=self.fixture_parent)
        self.addCleanup(temporary.cleanup)
        self.output = Path(temporary.name)
        self.unit = 'syson-u11a-profile-' + 'a' * 32 + '.scope'
        self.relative = '/user.slice/owned/' + self.unit
        self.group = self.output / 'groups' / self.relative.lstrip('/')
        self.group.mkdir(parents=True)
        (self.group / 'cgroup.events').write_text('populated 1\nfrozen 0\n')
        (self.group / 'cgroup.procs').write_text('123\n')
        self.calls = []
        self.states = []
        self.reply = SimpleNamespace(returncode=0, stdout=b'LoadState=not-found\n')
        self.globals = {'json': json, 're': re, 'Path': lambda value: self.output / 'groups'
                        if str(value) == '/sys/fs/cgroup' else Path(value),
                        'subprocess': SimpleNamespace(run=self.fake_run),
                        'time': SimpleNamespace(sleep=lambda seconds: None), 'env': {},
                        'outdir': self.output, 'unit': self.unit, 'cookie': 'c' * 32}
        exec(compile(self.definitions, str(self.source), 'exec'), self.globals)
        self.query = self.globals['scope_state']
        self.globals['scope_state'] = lambda: self.states.pop(0)
        self.cleanup = self.globals['cleanup_scope']
        self.loaded = {'LoadState': 'loaded', 'ActiveState': 'active',
                       'ControlGroup': self.relative, 'InvocationID': 'd' * 32}
        self.absent = {'LoadState': 'not-found', 'ActiveState': 'inactive', 'ControlGroup': ''}
        self.proof = {'unit': self.unit, 'cookie': 'c' * 32, 'control_group': self.relative,
                      'invocation_id': 'd' * 32, 'entry_pid': 123, 'entry_start_ticks': '456'}

    def fake_run(self, argv, **kwargs):
        self.calls.append((argv, kwargs))
        return self.reply

    def write_proof(self, proof=None):
        (self.output / 'ownership.json').write_text(json.dumps(self.proof if proof is None else proof))

    def test_absent_is_safe_without_ownership_or_kill(self):
        self.states = [copy.deepcopy(self.absent)]
        self.assertTrue(self.cleanup()['confirmed_empty_or_absent'])
        self.assertEqual(self.calls, [])

    def test_loaded_without_attestation_is_preserved(self):
        self.states = [copy.deepcopy(self.loaded)]
        with self.assertRaisesRegex(RuntimeError, 'ownership is unproven'):
            self.cleanup()
        self.assertEqual(self.calls, [])

    def test_each_attestation_identity_mismatch_preserves_scope(self):
        for key in ['unit', 'cookie', 'control_group', 'invocation_id']:
            with self.subTest(field=key):
                proof = dict(self.proof)
                proof[key] = 'wrong'
                self.write_proof(proof)
                self.states = [copy.deepcopy(self.loaded)]
                with self.assertRaisesRegex(RuntimeError, 'ownership differs'):
                    self.cleanup()
                self.assertEqual(self.calls, [])

    def test_reused_unit_with_later_invocation_is_preserved(self):
        self.write_proof()
        current = dict(self.loaded, InvocationID='e' * 32)
        self.states = [current]
        with self.assertRaisesRegex(RuntimeError, 'ownership differs'):
            self.cleanup()
        self.assertEqual(self.calls, [])

    def test_matching_live_scope_kills_only_exact_owned_unit(self):
        self.write_proof()
        self.states = [copy.deepcopy(self.loaded), copy.deepcopy(self.absent)]
        self.assertTrue(self.cleanup()['confirmed_empty_or_absent'])
        self.assertEqual(len(self.calls), 1)
        self.assertEqual(self.calls[0][0], ['/usr/bin/systemctl', '--user', 'kill',
                         '--kill-whom=all', '--signal=SIGKILL', self.unit])

    def test_matching_empty_scope_needs_no_kill(self):
        self.write_proof()
        (self.group / 'cgroup.events').write_text('populated 0\nfrozen 0\n')
        (self.group / 'cgroup.procs').write_text('')
        self.states = [copy.deepcopy(self.loaded)]
        self.assertTrue(self.cleanup()['confirmed_empty_or_absent'])
        self.assertEqual(self.calls, [])

    def test_matching_proof_with_wrong_cgroup_path_still_refuses(self):
        proof = dict(self.proof, control_group='/foreign.scope')
        self.write_proof(proof)
        self.states = [dict(self.loaded, ControlGroup='/foreign.scope')]
        with self.assertRaisesRegex(RuntimeError, 'unexpected owned cgroup'):
            self.cleanup()
        self.assertEqual(self.calls, [])

    def test_matching_empty_cgroup_is_preserved_without_kill(self):
        self.write_proof(dict(self.proof, control_group=''))
        self.states = [dict(self.loaded, ControlGroup=''), copy.deepcopy(self.absent)]
        with self.assertRaisesRegex(RuntimeError, 'unexpected owned cgroup'):
            self.cleanup()
        self.assertEqual(self.calls, [])

    def test_matching_invalid_invocation_is_preserved_without_kill(self):
        for invocation in ['0' * 32, 'invalid', 'a' * 31]:
            with self.subTest(invocation=invocation):
                self.calls.clear()
                self.write_proof(dict(self.proof, invocation_id=invocation))
                self.states = [dict(self.loaded, InvocationID=invocation), copy.deepcopy(self.absent)]
                with self.assertRaisesRegex(RuntimeError, 'invocation is unobservable'):
                    self.cleanup()
                self.assertEqual(self.calls, [])

    def test_scope_query_binds_invocation_and_refuses_unobservable_state(self):
        self.assertEqual(self.query()['LoadState'], 'not-found')
        self.assertIn('--property=InvocationID', self.calls[-1][0])
        self.assertEqual(self.calls[-1][0][3], self.unit)
        self.reply = SimpleNamespace(returncode=1, stdout=b'')
        with self.assertRaisesRegex(RuntimeError, 'unobservable'):
            self.query()


if __name__ == '__main__':
    unittest.main()
