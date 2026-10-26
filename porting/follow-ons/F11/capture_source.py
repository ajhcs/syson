#!/usr/bin/env python3
"""Capture the final source-preparation check without executing an upstream tool."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
from datetime import datetime, timezone
root = Path(__file__).resolve().parents[3]
here = Path(__file__).resolve().parent
out = here / 'validation/source-check-01'
out.mkdir(parents=True, exist_ok=False)
def stamp(): return datetime.now(timezone.utc).isoformat()
def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()
paths = [here / 'prepare.py', here / 'source-manifest.json', root / 'rust/crates/syson-ast-adapter/src/reserved.rs']
paths += [root / 'porting/source-map/syside-source' / name for name in json.loads((here / 'source-manifest.json').read_text())['input_sha256']]
def freeze(): return {str(p.relative_to(root)): sha(p) for p in paths}
argv = [sys.executable, '-B', str(here / 'prepare.py'), '--check']
r = {'started_utc': stamp(), 'argv': argv, 'source_before': freeze(), 'runner_sha256': sha(Path(__file__)), 'upstream_runtime': 'NOT_RUN'}
child = subprocess.run(argv, cwd=root, capture_output=True, timeout=30)
(out / 'stdout.txt').write_bytes(child.stdout)
(out / 'stderr.txt').write_bytes(child.stderr)
r.update(completed_utc=stamp(), exit_code=child.returncode, stdout_sha256=sha(out / 'stdout.txt'), stderr_sha256=sha(out / 'stderr.txt'), source_after=freeze())
r['source_unchanged'] = r['source_before'] == r['source_after']
r['status'] = 'PASS' if child.returncode == 0 and r['source_unchanged'] else 'FAIL'
(out / 'receipt.json').write_text(json.dumps(r, indent=2) + '\n')
print(json.dumps({'status': r['status'], 'receipt': str(out / 'receipt.json')}))
sys.exit(0 if r['status'] == 'PASS' else 1)
