"""Attest actual invocation and full-profile controls before loading this driver."""
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
driver_sha, profile_sha, unit, cookie, destination = sys.argv[1:]
output = Path(destination)
proof = owner['attest'](unit, cookie, 'syson-u12b-placement-profile-', output / 'ownership.json', os.environ)
group = Path('/sys/fs/cgroup') / proof['control_group'].lstrip('/')
expected = {'cpu.max': '200000 100000', 'memory.high': '2147483648',
            'memory.max': '3221225472', 'memory.swap.max': '0', 'pids.max': '128'}
actual = {name: (group / name).read_text().strip() for name in expected}
if actual != expected:
    raise RuntimeError('actual full-placement-profile controls differ')
driver, profile = Path(__file__).with_name('run_profile.py'), Path(__file__).with_name('profile.rs')
raw = driver.read_bytes()
if hashlib.sha256(raw).hexdigest() != driver_sha or hashlib.sha256(profile.read_bytes()).hexdigest() != profile_sha:
    raise RuntimeError('reviewed driver/profile changed')
record = {'actual': actual, 'control_group': proof['control_group'], 'invocation_id': proof['invocation_id'],
          'driver_sha256': driver_sha, 'profile_sha256': profile_sha}
with (output / 'actual-controls.json').open('x') as stream:
    stream.write(json.dumps(record, indent=2) + '\n')
sys.argv = [str(driver), profile_sha]
exec(compile(raw, str(driver), 'exec'), {'__name__': '__main__', '__file__': str(driver)})
