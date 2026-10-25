# SPDX-License-Identifier: EPL-2.0
"""Source-drift and false-acceptance regressions; no HTTP/security model."""
import copy
import hashlib
import json
from pathlib import Path
import unittest

from check_contract import (CASES, GAPS, Refusal, STATUS, SOURCES, pairs, projection,
                            read_json, validate_contract, verify_sources)

BASE = Path(__file__).resolve().parent
ROOT = BASE.parents[3]


class ContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.manifest = read_json(BASE / 'source-manifest.json')
        cls.sources = verify_sources(ROOT, cls.manifest)
        cls.contract = read_json(BASE / 'contract.json')
        cls.cases = read_json(BASE / 'negative-scenarios.json')
        cls.bytes = {p: (ROOT / p).read_bytes() for p in SOURCES.values()}

    def packet(self):
        return copy.deepcopy(self.contract), copy.deepcopy(self.cases)

    def validate(self, contract=None, scenarios=None):
        return validate_contract(contract or self.contract, scenarios or self.cases, self.sources)

    def test_source_preparation_pass_never_claims_runtime_acceptance(self):
        result = self.validate()
        self.assertEqual(result['qualification'], STATUS)
        self.assertEqual(result['native_runtime'], 'NOT_RUN')
        self.assertIsNone(result['host_adapter'])
        self.assertEqual((result['source_pins'], result['negative_scenarios']), (18, 12))

    def test_all_sources_equal_exact_upstream_git_blobs(self):
        self.assertEqual(set(self.sources), set(SOURCES))
        for row in self.manifest['evidence']:
            self.assertEqual(hashlib.sha256(self.bytes[row['path']]).hexdigest(), row['sha256'])

    def test_inherited_versions_and_cors_are_declarations(self):
        p = projection(self.sources)
        self.assertEqual(p['source_version'], '2026.9.3')
        self.assertEqual(p['spring_boot_parent'], '4.1.1')
        self.assertEqual(p['starter_version_expression'], '${sirius.web.version}')
        self.assertEqual(p['frontend_application_version'], '2026.9.3')
        self.assertEqual(p['properties']['sirius.components.cors.allowedOriginPatterns'], '*')
        self.assertEqual(p['dev-properties']['sirius.components.cors.allowedCredentials'], 'true')

    def test_duplicate_json_fields_are_not_silently_overwritten(self):
        with self.assertRaises(Refusal):
            json.loads('{"status":"BLOCKED","status":"QUALIFIED"}', object_pairs_hook=pairs)

    def test_wrong_commit_is_refused_before_reading(self):
        packet = copy.deepcopy(self.manifest)
        packet['upstream_commit'] = '0' * 40
        with self.assertRaises(Refusal):
            verify_sources(ROOT, packet, lambda _: self.fail('unexpected source read'))

    def test_source_paths_are_fixed_and_relative(self):
        for name in ['/etc/passwd', '../../outside', 'pom.xml/../elsewhere']:
            with self.subTest(name=name):
                p = copy.deepcopy(self.manifest)
                p['evidence'][0]['path'] = name
                with self.assertRaises(Refusal):
                    verify_sources(ROOT, p, lambda _: self.fail('unexpected source read'))

    def test_source_omission_or_duplicate_is_refused(self):
        for mode in ['omit', 'duplicate']:
            p = copy.deepcopy(self.manifest)
            if mode == 'omit':
                p['evidence'].pop()
            else:
                p['evidence'][1] = p['evidence'][0]
            with self.subTest(mode=mode), self.assertRaises(Refusal):
                verify_sources(ROOT, p, self.bytes.__getitem__)

    def test_configuration_source_drift_is_refused(self):
        data = dict(self.bytes)
        path = SOURCES['properties']
        data[path] = data[path].replace(b'allowedCredentials=true', b'allowedCredentials=false')
        with self.assertRaises(Refusal):
            verify_sources(ROOT, self.manifest, data.__getitem__)

    def test_anchor_text_and_ranges_are_checked(self):
        for field, value in [('text', 'invented security conclusion'), ('first_line', 0), ('last_line', 1)]:
            p = copy.deepcopy(self.manifest)
            p['evidence'][0]['anchors'][0][field] = value
            with self.subTest(field=field), self.assertRaises(Refusal):
                verify_sources(ROOT, p, self.bytes.__getitem__)

    def test_stale_source_projection_is_refused(self):
        c, _ = self.packet()
        c['source_projection']['source_version'] = '2026.9.0'
        with self.assertRaises(Refusal):
            self.validate(c)

    def test_source_pass_cannot_promote_any_runtime_claim(self):
        for field in self.contract['claims']:
            c, _ = self.packet()
            c['claims'][field] = True
            with self.subTest(field=field), self.assertRaises(Refusal):
                self.validate(c)

    def test_source_pass_cannot_select_a_host_adapter(self):
        c, _ = self.packet()
        c['host_adapter'] = 'candidate-rust-server'
        with self.assertRaises(Refusal):
            self.validate(c)

    def test_preparation_status_cannot_be_relabelled(self):
        c, _ = self.packet()
        c['status'] = 'SECURITY_QUALIFIED'
        with self.assertRaises(Refusal):
            self.validate(c)

    def test_every_native_boundary_stays_unresolved(self):
        self.assertEqual({g['id'] for g in self.contract['unresolved']}, GAPS)
        for index in range(len(GAPS)):
            c, _ = self.packet()
            c['unresolved'][index]['status'] = 'VERIFIED'
            with self.subTest(index=index), self.assertRaises(Refusal):
                self.validate(c)

    def test_native_boundary_cannot_be_removed(self):
        c, _ = self.packet()
        c['unresolved'].pop()
        with self.assertRaises(Refusal):
            self.validate(c)

    def test_artifact_digests_cannot_be_fabricated(self):
        for field in ['jar_sha256', 'image_digest', 'effective_configuration_sha256', 'policy_source_sha256']:
            c, _ = self.packet()
            c['artifact'][field] = 'a' * 64
            with self.subTest(field=field), self.assertRaises(Refusal):
                self.validate(c)

    def test_unknown_source_citations_are_refused(self):
        c, s = self.packet()
        c['unresolved'][0]['source_ids'] = ['unknown-source']
        with self.assertRaises(Refusal):
            self.validate(c)
        s['cases'][0]['source_ids'] = []
        with self.assertRaises(Refusal):
            self.validate(scenarios=s)

    def test_required_negative_scenarios_cannot_be_removed_or_duplicated(self):
        self.assertEqual({c['id'] for c in self.cases['cases']}, CASES)
        for mode in ['omit', 'duplicate']:
            _, s = self.packet()
            if mode == 'omit':
                s['cases'].pop()
            else:
                s['cases'][1] = s['cases'][0]
            with self.subTest(mode=mode), self.assertRaises(Refusal):
                self.validate(scenarios=s)

    def test_scenario_execution_and_native_outcomes_are_unrun(self):
        for field, value in [('native_expected', 'DENIED'), ('receipt', {'http_status': 403})]:
            _, s = self.packet()
            s['cases'][0][field] = value
            with self.subTest(field=field), self.assertRaises(Refusal):
                self.validate(scenarios=s)
        _, s = self.packet()
        s['execution'] = 'PASS'
        with self.assertRaises(Refusal):
            self.validate(scenarios=s)

    def test_all_boundary_families_have_explicit_negative_scenarios(self):
        _, s = self.packet()
        self.assertEqual({g for c in s['cases'] for g in c['boundaries']}, GAPS)
        for c in s['cases']:
            c['boundaries'] = [b if b != 'csrf' else 'authentication' for b in c['boundaries']]
        with self.assertRaises(Refusal):
            self.validate(scenarios=s)

    def test_unknown_fields_do_not_add_unreviewed_claims(self):
        c, _ = self.packet()
        c['native_secure'] = True
        with self.assertRaises(Refusal):
            self.validate(c)


if __name__ == '__main__':
    unittest.main()
