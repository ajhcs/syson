"""Exercise the public overlay/lease checks against the complete actual plan."""
import copy
import json
from pathlib import Path
import unittest
from team_policy import validate_dispatch, validate_projection, validate_teams_legacy as validate_teams
from unittest.mock import patch
from source_next_splits import legacy_plan
import team_policy

ROOT = Path(__file__).resolve().parents[1]


class TeamPolicyTests(unittest.TestCase):
    def setUp(self):
        self.policy = json.loads((ROOT / 'teams.json').read_bytes())
        self.plan = json.loads((ROOT / 'plan.json').read_bytes())
        self.current_plan = copy.deepcopy(self.plan)
        contract = json.loads((ROOT / 'completion-contract.json').read_bytes())
        self.plan = legacy_plan(contract, self.plan)
        mocked = patch.object(team_policy, 'validate_teams', validate_teams)
        mocked.start()
        self.addCleanup(mocked.stop)
        self.byid = {unit['id']: unit for unit in self.plan['units']}
        self.assignment = {row['unit']: row['primary_team'] for row in self.policy['assignments']}

    def lease(self, uid='U11b', author='author-a', path=None):
        return {'unit': uid, 'team': self.assignment[uid], 'author': author,
                'reviewer': 'independent-reviewer', 'write_paths': [path or self.byid[uid]['paths'][0]],
                'ownership_admitted_by': 'coordinator', 'overlaps_held_scope': False,
                'handoff_verified': False, 'shared_wiring': False}

    def dispatch(self, leases=None):
        return {'coordinator': 'coordinator', 'authors': leases or [self.lease()]}

    def test_complete_overlay_preserves_input_and_typed_joint_edges(self):
        before = copy.deepcopy(self.plan)
        report = validate_teams(self.policy, self.plan)
        self.assertEqual(self.plan, before)
        self.assertEqual(len(report['assignment']), 790)
        self.assertEqual(sum(report['team_counts'].values()), 790)
        self.assertEqual(len(report['team_counts']), 7)
        self.assertTrue(all(report['team_counts'].values()))
        self.assertIn({'from': 'accepted:C08', 'to': 'implementation:U12a',
                       'from_team': 'verification-release', 'to_team': 'geometry-diagrams'},
                      report['cross_team_edges'])
        validate_projection(self.policy, self.plan, report)

    def test_missing_duplicate_and_unknown_unit_assignments_refuse(self):
        for mutation, expected in [('missing', 'missing'), ('duplicate', 'duplicate'), ('unknown', 'unknown team unit')]:
            changed = copy.deepcopy(self.policy)
            if mutation == 'missing':
                changed['assignments'].pop()
            elif mutation == 'duplicate':
                changed['assignments'].append(changed['assignments'][0])
            else:
                changed['assignments'][0]['unit'] = 'UNKNOWN'
            with self.subTest(mutation=mutation), self.assertRaisesRegex(ValueError, expected):
                validate_teams(changed, self.plan)

    def test_unknown_team_and_coherent_assignment_change_refuse(self):
        for team, expected in [('eighth-team', 'unknown primary'), ('reference', 'primary assignment')]:
            changed = copy.deepcopy(self.policy)
            changed['assignments'][0]['primary_team'] = team
            with self.subTest(team=team), self.assertRaisesRegex(ValueError, expected):
                validate_teams(changed, self.plan)

    def test_original_task_or_unknown_unit_field_cannot_disappear(self):
        for key, value in [('tasks', []), ('unreviewed_extra', 'must remain detectable')]:
            changed = copy.deepcopy(self.plan)
            changed['units'][0][key] = value
            with self.subTest(key=key), self.assertRaisesRegex(ValueError, 'unit obligation'):
                validate_teams(self.policy, changed)

    def test_only_two_generated_unit_fields_are_excluded(self):
        changed = copy.deepcopy(self.plan)
        changed['units'][0]['source_fragment_sha256'] = 'new generated hash'
        changed['units'][0]['completion_contract'] = {'version': 'new pointer'}
        validate_teams(self.policy, changed)

    def test_typed_readiness_edge_cannot_be_dropped_or_retyped(self):
        for retype in [False, True]:
            changed = copy.deepcopy(self.plan)
            node = 'accepted:C08'
            changed['readiness_graph'][node].remove('implementation:U12a')
            if retype:
                changed['readiness_graph'][node].append('accepted:U12a')
            with self.subTest(retype=retype), self.assertRaisesRegex(ValueError, 'typed readiness'):
                validate_teams(self.policy, changed)

    def test_generated_cross_team_edge_or_identity_cannot_be_substituted(self):
        report = validate_teams(self.policy, self.plan)
        for mutation in ['drop', 'team', 'stage']:
            changed = copy.deepcopy(report)
            if mutation == 'drop':
                changed['cross_team_edges'].pop()
            elif mutation == 'team':
                changed['cross_team_edges'][0]['to_team'] = 'eighth-team'
            else:
                changed['cross_team_edges'][0]['from'] = 'implementation:C99'
            with self.subTest(mutation=mutation), self.assertRaisesRegex(ValueError, 'cross-team projection'):
                validate_projection(self.policy, self.plan, changed)

    def test_cap_review_hold_and_dependency_waivers_refuse(self):
        changes = {'source_authors_max': 4, 'coordinators_max': 2, 'concurrent_agents_max': 5,
                   'self_acceptance_allowed': True, 'independent_review_required': False,
                   'held_scope_release': 'automatic', 'shared_wiring': 'team_owned',
                   'dependency_or_acceptance_waiver': True, 'assignment_grants_write_lease': True}
        for key, value in changes.items():
            changed = copy.deepcopy(self.policy)
            changed['execution_policy'][key] = value
            with self.subTest(key=key), self.assertRaisesRegex(ValueError, 'execution caps/review/ownership'):
                validate_teams(changed, self.plan)

    def test_valid_declared_lease_does_not_modify_plan(self):
        dispatch = self.dispatch()
        result = validate_dispatch(self.policy, self.plan, dispatch)
        self.assertEqual(result['dispatch'], dispatch)
        self.assertEqual(result['team_execution'], validate_teams(self.policy, self.plan))

    def test_self_review_coordinator_authorship_and_missing_ownership_refuse(self):
        for key, value, expected in [('reviewer', 'author-a', 'self-review'),
                                     ('author', 'coordinator', 'role conflict'),
                                     ('ownership_admitted_by', 'author-a', 'ownership admission')]:
            lease = self.lease()
            lease[key] = value
            with self.subTest(key=key), self.assertRaisesRegex(ValueError, expected):
                validate_dispatch(self.policy, self.plan, self.dispatch([lease]))

    def test_false_held_declaration_does_not_release_known_factory_hold(self):
        lease = self.lease('M02')
        with self.assertRaisesRegex(ValueError, 'held scope bypass'):
            validate_dispatch(self.policy, self.plan, self.dispatch([lease]))
        lease.update(overlaps_held_scope=True, handoff_verified=True)
        validate_dispatch(self.policy, self.plan, self.dispatch([lease]))

    def test_declared_held_overlap_requires_explicit_handoff(self):
        lease = self.lease()
        lease['overlaps_held_scope'] = True
        with self.assertRaisesRegex(ValueError, 'held scope bypass'):
            validate_dispatch(self.policy, self.plan, self.dispatch([lease]))

    def test_manifest_false_shared_declaration_and_explicit_shared_write_refuse(self):
        for lease in [self.lease('B25S'), self.lease('RR01'), dict(self.lease(), shared_wiring=True)]:
            with self.assertRaisesRegex(ValueError, 'shared wiring'):
                validate_dispatch(self.policy, self.plan, self.dispatch([lease]))

    def test_different_primary_teams_cannot_overlap_export_source(self):
        leases = [self.lease('RR06', 'author-a'), self.lease('RR07', 'author-b')]
        with self.assertRaisesRegex(ValueError, 'overlapping author'):
            validate_dispatch(self.policy, self.plan, self.dispatch(leases))

    def test_four_source_authors_refuse(self):
        leases = [self.lease('U11b', 'a'), self.lease('C01a', 'b', 'rust/tools/reference-binding/src/policy.rs'),
                  self.lease('P01', 'c', self.byid['P01']['paths'][1]),
                  self.lease('B25S', 'd', self.byid['B25S']['paths'][2])]
        with self.assertRaisesRegex(ValueError, 'author/agent cap'):
            validate_dispatch(self.policy, self.plan, self.dispatch(leases))

    def test_outside_noncanonical_and_unknown_team_leases_refuse(self):
        for path in ['../escape', '/absolute', 'rust//bad', 'outside/scope', 17]:
            with self.subTest(path=path), self.assertRaises(ValueError):
                validate_dispatch(self.policy, self.plan, self.dispatch([self.lease(path=path)]))
        lease = self.lease()
        lease['team'] = 'reference'
        with self.assertRaisesRegex(ValueError, 'lease primary team'):
            validate_dispatch(self.policy, self.plan, self.dispatch([lease]))


class CurrentPreparationTeamTests(unittest.TestCase):
    def test_current_overlay_and_pointer_refusal(self):
        policy = json.loads((ROOT / 'teams.json').read_bytes())
        plan = json.loads((ROOT / 'plan.json').read_bytes())
        report = team_policy.validate_teams(policy, plan)
        self.assertEqual(len(report['assignment']), 799)
        self.assertEqual(report['assignment']['B25S1'], 'services-persistence')
        self.assertEqual(report['assignment']['M06a1'], 'language-interchange')
        self.assertEqual(report['assignment']['RR13a'], 'services-persistence')
        self.assertEqual(report['assignment']['RR13b'], 'verification-release')
        self.assertEqual(report['assignment']['B25d1'], 'services-persistence')
        self.assertEqual(report['assignment']['B25d2'], 'services-persistence')
        self.assertEqual(report['assignment']['M06a2'], 'services-persistence')
        self.assertEqual(report['assignment']['M06a2a'], 'services-persistence')
        self.assertEqual(report['assignment']['M06a2b'], 'services-persistence')
        plan['units'][0]['completion_contract'] = {'version': 'invalid'}
        with self.assertRaises(ValueError):
            team_policy.validate_teams(policy, plan)


    def test_generated_source_waves_include_every_preparation_child_first(self):
        plan = json.loads((ROOT / 'plan.json').read_bytes())
        units = {unit['id']: unit for unit in plan['units']}
        waves = plan['source_author_dispatch_waves']
        flattened = [uid for wave in waves for uid in wave]
        self.assertEqual(sorted(flattened), sorted(units))
        self.assertTrue(all(0 < len(wave) <= 3 for wave in waves))
        position = {uid: index for index, wave in enumerate(waves) for uid in wave}
        for uid, unit in units.items():
            prerequisites = unit['dependencies'] + unit.get('source_preparation_children', [])
            for dependency in prerequisites:
                with self.subTest(unit=uid, prerequisite=dependency):
                    self.assertLess(position[dependency], position[uid])
        self.assertEqual(plan['unordered_write_conflicts'], [])


if __name__ == '__main__':
    unittest.main()
