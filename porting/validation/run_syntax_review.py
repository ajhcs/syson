#!/usr/bin/env python3
"""Capture native-only syntax evidence from an exact Git snapshot. No upstream execution."""
import argparse
import datetime
import hashlib
import json
import os
import pathlib
import subprocess


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repository', required=True, type=pathlib.Path)
    parser.add_argument('--revision', required=True)
    parser.add_argument('--build-dir', required=True, type=pathlib.Path)
    parser.add_argument('--output-dir', required=True, type=pathlib.Path)
    args = parser.parse_args()
    if not args.build_dir.is_absolute() or not args.output_dir.is_absolute():
        parser.error('build and output directories must be absolute')
    repo = args.repository.resolve()
    def git(*command):
        return subprocess.check_output(['git', '-C', str(repo), *command])
    revision = git('rev-parse', '--verify', args.revision + '^{commit}').decode().strip()
    stage = args.build_dir / 'snapshot'
    stage.mkdir(parents=True, exist_ok=False)
    args.output_dir.mkdir(parents=True, exist_ok=False)
    prefixes = ['rust/crates/syson-syntax', 'rust/validation/syntax',
                'porting/validation/fixtures/parser-oracle.sysml',
                'porting/validation/fixtures/syntax-terminal-base.json']
    paths = git('ls-tree', '-r', '--name-only', revision, '--', *prefixes).decode().splitlines()
    hashes = {}
    for name in paths:
        data = git('show', revision + ':' + name)
        path = stage / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        hashes[name] = sha(data)
    expected_fixture = 'ebe5c225b064119eb2584a9a7d4f268b0a10e8de0e15df77a9240c12f05cbdf8'
    if hashes.get(prefixes[2]) != expected_fixture:
        raise ValueError('not the exact reviewed 37-byte Oracle fixture')
    env = os.environ.copy()
    env['CARGO_TARGET_DIR'] = str(args.build_dir / 'target')
    env['CARGO_BUILD_JOBS'] = '1'
    commands = []
    for label, action in [('tests', ['test']), ('observation', ['run', '--quiet', '--example', 'oracle_receipt'])]:
        command = ['cargo', *action, '--offline', '--locked', '--jobs', '1',
                   '--manifest-path', 'rust/validation/syntax/Cargo.toml']
        start = datetime.datetime.now(datetime.timezone.utc).isoformat()
        result = subprocess.run(command, cwd=stage, env=env, stdin=subprocess.DEVNULL,
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=120)
        end = datetime.datetime.now(datetime.timezone.utc).isoformat()
        record = {'argv': command, 'start_utc': start, 'end_utc': end, 'exit_code': result.returncode}
        for stream, raw in [('stdout', result.stdout), ('stderr', result.stderr)]:
            portable = raw.replace(str(args.build_dir).encode(), b'${SYSON_BUILD_DIR}/syntax-review-01')
            portable = portable.replace(str(repo).encode(), b'${REPOSITORY}')
            name = label + '.' + stream + ('.json' if label == 'observation' and stream == 'stdout' else '.txt')
            (args.output_dir / name).write_bytes(portable)
            record[stream] = {'path': name, 'raw_sha256': sha(raw), 'portable_sha256': sha(portable),
                              'raw_bytes': len(raw), 'portable_bytes': len(portable)}
        commands.append(record)
        if result.returncode:
            raise RuntimeError(f'{label} failed with exit code {result.returncode}; evidence preserved')
    observation = json.loads((args.output_dir / 'observation.stdout.json').read_bytes())
    if not (observation['syntax_recognized'] and observation['consumed_input_bytes'] == 37
            and observation['semantic_status'] == 'NotPerformed' and observation['ast_status'] == 'NotPerformed'):
        raise ValueError('unexpected native observation; do not claim qualification')
    receipt = {'harness_revision': revision, 'implementation_tree': git('rev-parse', revision + ':rust/crates/syson-syntax').decode().strip(),
               'runner_sha256': sha(pathlib.Path(__file__).read_bytes()), 'source_sha256': hashes,
               'commands': commands, 'fixture_sha256': expected_fixture,
               'upstream_execution_by_this_runner': 'NOT RUN', 'ast_comparison': 'NOT PERFORMED',
               'semantic_parity': 'NOT ESTABLISHED', 'all_source_rule_dispositions': 'UNPORTED',
               'portable_output_transform': 'replace selected build directory and repository prefix only',
               'test_result_lines': [s for s in (args.output_dir / 'tests.stdout.txt').read_text().splitlines() if s.startswith('test result:')]}
    (args.output_dir / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'revision': revision, 'receipt': 'receipt.json', 'tests': receipt['test_result_lines'],
                      'cst_nodes': len(observation['cst']), 'tokens': len(observation['tokens']), 'semantics': observation['semantic_status']}))


if __name__ == '__main__':
    main()
