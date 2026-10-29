from pathlib import Path
import hashlib
import json
import os
import re
import subprocess
import sys

path = Path(sys.argv[1])
unit, cookie = sys.argv[4:6]
if not re.fullmatch(r'syson-u11a-profile-[0-9a-f]{32}\.scope', unit) or not re.fullmatch(r'[0-9a-f]{32}', cookie):
    raise RuntimeError('invalid profiling invocation identity')
relative = next(line.split(':', 2)[2] for line in Path('/proc/self/cgroup').read_text().splitlines()
                if line.startswith('0::'))
if (not relative.startswith('/') or not relative.endswith('/' + unit)
        or '//' in relative or any(part in ('.', '..') for part in relative.split('/'))):
    raise RuntimeError('unexpected profiling cgroup')
state = subprocess.run(['/usr/bin/systemctl', '--user', 'show', unit, '--property=ControlGroup',
                        '--property=InvocationID'], capture_output=True, timeout=10, check=True)
values = dict(line.split('=', 1) for line in state.stdout.decode().splitlines() if '=' in line)
if (values.get('ControlGroup') != relative
        or not re.fullmatch(r'[0-9a-f]{32}', values.get('InvocationID', ''))
        or values.get('InvocationID') == '0' * 32):
    raise RuntimeError('profiling scope invocation is unobservable')
start_ticks = Path('/proc/self/stat').read_text().rsplit(')', 1)[1].split()[19]
proof = {'unit': unit, 'cookie': cookie, 'control_group': relative,
         'invocation_id': values['InvocationID'], 'entry_pid': os.getpid(), 'entry_start_ticks': start_ticks}
with (path.parent / 'u11a-profile-resource-01/ownership.json').open('x') as target:
    target.write(json.dumps(proof, indent=2) + '\n')
raw = path.read_bytes()
if hashlib.sha256(raw).hexdigest() != sys.argv[2]:
    raise RuntimeError('reviewed profiling driver changed before compilation')
profile = path.parent / 'u11a_profile.rs'
if hashlib.sha256(profile.read_bytes()).hexdigest() != sys.argv[3]:
    raise RuntimeError('reviewed profiler changed before execution')
group = Path('/sys/fs/cgroup') / relative.lstrip('/')
expected = {'cpu.max': '200000 100000', 'memory.high': '4294967296',
            'memory.max': '6442450944', 'memory.swap.max': '0', 'pids.max': '128'}
actual = {name: (group / name).read_text().strip() for name in expected}
if actual != expected:
    raise RuntimeError('actual profiling resource controls differ')
record = {'cgroup': relative, 'actual': actual, 'driver_sha256': sys.argv[2],
          'profiler_sha256': sys.argv[3]}
(path.parent / 'u11a-profile-resource-01/actual-controls.json').write_text(json.dumps(record, indent=2) + '\n')
sys.argv = [str(path), sys.argv[3]]
exec(compile(raw, str(path), 'exec'), {'__name__': '__main__', '__file__': str(path)})
