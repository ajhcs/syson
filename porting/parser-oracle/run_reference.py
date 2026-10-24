#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Read-only bounded reference invocation. Emit compressed raw evidence; write no files.

Run this parent and its Node child in the default restricted boundary, not an
unsandboxed escalation. Prepare fixtures separately and persist output separately.
The tool does not grant execution authority or certify filesystem/network effects.
"""
import argparse
import base64
import datetime
import hashlib
import json
import os
from pathlib import Path
import subprocess
import zlib

NODE_SHA = '7fde7b8afa198da66257f42ee2001d874c7355631e6d1579a5fb5ef1f246df4c'
BUNDLE_SHA = '47d8fe11fda1b58b6257463087f663be953c2ab6c2b5e1e2ccc44161ed5cd0bd'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def now():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--node', type=Path, required=True)
    p.add_argument('--bundle', type=Path, required=True)
    p.add_argument('--fixture', type=Path, required=True)
    p.add_argument('--fixture-sha256', required=True)
    a = p.parse_args()
    inputs = {}
    for label, path, wanted in [('node', a.node, NODE_SHA), ('bundle', a.bundle, BUNDLE_SHA), ('fixture', a.fixture, a.fixture_sha256)]:
        if not path.is_absolute() or not path.is_file():
            p.error(label + ' must be an existing absolute file')
        size = path.stat().st_size
        if label == 'fixture':
            if size > 1024 or path.suffix not in ('.sysml', '.kerml'):
                p.error('fixture must be at most 1024 bytes with a recognized suffix')
            path.read_bytes().decode('utf-8')
        hasher = hashlib.sha256()
        with path.open('rb') as stream:
            for chunk in iter(lambda: stream.read(1048576), b''):
                hasher.update(chunk)
        actual = hasher.hexdigest()
        if actual != wanted:
            p.error(label + ' SHA256 mismatch; no invocation performed')
        inputs[label] = {'path': str(path), 'sha256': actual, 'bytes': size}
    env = {k: v for k, v in os.environ.items() if not k.startswith('NODE_')}
    env['FORCE_COLOR'] = '0'
    command = [str(a.node), str(a.bundle), 'dump', '--validate', '--stdlib', 'none', str(a.fixture)]
    report = {'command': command, 'inputs': inputs, 'runner_sha256': digest(Path(__file__).read_bytes()), 'start_utc': now(), 'mode': 'secondary diagnostic CLI path', 'timeout_seconds': 60, 'stdin': 'closed', 'environment': 'all NODE_* removed; FORCE_COLOR=0; other environment retained', 'boundary': 'caller must retain the default network-restricted boundary; this report does not attest it'}
    try:
        r = subprocess.run(command, env=env, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=60)
        report.update(exit_code=r.returncode, timed_out=False)
        streams = {'stdout': r.stdout, 'stderr': r.stderr}
    except subprocess.TimeoutExpired as e:
        report.update(exit_code=None, timed_out=True)
        streams = {'stdout': e.stdout or b'', 'stderr': e.stderr or b''}
    report['end_utc'] = now()
    for name, data in streams.items():
        report[name] = {'bytes': len(data), 'sha256': digest(data), 'encoding': 'zlib9+base64', 'data': base64.b64encode(zlib.compress(data, 9)).decode('ascii')}
    print(json.dumps(report))
    # A nonzero reference exit can be an expected negative observation. Preserve
    # it in the report rather than suppressing the output or automatically retrying.


if __name__ == '__main__':
    main()
