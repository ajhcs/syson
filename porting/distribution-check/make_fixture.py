#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Build an explicitly synthetic bundle from pinned inputs and existing cached archives.

Never builds, downloads, installs, extracts or executes a dependency. The supplied
registry cache is read only; package output must be a fresh caller-owned directory.
"""
import argparse
import json
from pathlib import Path
import check


def make(source, output, capture, policy_path, profile, registry_cache):
    source, output, registry_cache = Path(source), Path(output), Path(registry_cache)
    policy = json.loads(Path(policy_path).read_bytes())
    selected, captured_root = check.capture_inputs(source, Path(capture), profile, policy)
    output.mkdir(parents=True, exist_ok=False)
    pins = policy['source_sha256']
    def put(dest, raw):
        path = output / dest
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(raw)
    for p in ['LICENSE', 'LICENSE-LGPL', 'NOTICE', 'rust/THIRD-PARTY-NOTICES.md', 'rust/third-party/manifest.json']:
        put(p, check.checked_bytes(source, p, pins[p]))
    put('source/SOURCE-REVISION', (policy['source_revision'] + '\n').encode())
    for item in selected.values():
        if item['source'] is None:
            rel = item['manifest_path'][len(captured_root) + 1:]
            prefix = str(Path(rel).parent) + '/'
            for path, h in pins.items():
                if path.startswith(prefix):
                    put('source/' + path, check.checked_bytes(source, path, h))
        else:
            label = item['name'] + '-' + item['version']
            row = policy['registry'][label]
            fixture = Path(__file__).parent / 'fixtures/registry-manifests' / (label + '.toml')
            raw = fixture.read_bytes()
            check.require(check.digest(raw) == row['source_manifest_sha256'], 'fixture manifest mismatch')
            put('dependency-manifests/' + label + '/Cargo.toml', raw)
            matches = list(registry_cache.glob('*/' + label + '.crate'))
            check.require(len(matches) == 1, 'supply one admitted archive for ' + label)
            archive = matches[0].read_bytes()
            check.require(check.digest(archive) == row['checksum'], 'cached archive differs from lock: ' + label)
            put('dependency-archives/' + label + '.crate', archive)
            for license_file in row['license_files']:
                p, h = license_file['path'], license_file['sha256']
                put(p, check.checked_bytes(source, p, h))
    for dest, origin in policy['runtime_assets'].items():
        put(dest, check.checked_bytes(source, origin, pins[origin]))
    artifact = b'\x00asm\x01\x00\x00\x00'
    put(policy['artifact_path'], artifact)
    descriptor = {'schema_version': 1, 'profile': profile, 'source_revision': policy['source_revision'],
                  'artifact': {'path': policy['artifact_path'], 'sha256': check.digest(artifact), 'synthetic': True}}
    put('distribution.json', (json.dumps(descriptor, indent=2) + '\n').encode())
    return check.verify(source, output, capture, profile, policy_path)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['source', 'output', 'capture', 'policy', 'registry-cache']:
        parser.add_argument('--' + name, required=True, type=Path)
    parser.add_argument('--profile', choices=['default', 'candidate'], required=True)
    args = parser.parse_args()
    print(json.dumps(make(args.source, args.output, args.capture, args.policy, args.profile, args.registry_cache), indent=2))
