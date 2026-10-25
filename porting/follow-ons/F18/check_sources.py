#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Hash-check exact pinned source and frozen new Rust files; no upstream runtime."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent

def sha(data):
    return hashlib.sha256(data).hexdigest()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--sirius-source', type=Path, required=True)
    args = parser.parse_args()
    manifest = json.loads((HERE / 'source-manifest.json').read_text())
    for row in manifest['sources']:
        root = ROOT if row['component'] == 'syson' else args.sirius_source
        expected = '6ba2ac64e6b16ee2b05752ad55395eebb87d4910' if row['component'] == 'syson' else '3191e2b8990373afee3415f6b26f8c5039a28321'
        if row['revision'] != expected:
            raise ValueError('Unexpected source pin')
        spec = expected + ':' + row['path']
        raw = subprocess.check_output(['git', '-C', str(root), 'show', spec])
        blob = subprocess.check_output(['git', '-C', str(root), 'rev-parse', spec], text=True).strip()
        if sha(raw) != row['sha256'] or blob != row['blob']:
            raise ValueError('Pinned source mismatch: ' + row['path'])
        lines = raw.decode().splitlines()
        for anchor in row['anchors']:
            first, last = anchor['first'], anchor['last']
            if not 1 <= first <= last <= len(lines):
                raise ValueError('Invalid source range')
            excerpt = '\n'.join(lines[first - 1:last])
            if excerpt != anchor['source'] or sha(excerpt.encode()) != anchor['sha256']:
                raise ValueError('Source excerpt drift')
    for path, expected in manifest['frozen_rust_inputs'].items():
        if sha((ROOT / path).read_bytes()) != expected:
            raise ValueError('Frozen Rust drift: ' + path)
    print(json.dumps({'status': 'PASS', 'pinned_source_files': len(manifest['sources']), 'frozen_rust_files': len(manifest['frozen_rust_inputs']), 'upstream_runtime': 'NOT_RUN'}))

if __name__ == '__main__':
    main()
