from copy import deepcopy
from hashlib import sha256
from pathlib import Path
from subprocess import Popen, TimeoutExpired
from time import monotonic_ns
import json
import os
import resource
import signal

root = Path.cwd()
exe = root / 'target/release/syson-public-snapshot-process-probe'
capture = root / 'capture'
state = root / 'states'
state.mkdir(mode=0o700)
key = 'rig.emf.json'
staged_key = 'rig.emf.staged'

def rid(i): return f'71000000-0000-4000-8000-{i:012x}'

def sid(i): return f'72000000-0000-4000-8000-{i:012x}'

def object_at(index, kind, attributes):
    return {'eClass': f'sysml:{kind}', 'id': rid(index), 'data': {'elementId': sid(index), **attributes}}

# Build the six expected objects independently of child stdout or the saved file.
expected = {
    'json': {'version': '1.0', 'encoding': 'utf-8'},
    'ns': {'sysml': 'http://www.eclipse.org/syson/sysml'},
    'content': [object_at(1, 'Package', {
        'declaredName': 'Rig',
        'ownedRelationship': [
            object_at(3, 'OwningMembership', {
                'ownedRelatedElement': [object_at(2, 'PartDefinition', {'declaredName': 'Link Rev A <&> 雪🧭'})]
            }),
            object_at(5, 'OwningMembership', {
                'ownedRelatedElement': [object_at(4, 'PartUsage', {
                    'declaredName': 'link',
                    'isComposite': True,
                    'ownedRelationship': [object_at(6, 'FeatureTyping', {'type': sid(2), 'typedFeature': sid(4)})],
                })]
            })
        ]
    })]
}

def canonical(document):
    return json.dumps(document, indent=2, ensure_ascii=False, sort_keys=True).encode()

expected_bytes = canonical(expected)
(capture / 'expected-rig.emf.json').write_bytes(expected_bytes)
alternate = deepcopy(expected)
alternate['content'][0]['data']['ownedRelationship'][0]['data']['ownedRelatedElement'][0]['data']['declaredName'] = 'Unpublishable alternative'
alternate_bytes = canonical(alternate)
rows = {}

def cap_child_output():
    resource.setrlimit(resource.RLIMIT_FSIZE, (65536, 65536))

def launch(label, mode, storage, expected_status):
    pre = monotonic_ns()
    out_path = capture / (label + '.stdout')
    err_path = capture / (label + '.stderr')
    with out_path.open('xb') as out, err_path.open('xb') as err:
        child = Popen([str(exe), mode, str(storage)], cwd=storage, stdout=out, stderr=err,
                      preexec_fn=cap_child_output, start_new_session=True)
        try:
            status = child.wait(timeout=10)
        except TimeoutExpired:
            os.killpg(child.pid, signal.SIGKILL)
            child.wait(timeout=2)
            raise
    post = monotonic_ns()
    assert status == expected_status, (label,status,err_path.read_text())
    assert not Path(f'/proc/{child.pid}').exists()
    stdout, stderr = out_path.read_bytes(), err_path.read_bytes()
    assert len(stdout) < 65536 and len(stderr) < 65536
    active = json.loads((stderr if status else stdout).decode())
    assert active['pid'] == child.pid
    assert not (stdout if status else stderr)
    rows[label] = {'mode':mode, 'pid':child.pid, 'exit':status, 'proc_entry_absent_after_wait':True,
                   'start_monotonic_ns':pre, 'reaped_monotonic_ns':post,
                   'stdout_sha256':sha256(stdout).hexdigest(),'stderr_sha256':sha256(stderr).hexdigest()}
    return active

def regular_state(path):
    s=path.stat()
    return (s.st_dev,s.st_ino,s.st_size,s.st_mtime_ns)

good = state / 'good'
good.mkdir(mode=0o700)
writer = launch('01-writer','write',good,0)
assert writer['allocations'] == 6 and writer['snapshot'] == key
assert writer['document'] == expected
snapshot = good / key
assert sorted(x.name for x in good.iterdir()) == [key]
assert snapshot.read_bytes() == expected_bytes, 'independently constructed canonical bytes differ from saved bytes'
baseline = regular_state(snapshot)

reader = launch('02-reader','read',good,0)
assert reader['document'] == expected
assert rows['01-writer']['reaped_monotonic_ns'] < rows['02-reader']['start_monotonic_ns']
assert reader['pid'] != writer['pid']
assert snapshot.read_bytes() == expected_bytes and regular_state(snapshot) == baseline

malformed = state / 'malformed'
malformed.mkdir(mode=0o700)
refusal = launch('03-malformed-import','malformed',malformed,2)
assert refusal['error'].startswith('stage: Parse(Malformed'), refusal
assert not list(malformed.iterdir()), 'invalid import published or staged data'

no_clobber = launch('04-duplicate-key','overwrite',good,2)
assert no_clobber['error'].startswith('save: AlreadyExists'), no_clobber
assert sorted(x.name for x in good.iterdir()) == [key,staged_key]
assert snapshot.read_bytes() == expected_bytes and regular_state(snapshot) == baseline
assert (good / staged_key).read_bytes() == alternate_bytes
again = launch('05-reader-after-duplicate','read',good,0)
assert again['document'] == expected
assert snapshot.read_bytes() == expected_bytes and regular_state(snapshot) == baseline

damaged = state / 'truncated'
damaged.mkdir(mode=0o700)
truncated = damaged / key
truncated.write_bytes(expected_bytes[:-13])
bad_bytes = truncated.read_bytes()
bad = launch('06-reader-truncated','read',damaged,2)
assert bad['error'].startswith('load: Model(Json('), bad
assert sorted(x.name for x in damaged.iterdir()) == [key] and truncated.read_bytes() == bad_bytes
final = launch('07-reader-after-truncated','read',good,0)
assert final['document'] == expected and snapshot.read_bytes() == expected_bytes and regular_state(snapshot) == baseline
assert len(rows) == len({row['pid'] for row in rows.values()}) == 7
print(json.dumps({
    'status':'PASS_BOUNDED_PUBLIC_API_PROCESS_REOPEN',
    'processes':rows,
    'fixed_ids':[[rid(i),sid(i)] for i in range(1,7)],
    'typed_reference_endpoints':{'type':sid(2),'typedFeature':sid(4)},
    'rename':'Link Rev A <&> 雪🧭',
    'independent_canonical_bytes':len(expected_bytes),
    'published_sha256':sha256(expected_bytes).hexdigest(),
    'published_file_dev_inode_size_mtime_ns':baseline,
    'duplicate_diagnostic_stage':{'path':staged_key,'sha256':sha256(alternate_bytes).hexdigest()},
    'truncated_sha256':sha256(bad_bytes).hexdigest(),
    'refusals':{'malformed_import':refusal['error'],'no_clobber':no_clobber['error'],'truncated_snapshot':bad['error']},
    'timings_single_sample_ns':{'writer':writer['observed_ns'],'reader':reader['observed_ns']},
    'feature_acceptance':False
},indent=2,ensure_ascii=False))
