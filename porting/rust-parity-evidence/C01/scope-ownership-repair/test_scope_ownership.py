"""Fake systemd fault controls; this module never calls a real systemctl."""
from pathlib import Path
import importlib.util
import json
import tempfile
from types import SimpleNamespace
import unittest
from unittest import mock

spec = importlib.util.spec_from_file_location('ownership', Path(__file__).with_name('scope_ownership.py'))
ownership = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ownership)
UNIT = 'syson-test-' + 'a' * 32 + '.scope'
COOKIE = 'b' * 32
GROUP = '/user.slice/' + UNIT
INVOCATION = 'c' * 32
ABSENT = {'LoadState': 'not-found'}
LOADED = {'LoadState': 'loaded', 'ControlGroup': GROUP, 'InvocationID': INVOCATION}
BAD_GROUPS = ['', 'relative/' + UNIT, '//user.slice/' + UNIT, '/user.slice/./' + UNIT,
              '/user.slice/../' + UNIT, '/user.slice//' + UNIT, '/other.scope']
BAD_INVOCATIONS = ['', '0' * 32, 'C' * 32, 'c' * 31, 'c' * 33, 'z' * 32]


class FakeSystemd:
    def __init__(self, states):
        self.states, self.kills = list(states), []

    def __call__(self, argv, **unused):
        if argv[2] == 'kill':
            self.kills.append(argv)
            return SimpleNamespace(returncode=0)
        if argv[2:4] != ['show', UNIT]:
            raise AssertionError('unexpected fake systemd query identity')
        current = self.states.pop(0) if len(self.states) > 1 else self.states[0]
        raw = ''.join(key + '=' + value + '\n' for key, value in current.items()).encode()
        return SimpleNamespace(returncode=0, stdout=raw)


class ScopeOwnershipTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(dir=Path(__file__).parent)
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.path = self.root / 'proof.json'
        self.proof = {'unit': UNIT, 'cookie': COOKIE, 'control_group': GROUP,
                      'invocation_id': INVOCATION, 'entry_pid': 123, 'entry_start_ticks': '456'}

    def save(self, **changes):
        self.path.write_text(json.dumps(dict(self.proof, **changes)))

    def cleanup(self, fake):
        return ownership.cleanup(UNIT, COOKIE, self.path, {}, run=fake, root=self.root, pause=lambda _: None)

    def refused(self, fake):
        with self.assertRaises(RuntimeError):
            self.cleanup(fake)
        self.assertEqual(fake.kills, [])

    def test_absent_without_proof_never_kills(self):
        fake = FakeSystemd([ABSENT])
        self.assertTrue(self.cleanup(fake)['confirmed_empty_or_absent'])
        self.assertEqual(fake.kills, [])

    def test_loaded_without_proof_preserves_foreign_scope(self):
        self.refused(FakeSystemd([LOADED]))

    def test_wrong_cookie_group_unit_and_stale_invocation_preserve(self):
        for changes in [{'cookie': 'd' * 32}, {'control_group': '/elsewhere/' + UNIT},
                        {'unit': 'other.scope'}, {'invocation_id': 'e' * 32}]:
            with self.subTest(changes=changes):
                self.save(**changes)
                self.refused(FakeSystemd([LOADED]))

    def test_reused_unit_after_collection_is_preserved(self):
        self.save()
        fake = FakeSystemd([dict(LOADED, InvocationID='f' * 32)])
        self.refused(fake)

    def test_owned_populated_scope_is_killed_and_absence_confirmed(self):
        self.save()
        fake = FakeSystemd([LOADED, ABSENT])
        self.assertTrue(self.cleanup(fake)['confirmed_empty_or_absent'])
        self.assertEqual(len(fake.kills), 1)
        self.assertEqual(fake.kills[0][-1], UNIT)
        self.assertIn('--kill-whom=all', fake.kills[0])
        self.assertIn('--signal=SIGKILL', fake.kills[0])

    def test_replacement_during_cleanup_is_never_killed_again(self):
        self.save()
        fake = FakeSystemd([LOADED, dict(LOADED, InvocationID='f' * 32)])
        with self.assertRaises(RuntimeError):
            self.cleanup(fake)
        self.assertEqual(len(fake.kills), 1)

    def test_owned_empty_scope_needs_no_kill(self):
        self.save()
        directory = self.root / GROUP.lstrip('/')
        directory.mkdir(parents=True)
        (directory / 'cgroup.events').write_text('populated 0\n')
        (directory / 'cgroup.procs').write_text('')
        fake = FakeSystemd([LOADED])
        self.assertTrue(self.cleanup(fake)['confirmed_empty_or_absent'])
        self.assertEqual(fake.kills, [])

    def test_unobservable_systemd_state_preserves_scope(self):
        self.save()
        self.refused(FakeSystemd([{'LoadState': 'error'}]))

    def test_entry_attests_actual_invocation_and_start_ticks_before_body(self):
        proc = {'/proc/self/cgroup': '0::' + GROUP + '\n',
                '/proc/self/stat': '123 (name with spaces) S ' + '0 ' * 18 + '456'}
        with mock.patch.object(ownership, 'state', return_value=LOADED), mock.patch.object(
                ownership.Path, 'read_text', autospec=True, side_effect=lambda path: proc[str(path)]):
            proof = ownership.attest(UNIT, COOKIE, 'syson-test-', self.path, {})
        self.assertEqual(proof['control_group'], GROUP)
        self.assertEqual(proof['invocation_id'], INVOCATION)
        self.assertEqual(proof['entry_start_ticks'], '456')
        self.assertEqual(json.loads(self.path.read_bytes()), proof)

    def test_entry_rejects_another_invocation_without_writing_proof(self):
        with mock.patch.object(ownership, 'state', return_value=dict(LOADED, ControlGroup='/foreign')), mock.patch.object(
                ownership.Path, 'read_text', return_value='0::' + GROUP + '\n'):
            with self.assertRaises(RuntimeError):
                ownership.attest(UNIT, COOKIE, 'syson-test-', self.path, {})
        self.assertFalse(self.path.exists())

    def test_matching_noncanonical_groups_raise_and_never_kill(self):
        for group in BAD_GROUPS:
            with self.subTest(group=group):
                self.save(control_group=group)
                self.refused(FakeSystemd([dict(LOADED, ControlGroup=group)]))

    def test_matching_invalid_invocation_ids_raise_and_never_kill(self):
        for invocation in BAD_INVOCATIONS:
            with self.subTest(invocation=invocation):
                self.save(invocation_id=invocation)
                self.refused(FakeSystemd([dict(LOADED, InvocationID=invocation)]))

    def test_entry_refuses_matching_invalid_identity_without_proof(self):
        cases = [(group, INVOCATION) for group in BAD_GROUPS] + [(GROUP, value) for value in BAD_INVOCATIONS]
        for group, invocation in cases:
            with self.subTest(group=group, invocation=invocation), mock.patch.object(
                    ownership, 'state', return_value=dict(LOADED, ControlGroup=group, InvocationID=invocation)), mock.patch.object(
                    ownership.Path, 'read_text', return_value='0::' + group + '\n'):
                with self.assertRaises(RuntimeError):
                    ownership.attest(UNIT, COOKIE, 'syson-test-', self.path, {})
                self.assertFalse(self.path.exists())

    def test_new_attempt_names_and_cookies_are_distinct(self):
        first, second = ownership.new_attempt('syson-test-'), ownership.new_attempt('syson-test-')
        self.assertNotEqual(first[0], second[0])
        self.assertNotEqual(first[1], second[1])
        ownership.validate(*first, 'syson-test-')
        ownership.validate(*second, 'syson-test-')


if __name__ == '__main__':
    unittest.main()
