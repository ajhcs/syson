#!/usr/bin/env python3
"""Read and hash pinned source without executing upstream code or a compiler."""
import argparse
import hashlib
import json
import pathlib
import subprocess
ROOT = pathlib.Path(__file__).resolve().parents[3]
SCOPE = pathlib.Path(__file__).resolve().parent

def main():
    p = argparse.ArgumentParser()
    p.add_argument('--sirius-source', type=pathlib.Path, required=True)
    args = p.parse_args()
    evidence = json.loads((SCOPE / 'source-mapping.json').read_text())
    for group in evidence['source_groups']:
        root = ROOT if group['component'] == 'syson' else args.sirius_source
        for row in group['files']:
            raw = subprocess.check_output(['git', '-C', str(root), 'show', group['revision'] + ':' + row['path']])
            assert hashlib.sha256(raw).hexdigest() == row['sha256'], row['path']
            assert (root / row['path']).read_bytes() == raw, ('working-source drift', row['path'])
            lines = raw.decode().splitlines()
            for excerpt in row.get('excerpts', []):
                assert '\n'.join(lines[excerpt['first_line'] - 1:excerpt['last_line']]) == excerpt['source'], excerpt['name']
    for row in evidence['reused_dependency_files']:
        assert hashlib.sha256((ROOT / row['path']).read_bytes()).hexdigest() == row['sha256'], row['path']
    print(json.dumps({'pinned_source_check': 'PASS', 'primary_node_contracts': 3, 'reused_dependency': 'syson-editor', 'upstream_execution': 'NOT_RUN'}))
if __name__ == '__main__':
    main()
