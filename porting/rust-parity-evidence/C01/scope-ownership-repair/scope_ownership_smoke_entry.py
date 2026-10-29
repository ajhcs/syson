"""Attest the fresh smoke scope, publish actual controls, then stay alive."""
from pathlib import Path
import hashlib
import json
import os
import signal
import sys

helper, unit, cookie, proof_path, ready_path = sys.argv[1:]
raw = Path(helper).read_bytes()
if hashlib.sha256(raw).hexdigest() != 'ff18c5d2e6af6a8b21a4d59129401d9c9274f4d4727e2e448eae331c72f6b92d':
    raise RuntimeError('reviewed ownership helper changed')
ownership = {'__file__': helper}
exec(compile(raw, helper, 'exec'), ownership)
proof = ownership['attest'](unit, cookie, 'syson-c01-ownership-smoke-', proof_path, os.environ)
group = Path('/sys/fs/cgroup') / proof['control_group'].lstrip('/')
expected = {'cpu.max': '200000 100000', 'memory.high': '4294967296',
            'memory.max': '6442450944', 'memory.swap.max': '0', 'pids.max': '128'}
actual = {name: (group / name).read_text().strip() for name in expected}
if actual != expected:
    raise RuntimeError('smoke scope resource controls differ')
ready = {'proof': proof, 'controls': actual, 'entry_pid': os.getpid()}
temporary = Path(ready_path + '.tmp')
with temporary.open('x') as target:
    target.write(json.dumps(ready, indent=2) + '\n')
temporary.replace(ready_path)
while True:
    signal.pause()
