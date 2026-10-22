#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Run nine real Rust fixture cases from pinned Git snapshots, offline, one job."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def timestamp():
    return datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ')


def run(root, build_dir, implementation_revision, harness_revision, output_dir):
    def git(*args):
        return subprocess.check_output(['git', '-C', str(root), *args])
    def full_revision(value):
        assert re.fullmatch('[0-9a-f]{40}', value), 'use a full immutable revision'
        assert git('rev-parse', value).decode().strip() == value
        return value
    implementation_revision = full_revision(implementation_revision)
    harness_revision = full_revision(harness_revision)
    def blob(revision, path):
        return git('show', f'{revision}:{path}')
    output = root / output_dir
    assert output_dir.startswith('porting/runs/') and '..' not in Path(output_dir).parts
    assert output.resolve().is_relative_to((root / 'porting/runs').resolve())
    assert not output.exists(), 'use a fresh evidence directory; never overwrite prior run artifacts'
    assert build_dir.is_dir(), 'SYSON_BUILD_DIR must be an existing approved build directory'
    stage = Path(tempfile.mkdtemp(prefix='ledger-fixtures-', dir=build_dir))
    # Snapshot product library files only; no upstream scripts or tests run.
    paths = git('ls-tree', '-r', '--name-only', implementation_revision, '--', 'rust/crates').decode().splitlines()
    for path in paths:
        if path.endswith('/Cargo.toml') or '/src/' in path:
            target = stage / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(blob(implementation_revision, path))
    assert not list((stage / 'rust/crates').rglob('build.rs')), 'review new local build scripts separately'
    ledger = json.loads(blob(harness_revision, 'porting/behavior-ledger.json'))
    harness_paths = ['rust/validation/Cargo.toml', 'rust/validation/Cargo.lock',
                     'rust/validation/tests/ledger_fixtures.rs',
                     'rust/validation/scripts/run_ledger_fixtures.py']
    fixture_paths = sorted({fixture['path'] for row in ledger['behaviors']
                            for group in row['cases'].values() for case in group for fixture in case['fixtures']})
    harness_paths += fixture_paths
    harness_files = []
    for path in harness_paths:
        raw = blob(harness_revision, path)
        target = stage / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(raw)
        harness_files.append({'path': path, 'sha256': digest(raw)})
    for row in ledger['behaviors']:
        assert row['status'] == 'unrun' and not row['runs']['upstream'], 'this bounded runner never promotes upstream status'
        row['implementation']['revision'] = implementation_revision
        for ref in row['implementation']['files']:
            ref['sha256'] = digest(blob(implementation_revision, ref['path']))
        for group in row['cases'].values():
            for case in group:
                for fixture in case['fixtures']:
                    assert digest(blob(harness_revision, fixture['path'])) == fixture['sha256']
    command = ['cargo', 'test', '--offline', '--locked', '--jobs', '1', '--manifest-path',
               str(stage / 'rust/validation/Cargo.toml'), '--test', 'ledger_fixtures', '--', '--test-threads=1']
    environment = dict(os.environ, CARGO_TARGET_DIR=str(build_dir / 'ledger-fixture-target'))
    started = timestamp()
    process = subprocess.run(command, cwd=stage, env=environment, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    finished = timestamp()
    transcript = process.stdout.decode(errors='replace')
    # Portable public transcript: redact checkout/build/cache prefixes, retaining actual test output.
    for value, placeholder in [(str(stage), '${SYSON_SNAPSHOT}'), (str(build_dir), '${SYSON_BUILD_DIR}'),
                               (str(root), '${SYSON_CHECKOUT}'),
                               (os.environ.get('CARGO_HOME', ''), '${CARGO_HOME}')]:
        if value:
            transcript = transcript.replace(value, placeholder)
    output.mkdir(parents=True)
    transcript_path = output / 'cargo-test.txt'
    transcript_path.write_text(transcript)
    observed = set(re.findall(r'^test case_([0-9]{6}) \.\.\. ok$', transcript, re.MULTILINE))
    expected = {case['id'].removeprefix('SYSON-CASE-') for row in ledger['behaviors'] for group in row['cases'].values() for case in group}
    assert process.returncode == 0 and observed == expected and len(observed) == 9, 'native fixture run did not pass all nine exact cases; transcript retained, ledger unchanged'
    transcript_ref = {'path': str(transcript_path.relative_to(root)), 'sha256': digest(transcript_path.read_bytes())}
    portable_command = ['cargo', 'test', '--offline', '--locked', '--jobs', '1', '--manifest-path',
                        '${SYSON_SNAPSHOT}/rust/validation/Cargo.toml', '--test', 'ledger_fixtures', '--', '--test-threads=1']
    for row in ledger['behaviors']:
        cases = [case for group in row['cases'].values() for case in group]
        report = {'behavior_id': row['id'], 'side': 'rust', 'revision': implementation_revision,
                  'executed': True, 'command': portable_command, 'started_at': started, 'finished_at': finished,
                  'exit_code': process.returncode, 'result': 'passed', 'case_ids': [case['id'] for case in cases],
                  'fixture_hashes': sorted({fixture['sha256'] for case in cases for fixture in case['fixtures']}),
                  'transcript': transcript_ref, 'harness_revision': harness_revision, 'harness_files': harness_files}
        report_path = output / (row['id'] + '-rust.json')
        report_path.write_text(json.dumps(report, indent=2) + '\n')
        row['runs']['rust'] = [{'path': str(report_path.relative_to(root)), 'sha256': digest(report_path.read_bytes())}]
        row['disposition'] = 'Nine hashed fixtures consumed by the pinned Rust native harness, with passing native records. Upstream runtime remains unrun; independent paired parity review is pending. Zero verified claim.'
    (root / 'porting/behavior-ledger.json').write_text(json.dumps(ledger, indent=2) + '\n')
    return {'native_cases_passed': len(observed), 'upstream_runs': 0, 'verified': 0, 'output': output_dir}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parents[3])
    parser.add_argument('--implementation-revision', required=True)
    parser.add_argument('--harness-revision', required=True)
    parser.add_argument('--output-dir', required=True)
    args = parser.parse_args()
    print(json.dumps(run(args.root.resolve(), Path(os.environ['SYSON_BUILD_DIR']).resolve(),
                         args.implementation_revision, args.harness_revision, args.output_dir), sort_keys=True))


if __name__ == '__main__':
    main()
