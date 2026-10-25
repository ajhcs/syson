#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Synthetic preflight boundary checks, not build or application tests."""
from copy import deepcopy
import unittest
import check


class PreflightChecks(unittest.TestCase):
    def setUp(self):
        self.plan = check.read_json((check.HERE / 'preflight.json').read_text())

    def test_frozen_source_objects_and_lifecycle_facts(self):
        self.assertEqual(check.verify_sources(), 13)
        check.verify_facts(self.plan)

    def test_valid_preparation_stays_blocked(self):
        result = check.validate_plan(self.plan)
        self.assertEqual((result['checker'], result['status'], result['execution']), ('PASS', 'BLOCKED', 'NOT_RUN'))
        self.assertGreaterEqual(len(result['blockers']), 6)

    def test_actual_inventory_streams_and_alternates_are_retained(self):
        inventory = check.read_json((check.HERE / 'tool-inventory.json').read_text())
        check.verify_tool_capture(inventory)
        versions = [r.get('observed_version') for r in inventory['commands']]
        self.assertIn('24.21.0', versions)
        self.assertIn('24.22.0', versions)
        self.assertNotIn(self.plan['requirements']['node'], versions)

    def test_missing_tool_does_not_become_registry_denial(self):
        self.plan['registry_access']['authorization'] = 'DENIED'
        with self.assertRaisesRegex(ValueError, 'without probe'):
            check.validate_plan(self.plan)

    def test_scope_cannot_promote_to_execution(self):
        for key, value in [('execution', 'RUN'), ('requires_separate_action_review', False), ('status', 'READY')]:
            plan = deepcopy(self.plan)
            plan[key] = value
            with self.assertRaises(ValueError):
                check.validate_plan(plan)

    def test_source_pin_changes_are_rejected(self):
        self.plan['source_revision'] = '0' * 40
        with self.assertRaisesRegex(ValueError, 'source pin'):
            check.validate_plan(self.plan)

    def test_workspace_root_and_traversal_are_rejected(self):
        for value in ['/tmp/reference', '/home/builderbob/reference', '/workspace/worktrees/builderbob/syson/reference-../bad/../x']:
            plan = deepcopy(self.plan)
            plan['workspace']['source_checkout'] = value
            with self.assertRaises(ValueError):
                check.validate_plan(plan)

    def test_cache_roots_cannot_collide(self):
        self.plan['workspace']['npm_cache'] = self.plan['workspace']['maven_repository']
        with self.assertRaisesRegex(ValueError, 'collide'):
            check.validate_plan(self.plan)

    def test_nonloopback_and_source_default_ports_do_not_silently_apply(self):
        self.plan['network']['bind_address'] = '0.0.0.0'
        with self.assertRaisesRegex(ValueError, 'loopback'):
            check.validate_plan(self.plan)
        self.plan['network']['bind_address'] = '127.0.0.1'
        self.plan['network']['application_port'] = self.plan['network']['database_port']
        with self.assertRaisesRegex(ValueError, 'collision'):
            check.validate_plan(self.plan)

    def test_unprobed_paths_or_ports_are_not_labeled_fresh(self):
        for section, key, value in [('workspace', 'destinations_verified_fresh', True), ('network', 'port_availability', 'FREE')]:
            plan = deepcopy(self.plan)
            plan[section][key] = value
            with self.assertRaisesRegex(ValueError, 'unprobed'):
                check.validate_plan(plan)

    def test_ownership_resource_and_cleanup_policies_are_fixed(self):
        for key, value in [('compiler_jobs', 2), ('owned_process_group_only', False), ('automatic_cleanup', True), ('service_registration', True), ('new_installations', True)]:
            plan = deepcopy(self.plan)
            plan['limits'][key] = value
            with self.assertRaises(ValueError):
                check.validate_plan(plan)

    def test_stages_are_not_reordered(self):
        self.plan['command_templates'].reverse()
        with self.assertRaisesRegex(ValueError, 'stage order'):
            check.validate_plan(self.plan)

    def test_maven_update_or_deploy_is_not_the_frozen_plan(self):
        for replacement in ['-U', 'deploy']:
            plan = deepcopy(self.plan)
            plan['command_templates'][3]['argv'].append(replacement)
            with self.assertRaisesRegex(ValueError, 'Maven'):
                check.validate_plan(plan)

    def test_lifecycle_suppression_is_not_dropped_in_cache_step(self):
        self.plan['command_templates'][1]['argv'].remove('--ignore-scripts')
        with self.assertRaisesRegex(ValueError, 'lifecycle'):
            check.validate_plan(self.plan)

    def test_launch_endpoint_and_argv_encoding_are_checked(self):
        for value in ['--server.address=0.0.0.0', '--server.address=127.0.0.1\nlaunch']:
            plan = deepcopy(self.plan)
            plan['command_templates'][4]['argv'][3] = value
            with self.assertRaises(ValueError):
                check.validate_plan(plan)

    def test_unreviewed_binding_and_snapshot_resolution_are_not_promoted(self):
        self.plan['bindings']['JAVA21_BINARY'] = '/workspace/java/bin/java'
        with self.assertRaisesRegex(ValueError, 'binding'):
            check.validate_plan(self.plan)
        self.plan['bindings']['JAVA21_BINARY'] = None
        self.plan['dependency_closure']['snapshot']['timestamp'] = 'guessed'
        with self.assertRaisesRegex(ValueError, 'SNAPSHOT'):
            check.validate_plan(self.plan)

    def test_stop_and_retention_conditions_cannot_be_omitted(self):
        for key in ['stop_conditions', 'retention']:
            plan = deepcopy(self.plan)
            plan[key] = []
            with self.assertRaisesRegex(ValueError, 'conditions'):
                check.validate_plan(plan)

    def test_missing_binding_or_script_keys_cannot_be_ignored(self):
        self.plan['bindings'].clear()
        with self.assertRaisesRegex(ValueError, 'binding manifest shape'):
            check.validate_plan(self.plan)
        with self.assertRaisesRegex(ValueError, 'script scope'):
            check.verify_facts({'requirements': {'java_major': '21', 'node': '24.20.0', 'npm': '11.19.0', 'maven': 'VERSION_NOT_PINNED_BY_INSPECTED_SOURCE'}, 'source_scripts': {'../unknown.json': {}}})

    def test_dependency_closure_cannot_be_inferred_from_source(self):
        self.plan['dependency_closure']['maven_effective_graph'] = 'COMPLETE'
        with self.assertRaisesRegex(ValueError, 'dependency promotion'):
            check.validate_plan(self.plan)

    def test_duplicate_json_keys_are_rejected(self):
        with self.assertRaisesRegex(ValueError, 'duplicate JSON key'):
            check.read_json('{"status":"BLOCKED","status":"READY"}')


if __name__ == '__main__':
    unittest.main()
