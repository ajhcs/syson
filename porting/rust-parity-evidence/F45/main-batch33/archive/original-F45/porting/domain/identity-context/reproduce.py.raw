#!/usr/bin/env python3
"""Verify pinned F45 inputs and new local harness metadata without native execution."""
import hashlib,json,pathlib,re,subprocess,tomllib,uuid
ROOT=pathlib.Path(__file__).resolve().parents[3]
OUT=pathlib.Path(__file__).resolve().parent
c=json.loads((OUT/'SOURCE-CONTRACT.json').read_text())
for s in c['sources']:
    raw=subprocess.check_output(['git','-C',str(ROOT),'show',s['revision']+':'+s['path']])
    assert hashlib.sha256(raw).hexdigest()==s['sha256'] and (ROOT/s['path']).read_bytes()==raw,s['path']
old=tomllib.loads((ROOT/'rust/crates/syson-workflow/Cargo.lock').read_text())['package']
new=tomllib.loads((OUT/'integration/Cargo.lock').read_text())['package']
old_registry={p['name']:p for p in old if 'source' in p}
new_registry={p['name']:p for p in new if 'source' in p}
assert new_registry==old_registry, 'Registry package record drift'
old_local={p['name']:p for p in old if 'source' not in p}
new_local={p['name']:p for p in new if 'source' not in p}
assert set(old_local)-set(new_local)=={'syson-initializer','syson-initializer-graph'}
assert set(new_local)-set(old_local)=={'syson-identity-context-integration'}
expected={k:dict(v) for k,v in old_local.items() if k not in {'syson-initializer','syson-initializer-graph'}}
expected['syson-workflow']['dependencies']=['syson-editor','syson-model','syson-persistence']
expected['syson-identity-context-integration']={'name':'syson-identity-context-integration','version':'0.1.0','dependencies':['syson-domain','syson-model','syson-workflow']}
assert new_local==expected, 'Unexpected local package graph drift'
manifest=tomllib.loads((OUT/'integration/Cargo.toml').read_text())
assert manifest['dependencies']=={k:{'path':v} for k,v in c['integration_dependencies'].items()}
for key,relative in c['integration_dependencies'].items():
    assert (OUT/'integration'/relative/'Cargo.toml').resolve().is_file(),key
for p,n in [(ROOT/'rust/crates/syson-domain/tests/identity_context.rs',18),(OUT/'integration/tests/snapshots.rs',1)]:
    assert len(re.findall(r'^#\[test\]',p.read_text(),re.M))==n
f=c['unicode_namespace_fixture'];assert str(uuid.uuid5(uuid.UUID(f['namespace']),f['name']))==f['uuid']
validation=json.loads((ROOT/'porting/follow-ons/F45/VALIDATION.json').read_text())
assert validation['new_domain_tests_passed']==18 and validation['existing_domain_regressions_passed']==28
assert validation['actual_file_backed_integration_tests_passed']==1
for capture in validation['captures']:
    receipt=ROOT/'porting/follow-ons/F45'/capture['receipt']
    assert hashlib.sha256(receipt.read_bytes()).hexdigest()==capture['original_receipt_sha256']
    observed=json.loads(receipt.read_text())
    assert observed['status']==capture['status']
    if observed['status']=='PASS':
        assert observed['source_unchanged']
        for path,expected_hash in observed['source_before'].items():
            assert hashlib.sha256((ROOT/path).read_bytes()).hexdigest()==expected_hash,path
print(json.dumps({'pinned_sources':len(c['sources']),'registry_package_changes':0,'new_local_packages':1,'pruned_inactive_local_packages':2,'coordinator_recorded_domain_test_passes':46,'coordinator_recorded_file_integration_passes':1,'passing_input_hashes':'UNCHANGED','upstream_runtime':'NOT_RUN'}))
