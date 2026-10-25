#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Frozen stdlib-only source packet checks; no upstream application execution."""
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]
BASE = ROOT / 'porting/area-inventories/API/session-isolation'
DOCS = ROOT / 'porting/follow-ons/F21'
run = sys.argv[1]
if not run.startswith('f21-') or '/' in run:
    raise SystemExit('Invalid capture ID')
out = DOCS / 'validation' / run
out.mkdir(parents=True, exist_ok=False)
(out / 'scratch').mkdir()
sys.path.insert(0, str(BASE))
from check_contract import SOURCES
manifest = json.loads((BASE / 'source-manifest.json').read_text())
if {r['id']: r['path'] for r in manifest['evidence']} != SOURCES:
    raise SystemExit('Native source closure differs from fixed checker paths')
inputs = sorted([*BASE.glob('*.py'), *BASE.glob('*.json'), DOCS / 'create_packet.py',
                 *[ROOT / p for p in SOURCES.values()]])
def sha(data):
    return hashlib.sha256(data).hexdigest()
def stamp():
    return datetime.now(timezone.utc).isoformat()
def freeze():
    return {str(p.relative_to(ROOT)): sha(p.read_bytes()) for p in inputs}
env = os.environ.copy()
env.update(PYTHONDONTWRITEBYTECODE='1', TMPDIR=str(out / 'scratch'))
commands = [[sys.executable, '-m', 'unittest', 'discover', '-s', str(BASE), '-p', 'test_contract.py', '-v'],
            [sys.executable, str(BASE / 'check_contract.py'), '--root', str(ROOT)]]
r = dict(run=run, started_utc=stamp(), runner_sha256=sha(Path(__file__).read_bytes()),
         source_before=freeze(), source_revision=subprocess.check_output(['git', '-C', str(ROOT), 'rev-parse', 'HEAD'], text=True).strip(),
         commands=[], upstream_runtime='NOT_RUN', source_status='BLOCKED_SOURCE_PREPARATION_ONLY')
(out / 'executed-runner.py').write_bytes(Path(__file__).read_bytes())
try:
    for i, argv in enumerate(commands):
        item = dict(argv=argv, started_utc=stamp())
        result = subprocess.run(argv, cwd=ROOT, env=env, capture_output=True, timeout=60)
        (out / f'{i:02d}-stdout.txt').write_bytes(result.stdout)
        (out / f'{i:02d}-stderr.txt').write_bytes(result.stderr)
        item.update(completed_utc=stamp(), exit_code=result.returncode,
                    stdout_sha256=sha(result.stdout), stderr_sha256=sha(result.stderr))
        r['commands'].append(item)
        if result.returncode:
            r['status'] = 'FAIL'
            break
    else:
        r['status'] = 'PASS'
finally:
    r.update(completed_utc=stamp(), source_after=freeze())
    r['source_unchanged'] = r['source_before'] == r['source_after']
    if not r['source_unchanged']:
        r['status'] = 'SOURCE_DRIFT'
    (out / 'receipt.json').write_text(json.dumps(r, indent=2) + '\n')
print(json.dumps({k: r[k] for k in ['run', 'status', 'source_unchanged', 'source_status']}))
sys.exit(0 if r['status'] == 'PASS' else 1)
