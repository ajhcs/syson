# SPDX-License-Identifier: EPL-2.0
"""Synthetic plan-format fixtures only; never product or upstream execution evidence."""
import copy
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / 'scripts/check_execution_backlog.py'
spec = importlib.util.spec_from_file_location('backlog', SCRIPT)
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)


class BacklogTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        if 'SYSON_BUILD_DIR' not in os.environ:
            raise RuntimeError('SYSON_BUILD_DIR is required; no root-disk temporary fallback')
        cls.temp = tempfile.TemporaryDirectory(dir=os.environ['SYSON_BUILD_DIR'])
        cls.root = Path(cls.temp.name)
        (cls.root / 'porting').mkdir()
        (cls.root / 'porting/PARITY-LEDGER.json').write_text(json.dumps({'features': [{'id': aid} for aid in gate.EXPECTED_AREA_IDS]}))
        (cls.root / 'evidence.txt').write_text('Synthetic schema fixture; not product execution evidence.\n')
        cls.git('init', '-q')
        cls.git('add', 'porting/PARITY-LEDGER.json', 'evidence.txt')
        cls.git('-c', 'user.name=Synthetic Plan Test', '-c', 'user.email=test@example.invalid',
                '-c', 'core.hooksPath=/dev/null', '-c', 'commit.gpgsign=false', 'commit', '-qm', 'synthetic format fixture')
        cls.revision = cls.git('rev-parse', 'HEAD').decode().strip()

    @classmethod
    def git(cls, *args):
        return subprocess.check_output(['git', '-C', str(cls.root), *args], stderr=subprocess.DEVNULL)

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def ref(self, kind='source', status='RECORDED'):
        return {'path': 'evidence.txt', 'kind': kind, 'status': status,
                'scope': 'Synthetic bounded format check only', 'revision': self.revision}

    def setUp(self):
        self.plan = {'schema_version': 1, 'plan_id': 'SYNTHETIC-PLAN-01',
                     'recorded_at_utc': '2026-10-22T00:00:00Z',
                     'baseline': {'syson': gate.BASELINE, 'sirius_source_candidate': '3191e2b8990373afee3415f6b26f8c5039a28321', 'syside_source_candidate': '1326f234cae23b0e4777d5a48a069b17e9e332f1', 'artifact_correspondence': 'OPEN synthetic fixture'},
                     'integrator': 'test-owner', 'existing_lanes': ['test-owner', 'other-owner'],
                     'resources': {'global_max_compile_jobs': 2, 'max_compile_jobs': 0, 'allocation_authority': 'Synthetic parent allocation', 'per_process_jobs': 1, 'compiler_leases': [], 'displays': 'Synthetic no display execution'},
                     'authority': {'authorized': ['Synthetic validation only'], 'not_granted': ['No runtime execution']},
                     'continuity': {'mode': 'Synthetic test', 'future_wakeup_guaranteed': False, 'scheduler_report': {'source': 'Synthetic', 'result': 'Not run', 'automation_created': False, 'workaround_created': False}, 'resume_entrypoint': 'Read synthetic state'},
                     'history': [{'event': 'SYNTHETIC', 'recorded_at_utc': '2026-10-22T00:00:00Z', 'detail': 'Not product evidence'}],
                     'denominator': {'baseline_area_ids': list(gate.EXPECTED_AREA_IDS),
                                     'current_area_ids': list(gate.EXPECTED_AREA_IDS),
                                     'inventory_complete': False, 'unit': 'Synthetic eighteen anchors',
                                     'history': [{'id': 'DENOM-000001', 'type': 'ESTABLISH',
                                                  'recorded_at_utc': '2026-10-22T00:00:00Z',
                                                  'area_ids': list(gate.EXPECTED_AREA_IDS),
                                                  'reason': 'Synthetic format fixture, not accepted source inventory',
                                                  'source': 'porting/PARITY-LEDGER.json',
                                                  'source_revision': self.revision}]},
                     'areas': {}, 'items': {}}
        for aid in gate.EXPECTED_AREA_IDS:
            iid = 'PREP-' + aid
            self.plan['areas'][aid] = {'title': 'Synthetic ' + aid, 'lifecycle': 'ACTIVE', 'inventory_complete': False, 'unclosed_surface': ['Unassessed synthetic scope'], 'dependency_semantics': 'Synthetic sequencing',
                'upstream_baselines': [gate.BASELINE], 'inventory_refs': ['Pending bounded source inventory'],
                'known_behaviors': ['Synthetic scope'], 'exclusions': [], 'owner_lane': 'test-owner',
                'writable_boundary': ['rust', 'porting', 'evidence.txt'], 'dependencies': [], 'milestones': [iid],
                'rust_tests': ['Native tests pending'], 'upstream_tests': ['Oracle pending'],
                'integration_acceptance': ['Integration pending'], 'real_user_acceptance': ['User acceptance pending'],
                'provenance_license_gates': ['Source/license closure pending'], 'completion_evidence': [],
                'status': {'implemented': 'NONE', 'rust_tested': 'NONE', 'parity_verified': 'NO'},
                'blockers': [], 'next_ready_item': iid}
            self.plan['items'][iid] = {'area_ids': [aid], 'description': 'Bounded synthetic preparation',
                'kind': 'PREPARATION', 'state': 'READY', 'owner_lane': 'test-owner',
                'writable_boundary': ['rust/tests/' + aid], 'depends_on': [],
                'authorization_scope': 'Synthetic metadata only; grants no execution authority', 'next_action': 'Check synthetic plan',
                'rust_tests': ['Pending'], 'upstream_tests': ['Not run'], 'acceptance': ['Review synthetic bounded scope'],
                'completion_evidence': [], 'blockers': [], 'lease': None}
        self.aid = gate.EXPECTED_AREA_IDS[0]
        self.iid = 'PREP-' + self.aid
        self.item = self.plan['items'][self.iid]
        self.area = self.plan['areas'][self.aid]

    def check(self):
        return gate.validate(self.root, self.plan)

    def reject(self):
        with self.assertRaises(gate.Invalid):
            self.check()

    def active(self, item=None):
        item = self.item if item is None else item
        item['state'] = 'ACTIVE'
        item['lease'] = {'holder': item['owner_lane'], 'writable_boundary': copy.deepcopy(item['writable_boundary']),
                         'status': 'ACTIVE', 'release_mode': 'EXPLICIT_ONLY',
                         'recorded_at_utc': '2026-10-22T00:00:00Z', 'acknowledgement': 'Synthetic explicit ACK'}

    def blocked(self, item=None):
        item = self.item if item is None else item
        item['state'] = 'BLOCKED'
        item['blockers'] = [{'action': 'Read missing source', 'target': 'synthetic target',
                             'reason': 'Unavailable in synthetic fixture', 'status': 'unavailable'}]
        for aid in item['area_ids']:
            self.plan['areas'][aid]['next_ready_item'] = None

    def done(self):
        if self.item['lease'] is None:
            self.active()
            self.item['lease']['status'] = 'RELEASED'
        self.item['state'] = 'DONE'
        self.item['completion_evidence'] = [self.ref()]
        self.area['next_ready_item'] = None

    def test_valid_pending_plan_keeps_all_eighteen_anchors(self):
        self.assertEqual(self.check()['areas'], 18)
        self.assertEqual(self.check()['parity_verified_areas'], 0)

    def test_missing_area_and_hidden_denominator_removal(self):
        del self.plan['areas'][self.aid]
        self.reject()

    def test_denominator_addition_without_reviewed_event(self):
        self.plan['denominator']['current_area_ids'].append('NEW-HIDDEN-AREA')
        self.reject()

    def test_duplicate_denominator_id(self):
        self.plan['denominator']['baseline_area_ids'].append(self.aid)
        self.reject()

    def test_establishment_source_must_be_pinned(self):
        self.plan['denominator']['history'][0]['source_revision'] = 'main'
        self.reject()

    def test_unsupported_retirement_cannot_hide_original_scope(self):
        self.plan['denominator']['history'][0]['type'] = 'RETIRE'
        self.reject()

    def test_future_event_requires_reviewed_extension(self):
        self.plan['denominator']['history'].append(copy.deepcopy(self.plan['denominator']['history'][0]))
        self.reject()

    def test_area_dependency_cycle(self):
        second = gate.EXPECTED_AREA_IDS[1]
        self.area['dependencies'] = [second]
        self.plan['areas'][second]['dependencies'] = [self.aid]
        self.reject()

    def test_unknown_area_dependency(self):
        self.area['dependencies'] = ['MISSING']
        self.reject()

    def test_item_dependency_cycle(self):
        second = 'PREP-' + gate.EXPECTED_AREA_IDS[1]
        self.item['depends_on'] = [second]
        self.plan['items'][second]['depends_on'] = [self.iid]
        self.reject()

    def test_unknown_item_dependency(self):
        self.item['depends_on'] = ['MISSING']
        self.reject()

    def test_ready_unmet_dependency(self):
        self.item['depends_on'] = ['PREP-' + gate.EXPECTED_AREA_IDS[1]]
        self.reject()

    def test_active_unmet_dependency(self):
        self.active()
        self.item['depends_on'] = ['PREP-' + gate.EXPECTED_AREA_IDS[1]]
        self.reject()

    def test_blocked_dependency_wait_is_valid(self):
        self.blocked()
        self.item['depends_on'] = ['PREP-' + gate.EXPECTED_AREA_IDS[1]]
        self.assertEqual(self.check()['states']['BLOCKED'], 1)

    def test_missing_evidence_plan(self):
        self.area['real_user_acceptance'] = []
        self.reject()

    def test_empty_object_is_not_evidence_plan(self):
        self.area['upstream_tests'] = [{}]
        self.reject()

    def test_unknown_milestone_reference(self):
        self.area['milestones'] = ['MISSING']
        self.reject()

    def test_mismatched_milestone_area(self):
        self.area['milestones'] = ['PREP-' + gate.EXPECTED_AREA_IDS[1]]
        self.reject()

    def test_unknown_item_area(self):
        self.item['area_ids'] = ['MISSING']
        self.reject()

    def test_missing_next_ready_pointer(self):
        self.area['next_ready_item'] = None
        self.reject()

    def test_next_ready_cannot_target_other_area(self):
        self.area['next_ready_item'] = 'PREP-' + gate.EXPECTED_AREA_IDS[1]
        self.reject()

    def test_active_requires_explicit_lease(self):
        self.item['state'] = 'ACTIVE'
        self.reject()

    def test_valid_active_acknowledged_lease(self):
        self.active()
        self.assertEqual(self.check()['held_leases'], 1)

    def test_wrong_lease_holder(self):
        self.active()
        self.item['lease']['holder'] = 'different-owner'
        self.reject()

    def test_wrong_lease_boundary(self):
        self.active()
        self.item['lease']['writable_boundary'] = ['rust/other']
        self.reject()

    def test_lease_expiration_is_forbidden(self):
        self.active()
        self.item['lease']['expires_at'] = '2026-10-22T01:00:00Z'
        self.reject()

    def test_lease_ttl_release_mode_is_forbidden(self):
        self.active()
        self.item['lease']['release_mode'] = 'EXPIRE_AFTER_IDLE'
        self.reject()

    def test_held_lease_still_conflicts_when_item_blocked(self):
        self.active()
        self.blocked()
        other = self.plan['items']['PREP-' + gate.EXPECTED_AREA_IDS[1]]
        other['owner_lane'] = 'other-owner'
        other['writable_boundary'] = list(self.item['writable_boundary'])
        self.active(other)
        self.reject()

    def test_nested_path_overlap_conflicts(self):
        self.item['writable_boundary'] = ['rust/tests']
        self.active()
        other = self.plan['items']['PREP-' + gate.EXPECTED_AREA_IDS[1]]
        other['owner_lane'] = 'other-owner'
        self.active(other)
        self.reject()

    def test_shared_prefix_without_path_boundary_is_not_conflict(self):
        self.item['writable_boundary'] = ['rust/foo']
        self.active()
        other = self.plan['items']['PREP-' + gate.EXPECTED_AREA_IDS[1]]
        other['owner_lane'] = 'other-owner'
        other['writable_boundary'] = ['rust/foobar']
        self.active(other)
        self.assertEqual(self.check()['held_leases'], 2)

    def test_explicit_release_allows_other_owner(self):
        self.active()
        self.item['lease']['status'] = 'RELEASED'
        self.item['state'] = 'READY'
        other = self.plan['items']['PREP-' + gate.EXPECTED_AREA_IDS[1]]
        other['owner_lane'] = 'other-owner'
        other['writable_boundary'] = list(self.item['writable_boundary'])
        self.active(other)
        self.assertEqual(self.check()['held_leases'], 1)

    def test_done_requires_bounded_pinned_evidence(self):
        self.item['state'] = 'DONE'
        self.area['next_ready_item'] = None
        self.reject()

    def test_done_cannot_count_pending_evidence(self):
        self.done()
        self.item['completion_evidence'][0]['status'] = 'PENDING'
        self.reject()

    def test_done_requires_release_of_existing_lease(self):
        self.active()
        self.done()
        self.reject()

    def test_bounded_done_is_not_area_parity(self):
        self.done()
        result = self.check()
        self.assertEqual(result['states']['DONE'], 1)
        self.assertEqual(result['parity_verified_areas'], 0)

    def test_missing_evidence_artifact(self):
        self.done()
        self.item['completion_evidence'][0]['kind'] = 'rust_test'
        self.item['completion_evidence'][0]['path'] = 'missing.json'
        self.reject()

    def test_evidence_revision_cannot_be_branch_name(self):
        self.done()
        self.item['completion_evidence'][0]['revision'] = 'HEAD'
        self.reject()

    def test_evidence_cannot_drift_from_pinned_bytes(self):
        self.done()
        self.item['completion_evidence'][0]['kind'] = 'rust_test'
        original = (self.root / 'evidence.txt').read_bytes()
        try:
            (self.root / 'evidence.txt').write_text('changed bytes')
            self.reject()
        finally:
            (self.root / 'evidence.txt').write_bytes(original)

    def test_parity_yes_cannot_be_inferred_from_native_preparation(self):
        self.area['status'] = {'implemented': 'COMPLETE', 'rust_tested': 'COMPLETE', 'parity_verified': 'YES'}
        self.area['completion_evidence'] = [self.ref('rust_test', 'PASSED')]
        self.reject()

    def test_parity_yes_requires_all_real_layers_not_not_run_plans(self):
        self.plan['denominator']['inventory_complete'] = True
        self.area['status'] = {'implemented': 'COMPLETE', 'rust_tested': 'COMPLETE', 'parity_verified': 'YES'}
        self.area['completion_evidence'] = [self.ref(k, 'NOT_RUN') for k in gate.EVIDENCE_KINDS]
        self.reject()

    def test_parity_yes_rejects_text_as_structured_run_proof(self):
        self.plan['denominator']['inventory_complete'] = True
        self.area['status'] = {'implemented': 'COMPLETE', 'rust_tested': 'COMPLETE', 'parity_verified': 'YES'}
        self.area['completion_evidence'] = [self.ref(k, 'ACCEPTED') for k in gate.EVIDENCE_KINDS]
        self.reject()

    def test_blocker_requires_action_target_reason_status(self):
        self.blocked()
        del self.item['blockers'][0]['action']
        self.reject()

    def test_blocker_status_unknown_rejected(self):
        self.blocked()
        self.item['blockers'][0]['status'] = 'probably-fine'
        self.reject()

    def test_blocked_requires_explicit_blocker(self):
        self.blocked()
        self.item['blockers'] = []
        self.reject()

    def test_absolute_parent_and_glob_boundaries_rejected(self):
        for path in ['/tmp/x', '../escape', 'rust/../other', 'rust/**', 'rust//tests']:
            with self.subTest(path=path):
                with self.assertRaises(gate.Invalid):
                    gate.boundary(path, 'synthetic')

    def test_cross_area_validator_boundary_is_explicitly_owned_by_item_lease(self):
        self.area['writable_boundary'] = ['porting/one-file.json']
        self.active()
        self.assertEqual(self.check()['held_leases'], 1)

    def test_duplicate_json_key_rejected(self):
        with self.assertRaises(gate.Invalid):
            gate.strict_json('{"state":"READY","state":"DONE"}')

    def test_non_json_constant_rejected(self):
        for value in ['NaN', 'Infinity', '-Infinity']:
            with self.assertRaises(gate.Invalid):
                gate.strict_json('{"value":' + value + '}')

    def test_boolean_schema_version_rejected(self):
        self.plan['schema_version'] = True
        self.reject()

    def test_cli_optimization_cannot_disable_dependency_checks(self):
        self.item['depends_on'] = ['MISSING']
        path = self.root / 'negative-plan.json'
        path.write_text(json.dumps(self.plan))
        result = subprocess.run([sys.executable, '-O', str(SCRIPT), '--root', str(self.root), '--plan', str(path)],
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
        self.assertEqual(result.returncode, 1)
        self.assertIn(b'EXECUTION BACKLOG INVALID', result.stderr)
        self.assertNotIn(b'Traceback', result.stderr)

    def test_historical_source_proof_uses_exact_git_blob_after_local_evolution(self):
        self.done()
        original = (self.root / 'evidence.txt').read_bytes()
        try:
            (self.root / 'evidence.txt').write_text('New source revision; older bounded proof remains pinned.')
            self.assertEqual(self.check()['states']['DONE'], 1)
        finally:
            (self.root / 'evidence.txt').write_bytes(original)

    def test_historical_source_proof_survives_later_path_removal(self):
        self.done()
        original = (self.root / 'evidence.txt').read_bytes()
        try:
            (self.root / 'evidence.txt').unlink()
            self.assertEqual(self.check()['states']['DONE'], 1)
        finally:
            (self.root / 'evidence.txt').write_bytes(original)

    def test_done_requires_explicit_released_record_even_without_previous_local_lease(self):
        self.done()
        self.item['lease'] = None
        self.reject()

    def test_new_metadata_fields_are_required(self):
        del self.plan['continuity']
        self.reject()

    def test_unknown_metadata_fields_rejected(self):
        self.plan['silent_override'] = True
        self.reject()

    def test_global_compiler_capacity_cannot_expand(self):
        self.plan['resources']['global_max_compile_jobs'] = 3
        self.reject()

    def test_project_allocation_cannot_exceed_shared_capacity(self):
        self.plan['resources']['max_compile_jobs'] = 3
        self.reject()

    def test_zero_allocation_accepts_zero_compiler_leases(self):
        self.assertEqual(self.plan['resources']['max_compile_jobs'], 0)
        self.check()

    def test_zero_allocation_rejects_even_a_valid_owned_compiler_lease(self):
        self.active()
        self.plan['resources']['compiler_leases'] = [{'holder': 'test-owner', 'item': self.iid, 'jobs': 1}]
        self.reject()

    def test_compiler_lease_requires_active_owned_item(self):
        self.plan['resources']['max_compile_jobs'] = 1
        self.plan['resources']['compiler_leases'] = [{'holder': 'test-owner', 'item': self.iid, 'jobs': 1}]
        self.reject()

    def test_compiler_lease_wrong_holder_rejected(self):
        self.active()
        self.plan['resources']['max_compile_jobs'] = 1
        self.plan['resources']['compiler_leases'] = [{'holder': 'other-owner', 'item': self.iid, 'jobs': 1}]
        self.reject()

    def test_valid_single_compiler_lease_is_metadata_only(self):
        self.active()
        self.plan['resources']['max_compile_jobs'] = 1
        self.plan['resources']['compiler_leases'] = [{'holder': 'test-owner', 'item': self.iid, 'jobs': 1}]
        self.check()  # No compiler invocation occurs in this format test.

    def test_duplicate_compiler_lease_rejected(self):
        self.active()
        self.plan['resources']['max_compile_jobs'] = 2
        ref = {'holder': 'test-owner', 'item': self.iid, 'jobs': 1}
        self.plan['resources']['compiler_leases'] = [ref, copy.deepcopy(ref)]
        self.reject()

    def test_per_process_parallel_jobs_rejected(self):
        self.plan['resources']['per_process_jobs'] = 2
        self.reject()

    def test_boolean_job_count_rejected(self):
        self.plan['resources']['max_compile_jobs'] = True
        self.reject()

    def test_future_wakeup_cannot_be_promised_from_unsupported_metadata(self):
        self.plan['continuity']['future_wakeup_guaranteed'] = True
        self.plan['continuity']['scheduler_report']['automation_created'] = True
        self.reject()

    def test_scheduler_workaround_is_not_an_authorized_substitution(self):
        self.plan['continuity']['scheduler_report']['workaround_created'] = True
        self.reject()

    def test_unknown_item_owner_rejected(self):
        self.item['owner_lane'] = 'replacement-writer'
        self.reject()

    def test_unknown_area_owner_rejected(self):
        self.area['owner_lane'] = 'replacement-writer'
        self.reject()

    def test_bounded_implementation_needs_recorded_source(self):
        self.area['status']['implemented'] = 'BOUNDED'
        self.reject()

    def test_bounded_rust_tested_needs_passed_native_evidence(self):
        self.area['status']['rust_tested'] = 'BOUNDED'
        self.reject()

    def test_complete_without_area_inventory_closure_rejected(self):
        self.area['status']['implemented'] = 'COMPLETE'
        self.area['completion_evidence'] = [self.ref()]
        self.reject()

    def test_inventory_flag_without_source_closure_proof_rejected(self):
        self.area['inventory_complete'] = True
        self.area['unclosed_surface'] = []
        self.reject()

    def test_global_inventory_flag_cannot_hide_open_areas(self):
        self.plan['denominator']['inventory_complete'] = True
        self.reject()

    def test_timezone_free_lease_timestamp_rejected(self):
        self.active()
        self.item['lease']['recorded_at_utc'] = '2026-10-22T00:00:00'
        self.reject()

    def test_malformed_enum_types_rejected_cleanly(self):
        self.area['status']['implemented'] = []
        self.reject()

    def test_control_characters_in_writable_path_rejected(self):
        self.item['writable_boundary'] = ['rust/hidden\npath']
        self.reject()

    def test_optimization_keeps_resource_and_lease_checks(self):
        self.active()
        self.item['lease']['holder'] = 'other-owner'
        path = self.root / 'negative-plan.json'
        path.write_text(json.dumps(self.plan))
        result = subprocess.run([sys.executable, '-O', str(SCRIPT), '--root', str(self.root), '--plan', str(path)],
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
        self.assertEqual(result.returncode, 1)
        self.assertIn(b'EXECUTION BACKLOG INVALID', result.stderr)
        self.assertNotIn(b'Traceback', result.stderr)

    def test_previous_held_lease_cannot_disappear_when_item_becomes_blocked(self):
        self.active()
        previous = copy.deepcopy(self.plan)
        self.blocked()
        self.item['lease'] = None
        self.check()  # Snapshot alone cannot establish whether ownership was released.
        with self.assertRaises(gate.Invalid):
            gate.validate_transition(previous, self.plan)

    def test_previous_held_item_cannot_be_deleted(self):
        self.active()
        previous = copy.deepcopy(self.plan)
        del self.plan['items'][self.iid]
        with self.assertRaises(gate.Invalid):
            gate.validate_transition(previous, self.plan)

    def test_previous_held_owner_cannot_be_replaced(self):
        self.active()
        previous = copy.deepcopy(self.plan)
        self.item['owner_lane'] = 'other-owner'
        self.item['lease']['holder'] = 'other-owner'
        self.check()
        with self.assertRaises(gate.Invalid):
            gate.validate_transition(previous, self.plan)

    def test_previous_held_scope_cannot_be_expanded(self):
        self.active()
        previous = copy.deepcopy(self.plan)
        self.item['writable_boundary'] = ['rust']
        self.item['lease']['writable_boundary'] = ['rust']
        self.check()
        with self.assertRaises(gate.Invalid):
            gate.validate_transition(previous, self.plan)

    def test_unchanged_blocked_hold_persists_across_snapshots(self):
        self.active()
        previous = copy.deepcopy(self.plan)
        self.blocked()
        self.check()
        self.assertEqual(gate.validate_transition(previous, self.plan), 1)

    def test_explicit_release_acknowledgement_is_a_valid_transition(self):
        self.active()
        previous = copy.deepcopy(self.plan)
        self.done()
        self.item['lease']['status'] = 'RELEASED'
        self.item['lease']['acknowledgement'] = 'Synthetic explicit release'
        self.check()
        self.assertEqual(gate.validate_transition(previous, self.plan), 1)

    def test_status_flip_without_explicit_release_acknowledgement_rejected(self):
        self.active()
        previous = copy.deepcopy(self.plan)
        self.done()
        self.item['lease']['status'] = 'RELEASED'
        self.check()
        with self.assertRaises(gate.Invalid):
            gate.validate_transition(previous, self.plan)

    def test_acknowledged_release_then_reassignment_is_valid(self):
        self.active()
        self.item['lease']['status'] = 'RELEASED'
        self.item['lease']['acknowledgement'] = 'Synthetic explicit release'
        self.item['state'] = 'READY'
        previous = copy.deepcopy(self.plan)
        self.item['owner_lane'] = 'other-owner'
        self.active()
        self.check()
        self.assertEqual(gate.validate_transition(previous, self.plan), 0)

    def test_previous_plan_identity_mismatch_rejected(self):
        previous = copy.deepcopy(self.plan)
        previous['plan_id'] = 'OTHER-PLAN'
        with self.assertRaises(gate.Invalid):
            gate.validate_transition(previous, self.plan)

    def test_cli_previous_snapshot_protects_held_lease_under_optimization(self):
        self.active()
        previous = self.root / 'previous-plan.json'
        previous.write_text(json.dumps(self.plan))
        self.blocked()
        self.item['lease'] = None
        current = self.root / 'current-plan.json'
        current.write_text(json.dumps(self.plan))
        result = subprocess.run([sys.executable, '-O', str(SCRIPT), '--root', str(self.root),
                                 '--plan', str(current), '--previous-plan', str(previous)],
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
        self.assertEqual(result.returncode, 1)
        self.assertIn(b'held lease silently deleted', result.stderr)

    def test_lease_utc_comparison_accepts_later_fractional_timestamp(self):
        self.active()
        previous = copy.deepcopy(self.plan)
        self.item['lease']['recorded_at_utc'] = '2026-10-22T00:00:00.100000+00:00'
        self.assertEqual(gate.validate_transition(previous, self.plan), 1)

    def test_lease_utc_cannot_move_backwards(self):
        self.active()
        previous = copy.deepcopy(self.plan)
        self.item['lease']['recorded_at_utc'] = '2026-10-21T23:59:59Z'
        with self.assertRaises(gate.Invalid):
            gate.validate_transition(previous, self.plan)


if __name__ == '__main__':
    unittest.main()
