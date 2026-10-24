#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Verify a supplied Rust/WASM source bundle against pinned, captured inputs.

No Cargo invocation, registry access, archive extraction, or build attestation.
The reviewed policy and captured metadata are explicit trust inputs.
"""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import tomllib


class VerificationError(ValueError):
    pass


def require(condition, message):
    if not condition:
        raise VerificationError(message)


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def safe_file(root, relative):
    p = PurePosixPath(relative)
    require(relative and not p.is_absolute() and str(p) == relative and all(x not in ('.', '..') for x in p.parts), 'unsafe package/source path: ' + relative)
    current = root
    for part in p.parts:
        current = current / part
        require(not current.is_symlink(), 'symlink is not admitted: ' + relative)
    require(current.is_file(), 'missing required file: ' + relative)
    return current


def checked_bytes(root, path, expected):
    raw = safe_file(root, path).read_bytes()
    require(digest(raw) == expected, 'altered or mismatched file: ' + path)
    return raw


def unique_index(rows, key):
    output = {row[key]: row for row in rows}
    require(len(output) == len(rows), 'duplicate metadata ' + key)
    return output


def resolved_graph(metadata, profile, policy):
    packages = unique_index(metadata['packages'], 'id')
    nodes = unique_index(metadata['resolve']['nodes'], 'id')
    root = metadata['resolve']['root']
    require(root in packages and root in nodes, 'missing resolved root')
    require(packages[root]['name'] == policy['root_package'], 'wrong graph root')
    require(sorted(nodes[root]['features']) == sorted(policy['profiles'][profile]), 'root feature profile mismatch')
    pending, selected = [root], {}
    while pending:
        key = pending.pop()
        if key in selected:
            continue
        require(key in packages and key in nodes, 'unresolved graph node: ' + key)
        package = packages[key]
        require(not any('proc-macro' in t['kind'] for t in package['targets']), 'proc-macro/runtime classification requires a new policy')
        selected[key] = package
        for edge in nodes[key]['deps']:
            require(edge['dep_kinds'], 'missing dependency kind')
            if any(k['kind'] is None for k in edge['dep_kinds']):
                pending.append(edge['pkg'])
    return selected, root


def capture_inputs(source, capture_path, profile, policy):
    raw = capture_path.read_bytes()
    require(digest(raw) == policy['capture_sha256'], 'capture manifest does not match reviewed policy')
    capture = json.loads(raw)
    require(capture['source_revision'] == policy['source_revision'], 'capture source revision mismatch')
    require(capture['source_unchanged'] and not capture['build_executed'], 'capture source/build disposition mismatch')
    record = capture['profiles'][profile]
    require(record['exit_code'] == 0, 'metadata capture did not succeed')
    argv = record['argv']
    require(argv[:2] == ['cargo', 'metadata'] and '--offline' in argv and '--locked' in argv, 'wrong metadata command')
    def option(name):
        return argv[argv.index(name) + 1] if name in argv else None
    require(option('--format-version') == '1' and option('--filter-platform') == policy['target'], 'metadata target/format mismatch')
    require('--all-features' not in argv and '--no-default-features' not in argv, 'unadmitted feature selection')
    expected_feature = ','.join(policy['profiles'][profile]) or None
    require(option('--features') == expected_feature, 'captured feature arguments mismatch')
    for path, h in capture['source_input_sha256'].items():
        require(policy['source_sha256'].get(path) == h, 'unreviewed capture source input: ' + path)
        checked_bytes(source, path, h)
    meta_raw = checked_bytes(capture_path.parent, profile + '.json', record['stdout_sha256'])
    checked_bytes(capture_path.parent, profile + '-stderr.txt', record['stderr_sha256'])
    metadata = json.loads(meta_raw)
    selected, root = resolved_graph(metadata, profile, policy)
    require(packages_path := selected[root]['manifest_path'], 'missing root manifest path')
    suffix = '/' + policy['root_manifest']
    require(packages_path.endswith(suffix), 'captured root manifest suffix mismatch')
    require(option('--manifest-path') == packages_path, 'captured manifest argument mismatch')
    captured_root = packages_path[:-len(suffix)]
    return selected, captured_root


def verify(source_root, package_root, capture_path, profile, policy_path):
    source, package = Path(source_root), Path(package_root)
    require(not source.is_symlink() and not package.is_symlink(), 'root symlink is not admitted')
    policy = json.loads(Path(policy_path).read_bytes())
    require(policy['schema_version'] == 1 and profile in policy['profiles'], 'unknown policy/profile')
    selected, captured_root = capture_inputs(source, Path(capture_path), profile, policy)
    pins = policy['source_sha256']
    lock = tomllib.loads(checked_bytes(source, policy['root_lock'], pins[policy['root_lock']]).decode())
    require(lock['version'] == 4, 'unsupported lockfile version')
    lock_rows = {}
    for row in lock['package']:
        key = (row['name'], row['version'], row.get('source'))
        require(key not in lock_rows, 'ambiguous duplicate lock identity')
        lock_rows[key] = row
    descriptor = json.loads(safe_file(package, 'distribution.json').read_bytes())
    require(descriptor['schema_version'] == 1 and descriptor['profile'] == profile, 'package profile mismatch')
    require(descriptor['source_revision'] == policy['source_revision'], 'package source revision mismatch')
    require(descriptor['artifact']['path'] == policy['artifact_path'], 'unadmitted artifact path')
    require(isinstance(descriptor['artifact']['synthetic'], bool), 'synthetic declaration must be boolean')
    required = {'distribution.json'}
    locals_, externals = [], []

    def package_file(path, expected):
        required.add(path)
        return checked_bytes(package, path, expected)

    for path in ['LICENSE', 'LICENSE-LGPL', 'NOTICE', 'rust/THIRD-PARTY-NOTICES.md', 'rust/third-party/manifest.json']:
        checked_bytes(source, path, pins[path])
        package_file(path, pins[path])
    package_file('source/SOURCE-REVISION', digest((policy['source_revision'] + '\n').encode()))
    for item in selected.values():
        key = (item['name'], item['version'], item['source'])
        require(key in lock_rows, 'metadata/lock identity mismatch: ' + item['name'])
        if item['source'] is None:
            prefix = captured_root + '/'
            require(item['manifest_path'].startswith(prefix), 'local manifest outside captured source')
            rel = item['manifest_path'][len(prefix):]
            require(rel in pins and rel.startswith('rust/crates/') and rel.endswith('/Cargo.toml'), 'unadmitted local dependency: ' + rel)
            crate = str(PurePosixPath(rel).parent) + '/'
            local_files = {p: h for p, h in pins.items() if p.startswith(crate)}
            actual = set()
            for path in (source / crate).rglob('*'):
                parts = path.relative_to(source / crate).parts
                if 'target' in parts:
                    continue
                require(not path.is_symlink(), 'source symlink is not admitted')
                if path.is_file():
                    actual.add(path.relative_to(source).as_posix())
            require(actual == set(local_files), 'unpinned or missing local source file: ' + crate)
            for path, h in local_files.items():
                checked_bytes(source, path, h)
                package_file('source/' + path, h)
            manifest = tomllib.loads((source / rel).read_text())['package']
            require(manifest['name'] == item['name'] and manifest['version'] == item['version'] and manifest['license'] == item['license'], 'local manifest/metadata mismatch')
            locals_.append(item['name'])
        else:
            label = item['name'] + '-' + item['version']
            require(label in policy['registry'], 'unknown active registry dependency: ' + label)
            row = policy['registry'][label]
            require(item['source'] == row['source'] and lock_rows[key]['checksum'] == row['checksum'], 'registry source/checksum mismatch: ' + label)
            raw = package_file('dependency-manifests/' + label + '/Cargo.toml', row['source_manifest_sha256'])
            manifest = tomllib.loads(raw.decode())['package']
            require((manifest['name'], manifest['version'], manifest['license']) == (item['name'], item['version'], item['license']), 'registry manifest/metadata mismatch: ' + label)
            package_file('dependency-archives/' + label + '.crate', row['checksum'])
            for license_file in row['license_files']:
                require(pins.get(license_file['path']) == license_file['sha256'], 'unreviewed license policy')
                checked_bytes(source, license_file['path'], license_file['sha256'])
                package_file(license_file['path'], license_file['sha256'])
            externals.append(label)
    for dest, origin in policy['runtime_assets'].items():
        checked_bytes(source, origin, pins[origin])
        package_file(dest, pins[origin])
    artifact = package_file(policy['artifact_path'], descriptor['artifact']['sha256'])
    require(artifact.startswith(b'\x00asm\x01\x00\x00\x00'), 'artifact lacks WASM v1 header')
    actual_files = set()
    for path in package.rglob('*'):
        require(not path.is_symlink(), 'package symlink is not admitted')
        if path.is_file():
            actual_files.add(path.relative_to(package).as_posix())
    require(actual_files == required, 'unknown or missing packaged files: ' + ','.join(sorted(actual_files ^ required)))
    return {'status': 'SUPPLIED_PACKAGE_AND_CAPTURE_MATCH_POLICY', 'profile': profile,
            'source_revision': policy['source_revision'], 'target': policy['target'],
            'active_local_packages': sorted(locals_), 'active_registry_packages': sorted(externals),
            'files_checked': len(required), 'policy_sha256': digest(Path(policy_path).read_bytes()),
            'synthetic_artifact_declared': descriptor['artifact']['synthetic'],
            'artifact_build_correspondence': 'UNPROVEN', 'upstream_provenance': 'OPEN',
            'full_legal_clearance': 'NOT_ESTABLISHED', 'build_executed_by_verifier': False}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['source-root', 'package-root', 'capture', 'policy']:
        parser.add_argument('--' + name, required=True, type=Path)
    parser.add_argument('--profile', choices=['default', 'candidate'], required=True)
    args = parser.parse_args()
    try:
        result = verify(args.source_root, args.package_root, args.capture, args.profile, args.policy)
    except (VerificationError, OSError, KeyError, TypeError, tomllib.TOMLDecodeError, json.JSONDecodeError) as error:
        print(json.dumps({'status': 'REJECTED', 'reason': str(error)}))
        return 1
    print(json.dumps(result, indent=2))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
