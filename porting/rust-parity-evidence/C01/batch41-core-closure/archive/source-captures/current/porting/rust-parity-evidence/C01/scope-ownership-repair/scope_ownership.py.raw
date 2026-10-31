"""Invocation-bound scope ownership; unknown or reused units are preserved."""
from pathlib import Path
import json
import os
import re
import subprocess
import time
import uuid


def require(ok, message):
    if not ok:
        raise RuntimeError(message)


def new_attempt(prefix):
    return prefix + uuid.uuid4().hex + '.scope', uuid.uuid4().hex


def validate(unit, cookie, prefix):
    require(re.fullmatch(re.escape(prefix) + r'[0-9a-f]{32}\.scope', unit)
            and re.fullmatch(r'[0-9a-f]{32}', cookie), 'invalid invocation identity')


def state(unit, env, run=subprocess.run):
    result = run(['/usr/bin/systemctl', '--user', 'show', unit,
                  '--property=LoadState', '--property=ControlGroup', '--property=InvocationID'],
                 env=env, capture_output=True, timeout=10)
    values = dict(line.split('=', 1) for line in result.stdout.decode().splitlines() if '=' in line)
    require(result.returncode in (0, 4) and values.get('LoadState') in ('loaded', 'not-found'),
            'scope state is unobservable')
    return values


def identity(relative, invocation, unit):
    require(relative.startswith('/') and relative.endswith('/' + unit)
            and all(part not in ('', '.', '..') for part in relative.split('/')[1:])
            and '\x00' not in relative, 'preserve scope: noncanonical cgroup identity')
    require(re.fullmatch(r'[0-9a-f]{32}', invocation) and invocation != '0' * 32,
            'preserve scope: invalid invocation ID')


def attest(unit, cookie, prefix, destination, env):
    validate(unit, cookie, prefix)
    relative = next(line[3:] for line in Path('/proc/self/cgroup').read_text().splitlines()
                    if line.startswith('0::'))
    observed = state(unit, env)
    identity(relative, observed.get('InvocationID', ''), unit)
    require(observed.get('LoadState') == 'loaded' and observed.get('ControlGroup') == relative,
            'entry invocation is unobservable')
    proof = {'unit': unit, 'cookie': cookie, 'control_group': relative,
             'invocation_id': observed['InvocationID'], 'entry_pid': os.getpid(),
             'entry_start_ticks': Path('/proc/self/stat').read_text().rsplit(')', 1)[1].split()[19]}
    with Path(destination).open('x') as target:
        target.write(json.dumps(proof, indent=2) + '\n')
    return proof


def owned(observed, proof, unit, cookie):
    relative = observed.get('ControlGroup', '')
    identity(relative, observed.get('InvocationID', ''), unit)
    require(proof.get('unit') == unit and proof.get('cookie') == cookie
            and proof.get('control_group') == relative
            and proof.get('invocation_id') == observed['InvocationID']
            and isinstance(proof.get('entry_pid'), int) and proof['entry_pid'] > 0
            and str(proof.get('entry_start_ticks', '')).isdigit(),
            'preserve scope: invocation ownership is unproven or differs')


def cleanup(unit, cookie, proof_path, env, run=subprocess.run,
            root=Path('/sys/fs/cgroup'), pause=time.sleep):
    observations = []
    for attempt in range(5):
        observed = state(unit, env, run)
        observations.append(observed)
        if observed['LoadState'] == 'not-found':
            return {'confirmed_empty_or_absent': True, 'observations': observations}
        try:
            proof = json.loads(Path(proof_path).read_bytes())
        except (OSError, ValueError) as error:
            raise RuntimeError('preserve scope: missing invocation proof') from error
        owned(observed, proof, unit, cookie)
        group = root / observed['ControlGroup'].lstrip('/')
        try:
            observed['events'] = (group / 'cgroup.events').read_text()
            observed['processes'] = (group / 'cgroup.procs').read_text()
            if 'populated 0' in observed['events'].splitlines() and not observed['processes'].strip():
                return {'confirmed_empty_or_absent': True, 'observations': observations}
        except FileNotFoundError:
            pass
        if attempt == 0:
            killed = run(['/usr/bin/systemctl', '--user', 'kill', '--kill-whom=all',
                          '--signal=SIGKILL', unit], env=env, capture_output=True, timeout=10)
            observed['kill_exit'] = killed.returncode
        pause(0.2)
    raise RuntimeError('owned scope cleanup not confirmed')
