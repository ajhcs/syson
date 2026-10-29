"""Source admission controls only: no retained tool executable is ever launched."""
import copy
import hashlib
import json
import os
from pathlib import Path
import stat
import subprocess
import sys
import types
import unittest
from unittest.mock import Mock, patch

BASE = Path(__file__).resolve().parents[1]
RUNNER_SHA256 = 'd2a539db7b4aabdda93a1702c2ac35b0659ddc44813e9a7402cc70feecdc4682'
FIXTURES = BASE.parent / ('c01-version-controls-' + ('optimized' if sys.flags.optimize else 'normal') + '-01')


def load_subject():
    path = BASE / 'check_versions.py'
    source = path.read_bytes()
    if hashlib.sha256(source).hexdigest() != RUNNER_SHA256:
        raise RuntimeError('reviewed version source changed')
    module = types.ModuleType('version_controls_subject')
    module.__file__ = str(path)
    exec(compile(source, str(path), 'exec'), module.__dict__)
    return module


class VersionAdmissionControls(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.subject = load_subject()
        FIXTURES.mkdir(mode=0o700)
        cls.proposal = json.loads((BASE / 'proposal.json').read_bytes())

    def setUp(self):
        self.fixture = FIXTURES / self._testMethodName
        self.fixture.mkdir(mode=0o700)
        self.root = self.fixture / 'tool'
        self.root.mkdir(mode=0o700)
        (self.root / 'bin').mkdir(mode=0o700)
        content = b'bounded fixture, never executable'
        (self.root / 'bin/tool').write_bytes(content)
        (self.root / 'bin/tool').chmod(0o600)
        (self.root / 'shortcut').symlink_to('bin/tool')
        self.rows = [
            {'path': 'bin', 'kind': 'directory', 'mode': 0o700},
            {'path': 'bin/tool', 'kind': 'file', 'mode': 0o600, 'bytes': len(content),
             'sha256': hashlib.sha256(content).hexdigest()},
            {'path': 'shortcut', 'kind': 'symlink', 'mode': 0o777, 'target': 'bin/tool',
             'target_exists': True, 'sha256': hashlib.sha256(b'bin/tool').hexdigest()},
        ]
        self.manifest = self.fixture / 'manifest.json'
        self.spec = {'id': 'fixture', 'root': str(self.root), 'manifest': str(self.manifest), 'entries': 3}
        self.write_manifest()

    def write_manifest(self):
        self.manifest.write_text(json.dumps(self.rows))

    def test_exact_tree_and_proposal_pairing(self):
        result = self.subject.verify_tree(self.spec)
        self.assertEqual(result['entries'], 3)
        self.assertEqual(result['regular_bytes'], self.rows[1]['bytes'])
        self.assertEqual(self.subject.PROPOSAL_SHA256,
                         hashlib.sha256((BASE / 'proposal.json').read_bytes()).hexdigest())
        self.assertEqual([b['entries'] for b in self.proposal['bundles']], [541, 109, 5887])

    def test_same_size_file_corruption_refuses(self):
        path = self.root / 'bin/tool'
        raw = bytearray(path.read_bytes())
        raw[0] ^= 1
        path.write_bytes(raw)
        with self.assertRaises(self.subject.Refusal):
            self.subject.verify_tree(self.spec)

    def test_extra_missing_and_type_changed_entries_refuse(self):
        extra = self.root / 'unexpected'
        extra.write_bytes(b'extra')
        with self.assertRaises(self.subject.Refusal):
            self.subject.verify_tree(self.spec)
        extra.unlink()
        target = self.root / 'bin/tool'
        target.unlink()
        with self.assertRaises(FileNotFoundError):
            self.subject.verify_tree(self.spec)
        target.mkdir(mode=0o600)
        with self.assertRaises(self.subject.Refusal):
            self.subject.verify_tree(self.spec)

    def test_mode_change_refuses(self):
        (self.root / 'bin/tool').chmod(0o700)
        with self.assertRaises(self.subject.Refusal):
            self.subject.verify_tree(self.spec)

    def test_symlink_escape_refuses_even_with_coherent_target_metadata(self):
        (self.fixture / 'outside').write_bytes(b'outside')
        link = self.root / 'shortcut'
        link.unlink()
        link.symlink_to('../outside')
        self.rows[2].update(target='../outside', sha256=hashlib.sha256(b'../outside').hexdigest())
        self.write_manifest()
        with self.assertRaises(self.subject.Refusal):
            self.subject.verify_tree(self.spec)

    def test_duplicate_traversal_and_nontext_manifest_paths_refuse(self):
        original = copy.deepcopy(self.rows)
        for replacement in ['../outside', '/outside', './bin', 3]:
            with self.subTest(path=replacement):
                self.rows = copy.deepcopy(original)
                self.rows[0]['path'] = replacement
                self.write_manifest()
                with self.assertRaises(self.subject.Refusal):
                    self.subject.verify_tree(self.spec)
        self.rows = original + [original[0]]
        self.spec['entries'] = 4
        self.write_manifest()
        with self.assertRaises(self.subject.Refusal):
            self.subject.verify_tree(self.spec)

    def test_manifest_count_and_unknown_kind_refuse(self):
        with self.assertRaises(self.subject.Refusal):
            self.subject.verify_tree(dict(self.spec, entries=4))
        self.rows[1]['kind'] = 'device'
        self.write_manifest()
        with self.assertRaises(self.subject.Refusal):
            self.subject.verify_tree(self.spec)

    def test_bound_input_drift_refuses_before_any_launch(self):
        bindings = {str(self.manifest): hashlib.sha256(self.manifest.read_bytes()).hexdigest()}
        self.subject.bound_inputs({'bindings': bindings})
        self.manifest.write_text('changed')
        with self.assertRaises(self.subject.Refusal):
            self.subject.bound_inputs({'bindings': bindings})

    def test_parent_injection_is_excluded_and_home_is_preserved(self):
        injected = {name: 'untrusted' for name in ['NODE_OPTIONS', 'NODE_PATH', 'JAVA_TOOL_OPTIONS',
                    '_JAVA_OPTIONS', 'JDK_JAVA_OPTIONS', 'CLASSPATH', 'LD_PRELOAD', 'GH_TOKEN',
                    'MAVEN_ARGS', 'MAVEN_CONFIG', 'npm_config_userconfig']}
        with patch.dict(os.environ, injected):
            environment, options = self.subject.clean_environment(self.proposal, self.fixture)
        self.assertEqual(environment['HOME'], os.environ['HOME'])
        self.assertTrue(set(injected).isdisjoint(environment))
        self.assertEqual(environment['MAVEN_SKIP_RC'], '1')
        self.assertEqual(environment['NODE_DISABLE_COMPILE_CACHE'], '1')
        self.assertIn('-XX:-UsePerfData', options)
        self.assertIn(str(self.fixture / 'java-home'), environment['MAVEN_OPTS'])

    def test_only_five_fixed_version_handlers_exist(self):
        proposal = dict(self.proposal, arbitrary_commands=['install', 'run', 'package'])
        commands = self.subject.commands(proposal, self.fixture)
        self.assertEqual([name for name, _ in commands], ['java', 'javac', 'maven', 'node', 'npm'])
        for name, argv in commands:
            self.assertIn('-version' if name in ['java', 'javac'] else '--version', argv)
            self.assertTrue(set(['install', 'run', 'package']).isdisjoint(argv))
        npm = commands[-1][1]
        self.assertIn('--prefix=' + str(self.fixture / 'cwd'), npm)
        self.assertIn('--ignore-scripts', npm)
        self.assertIn('--update-notifier=false', npm)

    def test_version_mismatches_and_prefix_collisions_refuse(self):
        for name, output in [('java', b'NOT Temurin-21.0.12.1+1'), ('javac', b'javac 21.0.12.11'),
                             ('maven', b'Apache Maven 3.9.160\nJava version: 21.0.12.1,'),
                             ('node', b'v24.20.01'), ('npm', b'11.19.00')]:
            with self.subTest(name=name):
                with self.assertRaises(self.subject.Refusal):
                    self.subject.check_version(name, output, b'')
        self.subject.check_version('javac', b'javac 21.0.12.1\n', b'')
        self.subject.check_version('node', b'v24.20.0\n', b'')
        self.subject.check_version('npm', b'11.19.0\n', b'')

    def test_maven_override_and_descriptor_closure_without_launching(self):
        environment, _ = self.subject.clean_environment(self.proposal, self.fixture)
        child = Mock()
        child.wait.return_value = child.poll.return_value = 0
        def fake_launch(argv, **kwargs):
            self.assertEqual(kwargs['env']['JDK_JAVA_OPTIONS'], environment['MAVEN_OPTS'])
            self.assertTrue(kwargs['close_fds'])
            self.assertTrue(kwargs['start_new_session'])
            self.assertEqual(kwargs['stdin'], subprocess.DEVNULL)
            kwargs['stdout'].write(b'Apache Maven 3.9.16\nJava version: 21.0.12.1, vendor: Eclipse Adoptium\n')
            return child
        with patch.object(self.subject.subprocess, 'Popen', side_effect=fake_launch):
            with patch.object(self.subject, 'stop_group', return_value=False):
                result = self.subject.run_one('maven', ['not-executed'], environment, self.fixture)
        self.assertEqual(result['exit'], 0)
        self.assertEqual(result['environment']['JDK_JAVA_OPTIONS'], environment['MAVEN_OPTS'])
        self.assertNotIn('JDK_JAVA_OPTIONS', environment)

    def test_timeout_kills_and_reaps_only_mock_owned_group(self):
        child = Mock(pid=987654321)
        child.wait.side_effect = [subprocess.TimeoutExpired(['not-executed'], 20), 0]
        child.poll.return_value = None
        with patch.object(self.subject.subprocess, 'Popen', return_value=child):
            with patch.object(self.subject, 'stop_group') as stop:
                with self.assertRaises(subprocess.TimeoutExpired):
                    self.subject.run_one('node', ['not-executed'], {}, self.fixture)
        stop.assert_called_once_with(987654321)
        self.assertEqual(child.wait.call_count, 2)


if __name__ == '__main__':
    unittest.main()
