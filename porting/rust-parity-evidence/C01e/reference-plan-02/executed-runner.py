"""Read actual staged source/ZIP observations with the frozen Rust planner."""
import hashlib,json,subprocess
from pathlib import Path
root=Path('/workspace/build/builderbob/syson-rust-parity-batch5-20261026')
out=root/'reference-plan-02';out.mkdir(exist_ok=False)
packet=root/'reference-inputs/packet-final.json'
binary=Path('/workspace/build/builderbob/syson-audit-20261026/build/target/debug/syson-reference-delta-plan')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
before={str(p):sha(p) for p in [binary,packet]}
(out/'executed-runner.py').write_bytes(Path(__file__).read_bytes())
receipt={'status':'FAIL','inputs_before':before,'independent_build':False,'C01':'OPEN'}
try:
    result=subprocess.run([str(binary),str(packet)],capture_output=True,timeout=15)
    (out/'plan.json').write_bytes(result.stdout);(out/'stderr.txt').write_bytes(result.stderr)
    receipt['exit_code']=result.returncode
    assert result.returncode==0,result.stderr.decode(errors='replace')
    plan=json.loads(result.stdout)
    assert len(plan['resources'])==114 and len(plan['families'])==7 and len(plan['unchanged_containers'])==309
    assert plan['C01']=='OPEN' and plan['independent_build'] is False
    cases=[{'id':f'image-{n:03d}','method':'GET','path':'/api/images/'+r['entry'],
            'headers':[['Accept','image/svg+xml']],'body_utf8':'',
            'expected_sha256':r['new_sha256'],'source_path':r['source_path']}
           for n,r in enumerate(plan['resources'])]
    for n,accept in enumerate(['application/json','application/graphql-response+json']):
        cases.append({'id':f'query-{n}','method':'POST','path':'/api/graphql',
                      'headers':[['Accept',accept],['Content-Type','application/json']],
                      'body_utf8':'{"query":"{ __typename }"}'})
    (out/'image-probes.json').write_text(json.dumps({'schema':1,'cases':cases},indent=2)+'\n')
    receipt.update(status='PASS_ACTUAL_SOURCE_DELTA_PLAN',plan_sha256=sha(out/'plan.json'),
                   resources=114,java_families=7,unchanged_containers=309,docs_excluded=4)
finally:
    receipt['inputs_after']={p:sha(Path(p)) for p in before}
    receipt['source_unchanged']=before==receipt['inputs_after']
    (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(receipt['status'])
