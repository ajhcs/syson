# SPDX-License-Identifier: EPL-2.0
"""Synthetic bundle tests using actual pinned source/metadata and existing archives.

The eight-byte artifact is an empty synthetic WASM module, never a built product.
"""
import copy
import json
import os
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest.mock import patch
import check
import make_fixture

ROOT = Path(__file__).resolve().parents[2]
TOOL = Path(__file__).resolve().parent
CAPTURE = ROOT / 'porting/follow-ons/F43/metadata/capture.json'
POLICY = TOOL / 'policy.json'
CACHE = Path(os.environ.get('SYSON_REGISTRY_CACHE', '/workspace/cache/builderbob/global-tools/cargo/registry/cache'))


class BundleChecks(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        base = Path(os.environ['SYSON_TEST_DIR'])
        if not base.is_dir() or not base.resolve().is_relative_to(Path('/workspace')):
            raise RuntimeError('SYSON_TEST_DIR must name an existing admitted workspace directory')
        cls.scratch = tempfile.TemporaryDirectory(prefix='f43-', dir=base)
        cls.tmp = Path(cls.scratch.name)
        cls.policy = json.loads(POLICY.read_bytes())
        for profile in ['default', 'candidate']:
            make_fixture.make(ROOT, cls.tmp / profile, CAPTURE, POLICY, profile, CACHE)

    @classmethod
    def tearDownClass(cls):
        cls.scratch.cleanup()

    def setUp(self):
        self.package = self.tmp / self.id().split('.')[-1]
        shutil.copytree(self.tmp / 'default', self.package)

    def verify(self, profile='default', source=ROOT, package=None, capture=CAPTURE):
        return check.verify(source, package or self.package, capture, profile, POLICY)

    def rejected(self, operation, message):
        with self.assertRaisesRegex(check.VerificationError, message):
            operation()

    def test_default_bundle_has_actual_notices_sources_and_archives(self):
        result = self.verify()
        self.assertEqual(len(result['active_local_packages']), 5)
        self.assertEqual(len(result['active_registry_packages']), 5)
        self.assertTrue(result['synthetic_artifact_declared'])
        self.assertEqual(result['artifact_build_correspondence'], 'UNPROVEN')
        self.assertEqual(result['full_legal_clearance'], 'NOT_ESTABLISHED')
        self.assertFalse(result['build_executed_by_verifier'])

    def test_candidate_profile_includes_optional_path_dependencies(self):
        result = self.verify('candidate', package=self.tmp / 'candidate')
        self.assertEqual(len(result['active_local_packages']), 9)
        self.assertIn('syson-initializer-graph', result['active_local_packages'])
        self.assertIn('syson-workflow', result['active_local_packages'])

    def test_missing_notice_is_not_satisfied_by_source_notice(self):
        (self.package / 'NOTICE').unlink()
        self.rejected(self.verify, 'missing required file: NOTICE')

    def test_altered_license_is_rejected(self):
        p = self.package / 'rust/third-party/serde_json-1.0.151/LICENSE-MIT'
        p.write_bytes(p.read_bytes() + b'changed')
        self.rejected(self.verify, 'altered or mismatched file')

    def test_packaged_lock_mismatch_is_rejected(self):
        p = self.package / 'source/rust/crates/syson-web/Cargo.lock'
        p.write_bytes(p.read_bytes().replace(b'1.0.151', b'1.0.150'))
        self.rejected(self.verify, 'altered or mismatched file')

    def test_packaged_source_change_is_rejected(self):
        p = self.package / 'source/rust/crates/syson-web/src/lib.rs'
        p.write_bytes(p.read_bytes() + b'// mismatched source')
        self.rejected(self.verify, 'altered or mismatched file')

    def copied_source(self):
        source = self.package / 'copied-source'
        for path, h in self.policy['source_sha256'].items():
            dest = source / path
            dest.parent.mkdir(parents=True, exist_ok=True)
            dest.write_bytes(check.checked_bytes(ROOT, path, h))
        return source

    def test_current_source_lock_change_rejects_old_capture(self):
        source = self.copied_source()
        p = source / 'rust/crates/syson-web/Cargo.lock'
        p.write_bytes(p.read_bytes() + b'changed')
        self.rejected(lambda: self.verify(source=source), 'altered or mismatched file')

    def test_extra_unpinned_active_crate_source_is_rejected(self):
        source = self.copied_source()
        (source / 'rust/crates/syson-web/src/extra.rs').write_text('// new source')
        self.rejected(lambda: self.verify(source=source), 'unpinned or missing local source file')

    def test_registry_archive_checksum_is_verified(self):
        p = self.package / 'dependency-archives/itoa-1.0.18.crate'
        p.write_bytes(p.read_bytes() + b'changed')
        self.rejected(self.verify, 'altered or mismatched file')

    def test_registry_manifest_checksum_is_verified(self):
        p = self.package / 'dependency-manifests/itoa-1.0.18/Cargo.toml'
        p.write_bytes(p.read_bytes() + b'changed')
        self.rejected(self.verify, 'altered or mismatched file')

    def test_unknown_active_registry_dependency_is_not_admitted_by_lock_presence(self):
        metadata = json.loads((CAPTURE.parent / 'default.json').read_bytes())
        selected, _ = check.resolved_graph(metadata, 'default', self.policy)
        manifest = selected[metadata['resolve']['root']]['manifest_path']
        captured_root = manifest[:-len('/' + self.policy['root_manifest'])]
        unknown = copy.deepcopy(next(p for p in selected.values() if p['name'] == 'itoa'))
        unknown.update(id='synthetic-unit-input:serde', name='serde', version='1.0.229')
        selected[unknown['id']] = unknown
        # Isolate admission logic; never fabricate a successful Cargo capture.
        with patch.object(check, 'capture_inputs', return_value=(selected, captured_root)):
            self.rejected(self.verify, 'unknown active registry dependency: serde-1.0.229')

    def test_root_feature_mismatch_is_rejected(self):
        metadata = json.loads((CAPTURE.parent / 'default.json').read_bytes())
        node = next(n for n in metadata['resolve']['nodes'] if n['id'] == metadata['resolve']['root'])
        node['features'].append('raw-factory-candidate')
        self.rejected(lambda: check.resolved_graph(metadata, 'default', self.policy), 'root feature profile mismatch')

    def test_missing_graph_target_is_rejected(self):
        metadata = json.loads((CAPTURE.parent / 'default.json').read_bytes())
        node = next(n for n in metadata['resolve']['nodes'] if n['id'] == metadata['resolve']['root'])
        node['deps'].append({'pkg': 'synthetic-missing-node', 'dep_kinds': [{'kind': None, 'target': None}]})
        self.rejected(lambda: check.resolved_graph(metadata, 'default', self.policy), 'unresolved graph node')

    def test_capture_manifest_tampering_is_rejected(self):
        capture = self.package / 'tampered-capture.json'
        capture.write_bytes(CAPTURE.read_bytes() + b' ')
        self.rejected(lambda: self.verify(capture=capture), 'capture manifest does not match reviewed policy')

    def test_extra_packaged_payload_is_rejected(self):
        (self.package / 'unknown.dll').write_bytes(b'not admitted')
        self.rejected(self.verify, 'unknown or missing packaged files')

    def test_symlink_to_source_license_does_not_satisfy_package_inclusion(self):
        p = self.package / 'LICENSE'
        p.unlink()
        p.symlink_to(ROOT / 'LICENSE')
        self.rejected(self.verify, 'symlink is not admitted')

    def test_wrong_source_revision_is_rejected(self):
        p = self.package / 'distribution.json'
        d = json.loads(p.read_bytes())
        d['source_revision'] = '0' * 40
        p.write_text(json.dumps(d))
        self.rejected(self.verify, 'package source revision mismatch')

    def test_artifact_digest_agreement_is_required(self):
        p = self.package / 'web/syson_web.wasm'
        p.write_bytes(p.read_bytes() + b'changed')
        self.rejected(self.verify, 'altered or mismatched file')

    def test_fixture_builder_refuses_existing_output(self):
        with self.assertRaises(FileExistsError):
            make_fixture.make(ROOT, self.package, CAPTURE, POLICY, 'default', CACHE)


if __name__ == '__main__':
    unittest.main()
