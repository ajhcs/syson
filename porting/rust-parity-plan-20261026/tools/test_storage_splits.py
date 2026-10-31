"""Full current-plan mutations; no product, compiler or filesystem subprocesses."""
import copy
import json
from pathlib import Path
import unittest
from unittest import mock
import preparation_splits as P
import storage_splits as S

ROOT = Path(__file__).resolve().parents[1]


class StorageSplitTests(unittest.TestCase):
    def setUp(self):
        self.contract = json.loads((ROOT / 'completion-contract.json').read_bytes())
        self.plan = json.loads((ROOT / 'plan.json').read_bytes())
        self.policy = self.contract['storage_source_splits']

    def unit(self, plan, uid):
        return next(unit for unit in plan['units'] if unit['id'] == uid)

    def deny(self, change):
        contract, plan = copy.deepcopy(self.contract), copy.deepcopy(self.plan)
        change(contract, plan)
        for check in [S.validate_storage_splits, S.previous_plan, S.legacy_plan]:
            with self.subTest(check=check.__name__), self.assertRaises(ValueError):
                check(contract, plan)

    def test_full_current_and_both_legacy_proofs_preserve_private_inputs(self):
        before = copy.deepcopy((self.contract, self.plan))
        self.assertEqual(S.validate_storage_splits(self.contract, self.plan,
                         lambda path: (ROOT / path).read_bytes()), self.policy)
        old_c, old_p = S.previous_contract(self.contract), S.previous_plan(self.contract, self.plan)
        self.assertEqual(S.digest(old_c), S.BASE_CONTRACT)
        self.assertEqual((old_p['unit_count'], len(old_p['units']), len(old_p['readiness_graph'])),
                         (792, 792, 1584))
        self.assertEqual(S.digest([P._normalized(unit) for unit in sorted(old_p['units'],
                         key=lambda row: row['id'])]), S.BASE_UNITS)
        self.assertEqual(S.digest(old_p['readiness_graph']), S.BASE_GRAPH)
        self.assertEqual(S.digest(S.legacy_contract(self.contract)), P.BASE_CONTRACT)
        self.assertEqual(len(S.legacy_plan(self.contract, self.plan)['units']), 790)
        old_p['units'][0]['tasks'] = []
        old_c['families']['model'].clear()
        self.assertEqual((self.contract, self.plan), before)

    def test_pending_policy_unknown_version_and_policy_repin_are_refused(self):
        with mock.patch.object(S, 'POLICY_SHA256', 'PENDING_ROOT_FREEZE'):
            with self.assertRaises(ValueError):
                S.previous_contract(self.contract)
        for version in ['1.0.9', '1.0.11', 'future', None]:
            contract = copy.deepcopy(self.contract)
            contract['version'] = version
            with self.subTest(version=version), self.assertRaises(ValueError):
                S.validate_storage_splits(contract)
        for field, value in [('schema', True), ('original_unit_count', 791),
                             ('historical_unit_count', 54), ('extra_exclusion', ['M06a1'])]:
            self.deny(lambda c, p: c['storage_source_splits'].__setitem__(field, value))
        def repin(c, p):
            child = self.unit(p, 'RR13a')
            child['verification'] = []
            c['storage_source_splits']['children']['RR13a']['normalized_sha256'] = (
                S.digest(P._normalized(child)))
        self.deny(repin)

    def test_contract_inventory_family_old_policies_and_budget_exceptions_stay_exact(self):
        changes = [
            lambda c, p: c['unit_ids'].remove('RR13a'),
            lambda c, p: c['unit_ids'].append('RR13b'),
            lambda c, p: c['families']['repairs'].remove('RR13a'),
            lambda c, p: c['families']['model'].append('RR13a'),
            lambda c, p: c['source_preparation_splits']['children']['M06a1'].clear(),
            lambda c, p: c['team_execution'].update(unreviewed=True),
            lambda c, p: c['legacy_prerequisite_budget_dispositions'].update(RR13a={}),
        ]
        for index, change in enumerate(changes):
            with self.subTest(index=index):
                self.deny(change)

    def test_complete_child_api_tests_pins_caps_and_unknown_fields_are_sealed(self):
        for uid in sorted(S.CHILDREN):
            for field, value in [('tasks', []), ('verification', []), ('paths', []),
                                 ('source_freeze', {}), ('unknown_extra', {'not': 'ignored'})]:
                with self.subTest(uid=uid, field=field):
                    self.deny(lambda c, p: self.unit(p, uid).__setitem__(field, value))
            self.deny(lambda c, p: self.unit(p, uid)['budget'].__setitem__('production_lines_max', 401))
            self.deny(lambda c, p: self.unit(p, uid)['budget'].__setitem__('authored_lines_max', True))

    def test_all792_prior_records_including_unchanged_m06a1_remain_authoritative(self):
        for uid, field, value in [('M06a1', 'tasks', []), ('RR13', 'verification', []),
                                 ('M06', 'source_refs', []), ('M104', 'done_when', 'accepted'),
                                 ('B06', 'dependencies', []), ('B02', 'dependencies', []),
                                 ('C01', 'unknown_future_field', 'cannot disappear')]:
            with self.subTest(uid=uid, field=field):
                self.deny(lambda c, p: self.unit(p, uid).__setitem__(field, value))
        for gate in ['M03', 'M05', 'RR01', 'RR02', 'M104', 'C01', 'C02']:
            self.deny(lambda c, p: self.unit(p, 'M06a1')['acceptance_gates'].remove(gate))
        for uid, fields in self.policy['record_updates'].items():
            for field in fields:
                with self.subTest(uid=uid, field=field):
                    self.deny(lambda c, p: self.unit(p, uid).pop(field))

    def test_current_pointers_counts_missing_duplicates_and_unknown_units_refuse(self):
        for uid in ['RR13a', 'RR13b', 'M06a1', 'M06', 'U12']:
            self.deny(lambda c, p: self.unit(p, uid)['completion_contract'].__setitem__(
                'id', 'SYSON-RUST-COMPLETION/1.0.9'))
        for change in [lambda c, p: p.__setitem__('unit_count', 792),
                       lambda c, p: p['units'].pop(),
                       lambda c, p: p['units'].append(copy.deepcopy(p['units'][0])),
                       lambda c, p: self.unit(p, 'RR13a').__setitem__('id', 'UNKNOWN'),
                       lambda c, p: p['completion_contract'].__setitem__('lock', 'alternate.json')]:
            self.deny(change)

    def test_fault_parent_and_every_original_or_new_typed_edge_are_required(self):
        self.assertTrue({'RR13a', 'RR13b'} <= set(self.unit(self.plan, 'RR13')['acceptance_children']))
        self.deny(lambda c, p: self.unit(p, 'RR13')['dependencies'].remove('RR04'))
        self.deny(lambda c, p: self.unit(p, 'RR13')['acceptance_children'].remove('RR13b'))
        for node, edges in self.policy['graph_additions'].items():
            for edge in edges:
                with self.subTest(node=node, edge=edge):
                    self.deny(lambda c, p: p['readiness_graph'][node].remove(edge))
        for node, edge in [('accepted:M06a1', 'accepted:C01'),
                           ('implementation:B02', 'implementation:C01')]:
            self.deny(lambda c, p: p['readiness_graph'][node].remove(edge))
        self.deny(lambda c, p: p['readiness_graph']['implementation:RR13a'].append(
            'implementation:RR13b'))
        self.deny(lambda c, p: p['readiness_graph']['implementation:RR13b'].clear())
        self.deny(lambda c, p: p['readiness_graph']['accepted:RR13a'].append('accepted:RR13'))
        self.deny(lambda c, p: p['readiness_graph']['accepted:RR13'].append('accepted:RR13a'))
        self.deny(lambda c, p: p['readiness_graph'].pop('accepted:RR13b'))
        self.deny(lambda c, p: p['readiness_graph'].__setitem__('accepted:UNKNOWN', []))

    def test_even_a_reviewed_new_graph_addition_cannot_create_a_cycle(self):
        contract, plan = copy.deepcopy(self.contract), copy.deepcopy(self.plan)
        self.assertIn('accepted:RR13', plan['readiness_graph']['accepted:C03'])
        plan['readiness_graph']['accepted:RR13'].append('accepted:C03')
        contract['storage_source_splits']['graph_additions'].setdefault(
            'accepted:RR13', []).append('accepted:C03')
        # Deliberately pass the seal to exercise the independent cycle defense.
        with mock.patch.object(S, 'POLICY_SHA256', S.digest(contract['storage_source_splits'])):
            with self.assertRaisesRegex(ValueError, 'storage typed cycle'):
                S.previous_plan(contract, plan)

    def test_all55_history_records_and_preserved_hold_policy_bytes_are_checked(self):
        original = {name: (ROOT / name).read_bytes() for name in
                    set(self.policy['preserved_inputs']) | {'execution.json'}}
        for mutation in ['delete', 'change', 'extra']:
            history = json.loads(original['execution.json'])
            if mutation == 'delete':
                history['units'].pop('RR13')
            elif mutation == 'change':
                history['units']['RR13']['feature_acceptance'] = 'ACCEPTED'
            else:
                history['units']['UNREVIEWED'] = {}
            changed = dict(original, **{'execution.json': json.dumps(history).encode()})
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                S.validate_storage_splits(self.contract, self.plan, changed.__getitem__)
        for name in self.policy['preserved_inputs']:
            changed = dict(original)
            changed[name] += b' '
            with self.subTest(path=name), self.assertRaises(ValueError):
                S.validate_storage_splits(self.contract, self.plan, changed.__getitem__)


if __name__ == '__main__':
    unittest.main()
