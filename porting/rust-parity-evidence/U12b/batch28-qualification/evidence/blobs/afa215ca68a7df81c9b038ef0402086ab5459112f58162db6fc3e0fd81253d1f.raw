"""Attest invocation and exact check-only controls before loading the checker."""
from pathlib import Path
import hashlib
import json
import os
import sys

helper = Path('/workspace/worktrees/builderbob/syson/batch26-scope-ownership/porting/rust-parity-evidence/C01/scope-ownership-repair/scope_ownership.py')
raw = helper.read_bytes()
if hashlib.sha256(raw).hexdigest() != 'ff18c5d2e6af6a8b21a4d59129401d9c9274f4d4727e2e448eae331c72f6b92d':
    raise RuntimeError('reviewed ownership helper changed')
owner = {'__file__': str(helper)}
exec(compile(raw, str(helper), 'exec'), owner)
unit, cookie, destination = sys.argv[1:]
output = Path(destination)
proof = owner['attest'](unit, cookie, 'syson-u12b-checks-', output / 'ownership.json', os.environ)
group = Path('/sys/fs/cgroup') / proof['control_group'].lstrip('/')
expected = {'cpu.max': '200000 100000', 'memory.high': '536870912',
            'memory.max': '1073741824', 'memory.swap.max': '0', 'pids.max': '128'}
actual = {name: (group / name).read_text().strip() for name in expected}
if actual != expected:
    raise RuntimeError('actual56-case qualification controls differ')
checker = Path(__file__).with_name('verify_forest_placement.py')
raw = checker.read_bytes()
if hashlib.sha256(raw).hexdigest() != '893834efd5a6b3c3e870c55e9444a838a00cb40120fc5ddd8cff26cd1763ea01':
    raise RuntimeError('reviewed checker changed')
record = {'actual': actual, 'control_group': proof['control_group'],
          'invocation_id': proof['invocation_id'], 'checker_sha256': hashlib.sha256(raw).hexdigest()}
with (output / 'actual-controls.json').open('x') as stream:
    stream.write(json.dumps(record, indent=2) + '\n')
sys.argv = [str(checker)]
exec(compile(raw, str(checker), 'exec'), {'__name__': '__main__', '__file__': str(checker)})
