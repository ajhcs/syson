"""Attest actual invocation before search driver checks or execution."""
from pathlib import Path
import hashlib, json, os, sys
helper = Path('/workspace/worktrees/builderbob/syson/batch26-scope-ownership/porting/rust-parity-evidence/C01/scope-ownership-repair/scope_ownership.py')
raw = helper.read_bytes()
if hashlib.sha256(raw).hexdigest() != 'ff18c5d2e6af6a8b21a4d59129401d9c9274f4d4727e2e448eae331c72f6b92d':
    raise RuntimeError('reviewed ownership helper changed')
ownership = {'__file__': str(helper)}
exec(compile(raw, str(helper), 'exec'), ownership)
driver = Path(sys.argv[1])
unit, cookie, proof_path, controls_path = sys.argv[4:]
proof = ownership['attest'](unit, cookie, 'syson-u11b-search-profile-', proof_path, os.environ)
group = Path('/sys/fs/cgroup') / proof['control_group'].lstrip('/')
expected = {'cpu.max': '200000 100000', 'memory.high': '3221225472',
            'memory.max': '4294967296', 'memory.swap.max': '0', 'pids.max': '128'}
actual = {name: (group / name).read_text().strip() for name in expected}
if actual != expected:
    raise RuntimeError('actual search profiling resource controls differ')
raw = driver.read_bytes()
profile = driver.parent / 'profile.rs'
if hashlib.sha256(raw).hexdigest() != sys.argv[2] or hashlib.sha256(profile.read_bytes()).hexdigest() != sys.argv[3]:
    raise RuntimeError('reviewed search driver or profiler changed')
record = {'cgroup': proof['control_group'], 'actual': actual,
          'driver_sha256': sys.argv[2], 'profiler_sha256': sys.argv[3]}
Path(controls_path).write_text(json.dumps(record, indent=2) + '\n')
sys.argv = [str(driver), sys.argv[3]]
exec(compile(raw, str(driver), 'exec'), {'__name__': '__main__', '__file__': str(driver)})
