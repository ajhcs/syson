#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Capture exact inputs and original output from one F02 check run."""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]
CONTRACT = ROOT / 'porting/area-inventories/DOMAIN-QUERY/imports'
OUT = Path(__file__).resolve().parent / 'validation' / sys.argv[1]
OUT.mkdir(parents=True, exist_ok=False)


def stamp():
    return datetime.now(timezone.utc).isoformat()


def inputs():
    return {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in sorted(CONTRACT.iterdir()) if p.is_file()}


result = {'started_utc': stamp(), 'source_base': subprocess.check_output(
    ['git', '-C', str(ROOT), 'rev-parse', 'HEAD'], text=True).strip(),
    'source_before': inputs(), 'commands': [], 'upstream_execution': 'NOT_RUN'}
commands = [[sys.executable, '-B', str(CONTRACT / 'validate.py'), '--root', str(ROOT)],
            [sys.executable, '-B', '-m', 'unittest', 'discover', '-s', str(CONTRACT), '-v']]
for i, command in enumerate(commands):
    started = stamp()
    child = subprocess.run(command, cwd=ROOT, capture_output=True, timeout=60)
    (OUT / f'{i:02d}-stdout.txt').write_bytes(child.stdout)
    (OUT / f'{i:02d}-stderr.txt').write_bytes(child.stderr)
    result['commands'].append({'argv': command, 'started_utc': started,
                               'finished_utc': stamp(), 'exit_code': child.returncode,
                               'stdout_sha256': hashlib.sha256(child.stdout).hexdigest(),
                               'stderr_sha256': hashlib.sha256(child.stderr).hexdigest()})
result['source_after'] = inputs()
result.update(finished_utc=stamp(), source_unchanged=result['source_before'] == result['source_after'])
result['status'] = 'PASS' if result['source_unchanged'] and all(c['exit_code'] == 0 for c in result['commands']) else 'FAIL'
(OUT / 'receipt.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({'status': result['status'], 'receipt': str(OUT / 'receipt.json')}))
sys.exit(result['status'] != 'PASS')
