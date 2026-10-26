#!/usr/bin/env python3
"""Retain actual read-only CLI behavior and exact native descriptor equality."""
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys

unit=sys.argv[1]
config={'C01c':('rp-c01c-graphql-contract','c01c-03','syson-native-graphql-contract','native-02/introspection-response.json'),
        'C01d':('rp-c01d-persistence-contract','c01d-01','syson-native-db-contract','native-01/catalog-stdout.txt')}
folder,capture_name,name,relative=config[unit]
base=Path('/workspace/build/builderbob/syson-rust-parity-batch4-20261026')
out=base/(unit.lower()+'-cli-01');out.mkdir()
(out/'executed-runner.py').write_bytes(Path(__file__).read_bytes())
root=Path('/workspace/worktrees/builderbob/syson')/folder
capture=json.loads((base/'captures'/capture_name/'receipt.json').read_text())
assert capture['status']=='PASS' and capture['source_unchanged']
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
freeze=lambda:{p:sha(root/p) for p in capture['source_after']}
before=freeze();assert before==capture['source_after']
program=Path('/workspace/build/builderbob/syson-audit-20261026/build/target/debug')/name
native=base/relative
directory=out/'directory';directory.mkdir()
symlink=out/'input-symlink.json';symlink.symlink_to(native)
bad=base/'native-01/capabilities-response.json' if unit=='C01c' else out/'malformed.json'
if unit=='C01d':bad.write_text('{"capture":0}\n')
cases=[('native',[str(native)],0),('refusal',[str(bad)],1),('usage',[],1),('directory',[str(directory)],1),('symlink',[str(symlink)],1)]
if unit=='C01c':cases += [('query',['--query'],0),('capabilities',['--capabilities'],0)]
records=[];errors=[]
for label,args,expected in cases:
    argv=[str(program),*args]
    child=subprocess.Popen(argv,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
    try:stdout,stderr=child.communicate(timeout=30);code=child.returncode
    except subprocess.TimeoutExpired:
        os.killpg(child.pid,signal.SIGKILL);stdout,stderr=child.communicate(timeout=3);code=124
    (out/(label+'-stdout.txt')).write_bytes(stdout);(out/(label+'-stderr.txt')).write_bytes(stderr)
    records.append(dict(case=label,argv=argv,exit_code=code,expected_exit_code=expected,stdout_sha256=hashlib.sha256(stdout).hexdigest(),stderr_sha256=hashlib.sha256(stderr).hexdigest()))
    if code!=expected:errors.append(label+': unexpected exit '+str(code))
native_output=json.loads((out/'native-stdout.txt').read_text())
original=json.loads(native.read_text())
if unit=='C01c':
    assert native_output['schema']==original['data']['__schema']
    assert native_output['C01']=='OPEN'
    assert (out/'query-stdout.txt').read_bytes()==(root/'porting/rust-parity-evidence/C01c/introspection.graphql').read_bytes()
    assert (out/'capabilities-stdout.txt').read_bytes()==(root/'porting/rust-parity-evidence/C01c/capabilities-single.graphql').read_bytes()
    assert b'GraphQL errors' in (out/'refusal-stderr.txt').read_bytes()
    equality=dict(full_selected_schema=True,counts=native_output['counts'])
else:
    assert native_output['capture']==original['capture']
    counts={}
    for source,kind in [('tables','table'),('columns','column'),('constraints','constraint'),('indexes','index'),('changelog','migration')]:
        projected=[r['observed'] for r in native_output['obligations'] if r['kind']==kind]
        canonical=lambda rows:sorted(json.dumps(row,sort_keys=True) for row in rows)
        assert canonical(projected)==canonical(original[source]),kind
        counts[kind]=len(projected)
    by_id={r['id']:r for r in native_output['obligations']}
    assert [by_id[k]['observed']['position'] for k in native_output['migration_order']]==list(range(len(original['changelog'])))
    assert all(r['state']=='NATIVE_CAPTURED_RUST_COMPARISON_PENDING' for r in native_output['obligations'])
    assert native_output['parent_C01']=='OPEN' and native_output['schema_parity']=='NOT_ESTABLISHED'
    equality=dict(all_descriptors=True,history_order=True,counts=counts,obligations=len(native_output['obligations']))
after=freeze()
receipt=dict(unit=unit,status='PASS' if not errors and before==after else 'FAIL',errors=errors,source_before=before,source_after=after,source_unchanged=before==after,program_sha256=sha(program),native_input=str(native),native_input_sha256=sha(native),commands=records,equality=equality,boundary='Observed current native contracts, not native/Rust behavior, whole-source rebuild, resolver/auth/UI or durable transaction acceptance.')
(out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({k:receipt[k] for k in ['unit','status','source_unchanged','equality']}))
assert receipt['status']=='PASS'
