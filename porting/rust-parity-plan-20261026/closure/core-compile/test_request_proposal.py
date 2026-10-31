"""Public fixed observations; stdlib-only tests never acquire or select artifacts."""
from pathlib import Path
import copy
import hashlib
import importlib.util
import json
import unittest
from unittest.mock import patch

SOURCE = Path(__file__).resolve().parent / 'request_proposal.py'
EVIDENCE = Path(__file__).resolve().parents[3] / 'rust-parity-evidence/C01'
FRONTIER = EVIDENCE / 'batch44-request-proposal/reference/models-02-frontier.json'
TEMPLATE = EVIDENCE / 'batch36-core-metadata/requests.json'
EXPECTED = [
    'org.apache:apache:23',
    'org.apache:apache:31',
    'org.apache:apache:39',
    'org.apache.maven:maven-parent:10',
    'org.apache.maven:maven-parent:16',
    'org.apache.maven:maven-parent:23',
    'org.apache.maven:maven-parent:44',
    'org.codehaus.plexus:plexus:25',
    'org.eclipse.sisu:sisu-plexus:0.9.0.M2',
    'org.junit:junit-bom:5.11.1',
    'org.junit:junit-bom:5.11.4',
    'org.junit:junit-bom:5.12.2',
    'org.junit:junit-bom:5.13.1',
    'org.junit:junit-bom:5.14.4',
    'org.junit:junit-bom:5.9.3',
    'org.slf4j:slf4j-bom:2.0.18',
    'org.sonatype.forge:forge-parent:10',
    'org.sonatype.forge:forge-parent:5']

SPEC = importlib.util.spec_from_file_location('fixed_model_proposals', SOURCE)
proposal = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(proposal)


class ProposalTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.raw_frontier, cls.raw_template = FRONTIER.read_bytes(), TEMPLATE.read_bytes()
        cls.frontier, cls.template = json.loads(cls.raw_frontier), json.loads(cls.raw_template)

    def setUp(self):
        block = patch('socket.create_connection', side_effect=AssertionError('network forbidden'))
        block.start()
        self.addCleanup(block.stop)

    def build(self):
        return proposal.propose('models-02', self.raw_frontier, self.raw_template)

    def deny_frontier(self, change, message):
        value = copy.deepcopy(self.frontier)
        change(value)
        raw = json.dumps(value, separators=(',', ':')).encode()
        fixed = copy.deepcopy(proposal.EPOCHS)
        fixed['models-02'].update(frontier_bytes=len(raw), frontier_sha256=hashlib.sha256(raw).hexdigest())
        # Test-only outer-envelope repin; independent primary authority is unchanged.
        with patch.object(proposal, 'EPOCHS', fixed), self.assertRaisesRegex(ValueError, message):
            proposal.propose('models-02', raw, self.raw_template)

    def test_actual_public_fixture_and_existing_main_template_are_pinned(self):
        self.assertEqual(hashlib.sha256(self.raw_frontier).hexdigest(),
                         'cfe1c1f36f4f9d438d7840e9798b15dc21bbafa3da3c9af50a0ec68215de769f')
        self.assertEqual(hashlib.sha256(self.raw_template).hexdigest(),
                         'd4c51a84aa264eaa057d42f4d208744573520085dca116103b874790c8fb445d')
        self.assertEqual((len(self.raw_frontier), len(self.raw_template)), (28714, 203892))
        self.assertEqual(len(self.raw_frontier.splitlines()), 19)
        self.assertEqual(set(proposal.EPOCHS), {'models-02'})

    def test_independent18_gav_order_and_canonical_urls(self):
        result = self.build()
        gavs = [':'.join(p['coordinate'][k] for k in ['groupId', 'artifactId', 'version'])
                for p in result['new_primaries']]
        self.assertEqual(gavs, EXPECTED)
        for index, gav in enumerate(EXPECTED):
            group, artifact, version = gav.split(':')
            relative = group.replace('.', '/') + '/' + artifact + '/' + version + '/' + artifact + '-' + version + '.pom'
            primary = result['new_primaries'][index]
            self.assertEqual(primary['relative_artifact'], relative)
            self.assertEqual(primary['url'], 'https://repo.maven.apache.org/maven2/' + relative)
            self.assertEqual((primary['kind'], primary['primary_index'], primary['allowed_status']), ('pom', index, [200]))

    def test_all90_ordered_sidecars_roles_statuses_bounds_and_uniqueness(self):
        result = self.build()
        rows = result['requests']
        self.assertEqual(len(rows), 90)
        self.assertEqual((len({r['url'] for r in rows}), len({r['relative_path'] for r in rows})), (90, 90))
        self.assertEqual([sum(r['role'] == role for r in rows) for role in ['pom', 'checksum', 'signature']], [18, 54, 18])
        for i in range(18):
            chunk = rows[i * 5:i * 5 + 5]
            self.assertEqual([r['url'][len(chunk[0]['url']):] for r in chunk], ['', '.sha512', '.sha256', '.sha1', '.asc'])
            self.assertEqual([r['primary_index'] for r in chunk], [i] * 5)
            self.assertEqual([r.get('algorithm') for r in chunk], [None, 'sha512', 'sha256', 'sha1', None])
            for j, row in enumerate(chunk):
                relative = row['url'].removeprefix(proposal.BASE)
                self.assertEqual(row['relative_path'], 'downloads/' + relative + '.partial')
                self.assertEqual(row['allowed_status'], [200] if j == 0 else [200, 404])
                self.assertTrue(all(type(s) is int for s in row['allowed_status']))
                self.assertEqual(row['maximum_bytes'], 2 * proposal.MIB if j == 0 else 65536)
                self.assertLessEqual(len(proposal.encoded(row)), 4096)

    def test_target_and_every_profile_field_unchanged_except_explicit_counts(self):
        result = self.build()
        expected = copy.deepcopy(self.template['limits'])
        expected.update(exact_primaries=18, exact_requests=90)
        self.assertEqual(result['limits'], expected)
        self.assertTrue(all(type(v) is int for v in result['limits'].values()))
        self.assertEqual(result['target'], self.template['target'])
        self.assertEqual(result['schema'], 'C01-NORMAL-CORE-CLOSURE-REQUESTS/1')
        self.assertEqual(result['epoch'], 'models-02')

    def test_full_frontier_reuses_roots_all214_ranges_and_remaining_gates_survive(self):
        result = self.build()
        self.assertEqual(result['observed_frontier'], self.frontier)
        self.assertEqual(len(result['observed_frontier']['inventory_reconciliation']['four_reuses']), 4)
        ranges = result['observed_frontier']['six_range_worklists']
        self.assertEqual((ranges['sets'], sum(len(r['versions']) for r in ranges['rows'])), (6, 214))
        self.assertFalse(ranges['winner_selected'])
        self.assertEqual(result['observed_frontier']['scope_preserved']['original9'], self.frontier['scope_preserved']['original9'])
        self.assertEqual(result['remaining_gates'], self.template['remaining_gates'] + self.frontier['next_actions'])
        self.assertFalse(any(result['authorization'].values()))
        self.assertFalse(result['target']['execution_authorized'])

    def test_six_bom_rows_stay_conditional_and_exact_source_contexts_survive(self):
        result = self.build()
        imports = [p for p in result['new_primaries'] if 'conditional_local_import_observation' in p['categories']]
        self.assertEqual(len(imports), 6)
        for original, primary in zip(self.frontier['next_finite_epoch']['rows'], result['new_primaries']):
            self.assertEqual(primary['disposition'], original['status'])
            self.assertEqual(len(primary['provenance']), len(original['evidence']))
            for evidence, observed in zip(original['evidence'], primary['provenance']):
                source, selector, raw_version, properties, contexts = evidence
                self.assertEqual(observed['binding'], self.frontier['source_index']['models'][source])
                self.assertEqual((observed['source_id'], observed['xml_path'], observed['raw_version'],
                                  observed['exact_local_properties']), (source, selector, raw_version, properties))
                self.assertEqual(observed['contexts'], [self.frontier['source_index']['contexts'][i] for i in contexts])
                self.assertEqual(observed['disposition'], original['status'])
        self.assertTrue(all(p['disposition'] == 'LOCAL_IMPORT_CANDIDATE_EFFECTIVE_PROPERTY_PROFILE_PROOF_OPEN' for p in imports))

    def test_nominal_two_escaped_metadata_copies_and_body_refusal_not_completion(self):
        result = self.build()
        charge = result['nominal_metadata_charge']
        self.assertEqual(charge['two_escaped_response_copies_bytes'], 19353600)
        self.assertEqual(charge['getter_nominal_bytes'], 21450752)
        self.assertEqual(charge['static_and_failure_reserve'], 2 * proposal.MIB)
        self.assertLess(charge['getter_nominal_bytes'], 63 * proposal.MIB)
        body = result['body_bound_note']
        self.assertEqual(body['sum_individual_caps_bytes'], 42467328)
        self.assertEqual(body['aggregate_refusal_cap_bytes'], 16 * proposal.MIB)
        self.assertFalse(body['completion_within_aggregate_promised'])

    def test_deterministic_json_bounded_exact_input_provenance_and_private_mutation(self):
        first, second = self.build(), self.build()
        self.assertEqual(proposal.encoded(first), proposal.encoded(second))
        self.assertLessEqual(len(proposal.encoded(first)), 256 * 1024)
        self.assertEqual(first['source_bindings'], [
            {'role': 'frozen_frontier', 'sha256': hashlib.sha256(self.raw_frontier).hexdigest(), 'bytes': 28714},
            {'role': 'unchanged_main_template', 'sha256': hashlib.sha256(self.raw_template).hexdigest(), 'bytes': 203892}])
        first['observed_frontier']['scope_preserved']['plugins'].clear()
        self.assertEqual(self.build()['observed_frontier'], self.frontier)

    def test_unknown_stale_nonbytes_and_oversize_refuse_before_parse_or_copy(self):
        cases = [('models-03', self.raw_frontier, self.raw_template), (None, self.raw_frontier, self.raw_template),
                 ('models-02', bytearray(self.raw_frontier), self.raw_template),
                 ('models-02', b'x' * 65537, self.raw_template), ('models-02', self.raw_frontier + b'\n', self.raw_template),
                 ('models-02', b' ' + self.raw_frontier[1:], self.raw_template),
                 ('models-02', self.raw_frontier, b' ' + self.raw_template[1:]),
                 ('models-02', self.raw_frontier, memoryview(self.raw_template)),
                 ('models-02', self.raw_frontier, b'x' * 262145), ('models-02', self.raw_frontier, self.raw_template + b'\n')]
        with patch.object(proposal.json, 'loads', side_effect=AssertionError('premature parse')), \
             patch.object(proposal.copy, 'deepcopy', side_effect=AssertionError('premature copy')):
            for args in cases:
                with self.subTest(epoch=args[0]), self.assertRaises(ValueError):
                    proposal.propose(*args)

    def test_coherent_coordinate_replacement_duplicate_and_order_fail_independent_seal(self):
        def replaced(value):
            row = value['next_finite_epoch']['rows'][0]
            row['GAV'], row['url'] = 'org.apache:apache:24', proposal.BASE + 'org/apache/apache/24/apache-24.pom'
        self.deny_frontier(replaced, 'independent fixed')
        self.deny_frontier(lambda f: f['next_finite_epoch']['rows'].reverse(), 'independent fixed')
        self.deny_frontier(lambda f: f['next_finite_epoch']['rows'].__setitem__(1, copy.deepcopy(f['next_finite_epoch']['rows'][0])), 'independent fixed')

    def test_canonical_url_role_conditionality_and_count_guards_are_independent(self):
        for change, message in [
            (lambda f: f['next_finite_epoch']['rows'][0].update(kind='jar'), 'POM URL'),
            (lambda f: f['next_finite_epoch']['rows'][0].update(url='http://unadmitted/other.pom'), 'POM URL'),
            (lambda f: f['next_finite_epoch']['rows'][0].update(GAV='org.apache:..:23'), 'canonical fixed'),
            (lambda f: f['next_finite_epoch']['rows'][9].update(status='EFFECTIVE_REQUIRED_IMPORT'), 'conditional observation'),
            (lambda f: f['next_finite_epoch'].update(primaries=18.0), 'strict frontier count'),
            (lambda f: f['next_finite_epoch']['rows'].pop(), 'fixed18'),
            (lambda f: f.update(schema='unknown'), 'frontier schema')]:
            self.deny_frontier(change, message)

    def test_template_strict_profile_cannot_be_coherently_relaxed(self):
        for value in [True, 16777216.0, 128 * proposal.MIB]:
            template = copy.deepcopy(self.template)
            template['limits']['payload_and_sidecar_total_bytes'] = value
            raw = json.dumps(template, separators=(',', ':')).encode()
            raw += b' ' * (203892 - len(raw))
            with patch.object(proposal, 'TEMPLATE_SHA', hashlib.sha256(raw).hexdigest()):
                with self.assertRaisesRegex(ValueError, 'fixed resource profile'):
                    proposal.propose('models-02', self.raw_frontier, raw)

    def test_duplicate_json_and_registry_count_types_refuse(self):
        raw = b'{"schema":1,"schema":1}' + b' ' * (203892 - 23)
        with patch.object(proposal, 'TEMPLATE_SHA', hashlib.sha256(raw).hexdigest()):
            with self.assertRaisesRegex(ValueError, 'duplicate JSON key'):
                proposal.propose('models-02', self.raw_frontier, raw)
        fixed = copy.deepcopy(proposal.EPOCHS)
        fixed['models-02']['requests'] = 90.0
        with patch.object(proposal, 'EPOCHS', fixed), self.assertRaisesRegex(ValueError, 'strict finite epoch'):
            self.build()

    def test_output_overflow_refuses_without_input_bound_or_version_shortcut(self):
        self.deny_frontier(lambda f: f['next_finite_epoch']['rows'][0].update(
            evidence=f['next_finite_epoch']['rows'][0]['evidence'] * 400), 'proposal output byte limit')

    def test_pure_api_performs_no_filesystem_reads_and_never_imports_acquired_code(self):
        with patch('builtins.open', side_effect=AssertionError('filesystem forbidden')):
            result = self.build()
        self.assertFalse(result['authorization']['network'])
        self.assertFalse(result['authorization']['acquired_code_execution'])


if __name__ == '__main__':
    unittest.main()
