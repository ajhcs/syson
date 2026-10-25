#!/usr/bin/env python3
"""Verify pinned ELK boundary source; never execute upstream JavaScript."""
import argparse
import hashlib
import json
import pathlib
import subprocess

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--sirius-source', type=pathlib.Path, required=True)
    args = parser.parse_args()
    record = json.loads(pathlib.Path(__file__).with_name('source-mapping.json').read_text())
    for row in record['files']:
        raw = subprocess.check_output(['git', '-C', str(args.sirius_source), 'show', record['sirius_candidate_revision'] + ':' + row['path']])
        assert hashlib.sha256(raw).hexdigest() == row['sha256'], row['path']
        assert (args.sirius_source / row['path']).read_bytes() == raw, ('source drift', row['path'])
        lines = raw.decode().splitlines()
        for anchor in row['anchors']:
            assert anchor['text'] in lines[anchor['line'] - 1], anchor
    print(json.dumps({'source_boundary_check': 'PASS', 'files': len(record['files']), 'upstream_execution': 'NOT_RUN', 'algorithm_origin': record['algorithm_origin']}))
if __name__ == '__main__':
    main()
