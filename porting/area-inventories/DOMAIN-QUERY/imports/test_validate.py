#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Mutation checks for the contract checker, not tests of a native importer."""
from copy import deepcopy
import unittest
import validate


class ContractChecks(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.contract = validate.read_json((validate.HERE / 'contract.json').read_text())
        cls.fixtures = validate.read_json((validate.HERE / 'fixtures.json').read_text())

    def setUp(self):
        self.data = deepcopy(self.fixtures)

    def case(self, name):
        return next(c for c in self.data['cases'] if c['id'] == name)

    def check(self):
        return validate.check_fixtures(self.contract, self.data)

    def test_complete_contract(self):
        self.assertEqual(self.check(), 19)

    def test_pinned_git_objects(self):
        validate.check_sources(self.contract, validate.HERE.parents[3])

    def test_changed_source_hash(self):
        c = deepcopy(self.contract)
        c['sources']['namespace']['sha256'] = '0' * 64
        with self.assertRaisesRegex(ValueError, 'source drift'):
            validate.check_sources(c, validate.HERE.parents[3])

    def test_changed_anchor(self):
        c = deepcopy(self.contract)
        c['sources']['filter']['anchors'][0]['first'] += 1
        with self.assertRaisesRegex(ValueError, 'anchor drift'):
            validate.check_sources(c, validate.HERE.parents[3])

    def test_duplicate_json_keys(self):
        with self.assertRaisesRegex(ValueError, 'duplicate JSON key'):
            validate.read_json('{"member":1,"member":2}')

    def test_duplicate_namespace_identity(self):
        self.data['namespaces'].append('A')
        with self.assertRaisesRegex(ValueError, 'duplicate namespace'):
            self.check()

    def test_import_order_mutation(self):
        self.case('ordered-imports')['calls'].reverse()
        with self.assertRaisesRegex(ValueError, 'import call order'):
            self.check()

    def test_exclusion_append_mutation(self):
        self.case('ordered-imports')['calls'][0]['excluded'] = ['A', 'X']
        with self.assertRaisesRegex(ValueError, 'exclusion append'):
            self.check()

    def test_filter_side_effect_mutation(self):
        self.case('rejected-short-name-retains-primary')['filter_steps'][0]['used_after'] = ['a']
        with self.assertRaisesRegex(ValueError, 'filter state effect'):
            self.check()

    def test_alias_identity_mutation(self):
        self.data['members']['alias-distinct']['same_name_reference'] = True
        with self.assertRaisesRegex(ValueError, 'filter acceptance'):
            self.check()

    def test_output_order_mutation(self):
        self.case('ordered-imports')['expected']['memberships'].reverse()
        with self.assertRaisesRegex(ValueError, 'accepted order'):
            self.check()

    def test_cycle_visited_mutation(self):
        self.case('cycle-A-B-A')['visits'][-1]['action'] = 'enter'
        with self.assertRaisesRegex(ValueError, 'visited/exclusion trace'):
            self.check()

    def test_unknown_graph_identity(self):
        self.case('cycle-A-B-A')['graph']['A']['imports'][0]['target'] = 'missing'
        with self.assertRaisesRegex(ValueError, 'graph import target'):
            self.check()

    def test_missing_unimplemented_outcome(self):
        self.data['cases'] = [c for c in self.data['cases'] if c['id'] != 'memberships-of-visibility-is-null']
        with self.assertRaisesRegex(ValueError, 'missing unsupported/null outcomes'):
            self.check()

    def test_null_outcome_cannot_be_empty(self):
        self.case('memberships-of-visibility-is-null')['expected']['memberships'] = []
        with self.assertRaisesRegex(ValueError, 'no fabricated list'):
            self.check()

    def test_unresolved_not_fabricated(self):
        self.case('unresolved-proxy-not-absence')['expected']['status'] = 'MEMBERSHIPS'
        with self.assertRaisesRegex(ValueError, 'membership outcome requires list'):
            self.check()

    def test_missing_obligation(self):
        for c in self.data['cases']:
            c['obligations'] = [x for x in c['obligations'] if x != 'owned_conflict_seed']
        with self.assertRaisesRegex(ValueError, 'missing obligation coverage'):
            self.check()


if __name__ == '__main__':
    unittest.main()
