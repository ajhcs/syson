"""Whole .12 restoration and coherent corruption controls on the full797 baseline."""
import copy
import json
import unittest
from unittest import mock
from test_derive import D, G, Fixture


class GuardTests(Fixture, unittest.TestCase):
    def checked(self, contract=None, plan=None, read=None):
        contract = self.contract if contract is None else contract
        plan = self.plan if plan is None else plan
        with mock.patch.object(G, 'POLICY_SHA256', D.digest(contract['identity_partition_splits'])):
            return G.checked(contract, plan, self.files.__getitem__ if read is None else read, *self.args)

    def test_full_restoration_before_old_guard_and_no_early_parent_completion(self):
        policy, contract, plan = self.checked()
        self.assertEqual(contract, json.loads(self.files['completion-contract.json']))
        self.assertEqual(plan, self.old)
        self.assertEqual(policy, self.contract['identity_partition_splits'])
        parent = next(u for u in self.plan['units'] if u['id'] == 'M06a2')
        self.assertEqual(parent['acceptance_children'], D.IDS)
        self.assertTrue(set(D.IDS) <= set(parent['implementation_ready_dependencies']))
        for uid in ['C04', 'C05', 'C06', 'C07', 'C08', 'C09']:
            now = next(u for u in self.plan['units'] if u['id'] == uid)
            self.assertTrue(set(D.IDS) <= set(now['expanded_implementation_inputs']))

    def test_default_pending_and_unknown_version_refuse(self):
        with self.assertRaisesRegex(ValueError, 'policy pending'):
            G.checked({}, {}, None, *self.args)
        contract = copy.deepcopy(self.contract)
        contract['version'] = '1.0.13'
        with self.assertRaisesRegex(ValueError, 'partition version'):
            self.checked(contract=contract)

    def test_missing_duplicate_and_foreign_unit_ids_refuse(self):
        for kind in ['missing', 'duplicate', 'foreign']:
            plan = copy.deepcopy(self.plan)
            if kind == 'missing':
                plan['units'].pop()
            elif kind == 'duplicate':
                plan['units'][-1] = plan['units'][0]
            else:
                next(u for u in plan['units'] if u['id'] == 'C99')['id'] = 'C98'
                plan['units'].sort(key=lambda u: u['id'])
            with self.subTest(kind=kind), self.assertRaises(ValueError):
                self.checked(plan=plan)

    def test_coherently_resealed_parent_weakening_refuses(self):
        contract, plan = copy.deepcopy(self.contract), copy.deepcopy(self.plan)
        parent = next(u for u in plan['units'] if u['id'] == 'M06a2')
        parent['acceptance_children'] = ['M06a2a']
        changes = contract['identity_partition_splits']['record_updates']['M06a2']
        changes['acceptance_children']['after'] = ['M06a2a']
        with self.assertRaisesRegex(ValueError, 'exact complete current partition'):
            self.checked(contract, plan)

    def test_coherent_old_body_rewrite_cannot_be_laundered_as_overlay(self):
        contract, plan = copy.deepcopy(self.contract), copy.deepcopy(self.plan)
        unit = next(u for u in plan['units'] if u['id'] == 'M06a1')
        before = copy.deepcopy(unit['tasks'])
        unit['tasks'] = ['weakened original obligation']
        contract['identity_partition_splits']['record_updates']['M06a1'] = {
            'tasks': {'before': {'present': True, 'value': before}, 'after': unit['tasks']}}
        with self.assertRaisesRegex(ValueError, 'exact complete current partition'):
            self.checked(contract, plan)

    def test_whole_child_budget_case_and_path_tampering_refuse(self):
        for field in ['budget', 'tasks', 'paths']:
            plan = copy.deepcopy(self.plan)
            child = next(u for u in plan['units'] if u['id'] == 'M06a2b')
            if field == 'budget':
                child[field]['production_lines_max'] = 401
            else:
                child[field].pop()
            with self.subTest(field=field), self.assertRaisesRegex(
                    ValueError, 'exact complete current partition'):
                self.checked(plan=plan)

    def test_all_old_reader_held_generator_and_schema_bytes_remain_bound(self):
        for name in ['teams.json', 'execution.json', 'tools/build_plan.py', 'acceptance-packet.schema.json']:
            def read(role):
                return self.files[role] + (b'\n' if role == name else b'')
            with self.subTest(name=name), self.assertRaises(ValueError):
                self.checked(read=read)

    def test_extra_current_metadata_graph_drop_and_cyclic_object_refuse(self):
        for kind in ['extra', 'graph', 'cycle']:
            plan = copy.deepcopy(self.plan)
            if kind == 'extra':
                plan['unreviewed'] = True
            elif kind == 'graph':
                plan['readiness_graph']['accepted:M06a2'] = ['implementation:M06a2']
            else:
                plan['cycle'] = plan
            with self.subTest(kind=kind), self.assertRaises(ValueError):
                self.checked(plan=plan)
