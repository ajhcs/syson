#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Reproduce the fixed-source policy; read-only Git/cache inputs, no Cargo/network."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tomllib

PIN = 'ad814c232ee24109639c1d52239cbeec6f4a3e92'


def prepare(source, capture, output, registry_source):
    source, capture, output = Path(source), Path(capture), Path(output)
    registry_source = Path(registry_source).resolve()
    def blob(path):
        return subprocess.check_output(['git', '-C', str(source), 'show', PIN + ':' + path])
    def sha(raw):
        return hashlib.sha256(raw).hexdigest()
    records = subprocess.check_output(['git', '-C', str(source), 'ls-tree', '-rz', PIN]).split(b'\0')
    files = {}
    for record in records:
        if not record:
            continue
        _, name = record.split(b'\t', 1)
        path = name.decode()
        if path.startswith(('rust/crates/', 'rust/third-party/')) or path in ['LICENSE', 'LICENSE-LGPL', 'NOTICE', 'rust/THIRD-PARTY-NOTICES.md']:
            files[path] = sha(blob(path))
    license_rows = json.loads(blob('rust/third-party/manifest.json'))
    lock = tomllib.loads(blob('rust/crates/syson-web/Cargo.lock').decode())
    identities = {p['name'] + '-' + p['version']: p for p in lock['package']}
    registry = {p['crate']: {**p, 'source': identities[p['crate']]['source'], 'checksum': identities[p['crate']]['checksum']} for p in license_rows}
    policy = {'schema_version': 1, 'source_revision': PIN, 'root_package': 'syson-web',
              'root_manifest': 'rust/crates/syson-web/Cargo.toml', 'root_lock': 'rust/crates/syson-web/Cargo.lock',
              'target': 'wasm32-unknown-unknown', 'profiles': {'default': [], 'candidate': ['raw-factory-candidate']},
              'source_sha256': files, 'registry': registry, 'capture_sha256': sha(capture.read_bytes()),
              'runtime_assets': {'web/index.html': 'rust/crates/syson-web/web/index.html',
                                 'web/loader.js': 'rust/crates/syson-web/web/loader.js',
                                 'web/style.css': 'rust/crates/syson-web/web/style.css'}, 'artifact_path': 'web/syson_web.wasm'}
    if json.loads(capture.read_bytes())['source_revision'] != PIN:
        raise ValueError('capture revision differs from pinned source')
    output.mkdir(parents=True, exist_ok=True)
    fixtures = output / 'fixtures/registry-manifests'
    fixtures.mkdir(parents=True, exist_ok=True)
    for item in json.loads((capture.parent / 'default.json').read_bytes())['packages']:
        key = item['name'] + '-' + item['version']
        if key not in registry:
            continue
        path = Path(item['manifest_path']).resolve()
        if not path.is_relative_to(registry_source) or path.name != 'Cargo.toml':
            raise ValueError('manifest outside explicit registry source root')
        raw = path.read_bytes()
        if sha(raw) != registry[key]['source_manifest_sha256']:
            raise ValueError('admitted registry manifest hash mismatch: ' + key)
        (fixtures / (key + '.toml')).write_bytes(raw)
    (output / 'policy.json').write_text(json.dumps(policy, indent=2) + '\n')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    for flag in ['source', 'capture', 'output', 'registry-source']:
        parser.add_argument('--' + flag, required=True, type=Path)
    args = parser.parse_args()
    prepare(args.source, args.capture, args.output, args.registry_source)
