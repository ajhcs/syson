"""Current complete plan controls; no subject/compiler/native processes."""
import copy
import json
from pathlib import Path
import unittest
from unittest import mock
import preparation_splits as P
import storage_splits as S

ROOT = Path(__file__).resolve().parents[1]


class PreparationSplitTests(unittest.TestCase):
    def setUp(self):
        self.contract = json.loads((ROOT / 'completion-contract.json').read_bytes())
        self.plan = json.loads((ROOT / 'plan.json').read_bytes())
        self.plan = S.previous_plan(self.contract, self.plan)
        self.contract = S.previous_contract(self.contract)
        self.policy = self.contract['source_preparation_splits']

    def unit(self, plan, uid):
        return next(unit for unit in plan['units'] if unit['id'] == uid)

    def deny(self, change):
        contract, plan = copy.deepcopy(self.contract), copy.deepcopy(self.plan)
        change(contract, plan)
        with self.assertRaises(ValueError):
            P.validate_preparation_splits(contract, plan)
        with self.assertRaises(ValueError):
            P.legacy_plan(contract, plan)

    def test_exact_current_contract_plan_and_legacy_views_are_private(self):
        before_c, before_p = copy.deepcopy(self.contract), copy.deepcopy(self.plan)
        self.assertEqual(P.validate_preparation_splits(self.contract, self.plan), self.policy)
        old_c = P.legacy_contract(self.contract)
        old_p = P.legacy_plan(self.contract, self.plan)
        self.assertEqual(P.canonical_hash(old_c), P.BASE_CONTRACT)
        normalized = [P._normalized(unit) for unit in sorted(old_p['units'], key=lambda u: u['id'])]
        self.assertEqual(P.canonical_hash(normalized), P.BASE_UNITS)
        self.assertEqual(P.canonical_hash(old_p['readiness_graph']), P.BASE_GRAPH)
        self.assertEqual((len(old_p['units']), len(old_p['readiness_graph'])), (790, 1580))
        self.assertEqual(old_p['unit_count'], 790)
        self.assertNotIn('source_preparation_splits', old_c)
        self.assertEqual(old_c['contract_id'], 'SYSON-RUST-COMPLETION/1.0.8')
        self.assertEqual({unit['completion_contract']['id'] for unit in old_p['units']},
                         {'SYSON-RUST-COMPLETION/1.0.8'})
        self.assertEqual(self.unit(old_p, 'B25S')['dependencies'], ['B01'])
        self.assertIn('RR24', self.unit(old_p, 'B25S')['acceptance_children'])
        self.assertTrue({'C06', 'C07'} <= set(self.unit(old_p, 'B25S')['acceptance_gates']))
        self.assertTrue({'M104', 'RR01', 'RR02'} <= set(self.unit(old_p, 'M06')['acceptance_children']))
        old_c['families']['model'].clear()
        old_p['units'][0]['tasks'] = []
        self.assertEqual((self.contract, self.plan), (before_c, before_p))

    def test_missing_or_changed_policy_refuses_before_legacy_exclusion(self):
        for mutation in ['missing', 'unknown', 'count', 'team', 'dependency', 'restore']:
            contract = copy.deepcopy(self.contract)
            policy = contract['source_preparation_splits']
            if mutation == 'missing':
                del contract['source_preparation_splits']
            elif mutation == 'unknown':
                policy['unreviewed_exclusion'] = ['RR24']
            elif mutation == 'count':
                policy['original_unit_count'] = 789
            elif mutation == 'team':
                policy['children']['B25S1']['team'] = 'verification-release'
            elif mutation == 'dependency':
                policy['children']['B25S1']['dependencies'] = []
            else:
                owner = next(iter(policy['record_updates']))
                policy['record_updates'][owner] = {}
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                P.legacy_contract(contract)

    def test_repinning_a_changed_child_inside_the_policy_does_not_authorize_it(self):
        def change(contract, plan):
            child = self.unit(plan, 'B25S1')
            child['verification'] = []
            contract['source_preparation_splits']['children']['B25S1']['normalized_sha256'] = (
                P.canonical_hash(P._normalized(child)))
        self.deny(change)

    def test_pending_seal_and_unsupported_versions_never_get_compatibility_view(self):
        with mock.patch.object(P, 'POLICY_SHA256', 'PENDING_ROOT_FREEZE'):
            with self.assertRaises(ValueError):
                P.legacy_contract(self.contract)
        for version in ['1.0.8', '1.0.10', 'future', None]:
            contract = copy.deepcopy(self.contract)
            contract['version'] = version
            with self.subTest(version=version), self.assertRaises(ValueError):
                P.legacy_contract(contract)

    def test_contract_children_inventory_family_and_old_policy_cannot_be_changed(self):
        mutations = [
            lambda c, p: c['unit_ids'].remove('B25S1'),
            lambda c, p: c['unit_ids'].append('M06a1'),
            lambda c, p: c['families']['interchange'].remove('B25S1'),
            lambda c, p: c['families']['model'].append('B25S1'),
            lambda c, p: c['common_gates'].append('invented gate'),
            lambda c, p: c['team_execution'].update(unreviewed=True),
            lambda c, p: c['forest_kernel_splits'].clear(),
        ]
        for index, change in enumerate(mutations):
            with self.subTest(index=index):
                self.deny(change)

    def test_child_records_cannot_drop_api_tests_source_pins_or_add_unknown_fields(self):
        for uid in ['B25S1', 'M06a1']:
            for key, value in [('tasks', []), ('verification', []), ('paths', []),
                               ('source_refs', []), ('unreviewed_extra', 'not ignored')]:
                with self.subTest(uid=uid, key=key):
                    self.deny(lambda c, p: self.unit(p, uid).__setitem__(key, value))
        self.deny(lambda c, p: self.unit(p, 'B25S1')['budget'].__setitem__('production_lines_max', 401))
        self.deny(lambda c, p: self.unit(p, 'M06a1')['budget'].__setitem__('authored_lines_max', True))

    def test_original790_unknown_fields_and_actual_caller_gates_remain_authoritative(self):
        for uid, key, value in [('B25S', 'verification', []), ('RR24', 'verification', []),
                               ('RR22', 'tasks', []), ('C03b', 'done_when', 'accepted'),
                               ('C03c', 'unknown_future_field', {'must': 'remain visible'})]:
            with self.subTest(uid=uid, key=key):
                self.deny(lambda c, p: self.unit(p, uid).__setitem__(key, value))

    def test_all_sealed_parent_and_derived_closure_additions_are_required(self):
        for uid, fields in self.policy['record_updates'].items():
            for field in fields:
                with self.subTest(uid=uid, field=field):
                    self.deny(lambda c, p: self.unit(p, uid).pop(field))
        self.deny(lambda c, p: self.unit(p, 'B25S')['acceptance_children'].remove('RR24'))
        self.deny(lambda c, p: self.unit(p, 'M06')['acceptance_children'].remove('RR02'))

    def test_current_pointer_cannot_be_hidden_by_normalization(self):
        for uid in ['B25S1', 'M06a1', 'B25S', 'U12']:
            with self.subTest(uid=uid):
                self.deny(lambda c, p: self.unit(p, uid)['completion_contract'].__setitem__(
                    'id', 'SYSON-RUST-COMPLETION/1.0.8'))
        self.deny(lambda c, p: p['completion_contract'].__setitem__('path', 'alternate.json'))

    def test_duplicate_missing_unknown_and_wrong_count_plan_units_refuse(self):
        changes = [
            lambda c, p: p['units'].append(copy.deepcopy(p['units'][0])),
            lambda c, p: p['units'].pop(),
            lambda c, p: self.unit(p, 'B25S1').__setitem__('id', 'UNREVIEWED_CHILD'),
            lambda c, p: p.__setitem__('unit_count', 790),
        ]
        for index, change in enumerate(changes):
            with self.subTest(index=index):
                self.deny(change)

    def test_child_stage_dependencies_and_cycles_cannot_be_forged(self):
        for uid in ['B25S1', 'M06a1']:
            with self.subTest(uid=uid, stage='implementation'):
                changed = [] if uid == 'B25S1' else ['implementation:M03']
                self.deny(lambda c, p: p['readiness_graph'].__setitem__('implementation:' + uid, changed))
            with self.subTest(uid=uid, stage='accepted'):
                self.deny(lambda c, p: p['readiness_graph']['accepted:' + uid].append('accepted:B25S'))
        self.deny(lambda c, p: p['readiness_graph']['implementation:B25S1'].append('implementation:B25S'))

    def test_each_m06a1_native_and_parent_acceptance_gate_remains_required(self):
        gates = ['M03', 'M05', 'RR01', 'RR02', 'M104', 'C01', 'C02']
        for gate in gates:
            self.assertIn(gate, self.unit(self.plan, 'M06a1')['acceptance_gates'])
            with self.subTest(gate=gate, mutation='record'):
                self.deny(lambda c, p: self.unit(p, 'M06a1')['acceptance_gates'].remove(gate))
            with self.subTest(gate=gate, mutation='typed_edge'):
                self.deny(lambda c, p: p['readiness_graph']['accepted:M06a1'].remove('accepted:' + gate))

    def test_every_added_edge_and_existing_typed_edge_remain_required(self):
        for node, edges in self.policy['graph_additions'].items():
            for edge in edges:
                with self.subTest(node=node, edge=edge):
                    self.deny(lambda c, p: p['readiness_graph'][node].remove(edge))
        for replacement in [None, 'accepted:U12a', 'implementation:UNKNOWN']:
            def change(c, p):
                edges = p['readiness_graph']['accepted:C08']
                edges.remove('implementation:U12a')
                if replacement is not None:
                    edges.append(replacement)
            with self.subTest(replacement=replacement):
                self.deny(change)

    def test_duplicate_edges_missing_nodes_and_extra_nodes_refuse(self):
        self.deny(lambda c, p: p['readiness_graph']['accepted:B25S'].append(
            p['readiness_graph']['accepted:B25S'][0]))
        self.deny(lambda c, p: p['readiness_graph'].pop('accepted:M06a1'))
        self.deny(lambda c, p: p['readiness_graph'].__setitem__('accepted:UNREVIEWED', []))


if __name__ == '__main__':
    unittest.main()
