"""Explicit byte-slice caller projections; comparator has no ignore/profile mode."""
import hashlib,json,subprocess
from pathlib import Path
base=Path('/workspace/build/builderbob/syson-rust-parity-batch6-20261026');out=base/'diff-cli-01';out.mkdir(exist_ok=False)
binary=Path('/workspace/build/builderbob/syson-audit-20261026/build/target/debug/syson-exact-json-diff')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
decoder=json.JSONDecoder()
def member(raw,keys):
    if not keys:return raw
    i=0
    while raw[i].isspace():i+=1
    assert raw[i]=='{';i+=1
    while True:
        while raw[i].isspace():i+=1
        key,i=decoder.raw_decode(raw,i)
        while raw[i].isspace():i+=1
        assert raw[i]==':';i+=1
        while raw[i].isspace():i+=1
        start=i;_,i=decoder.raw_decode(raw,i)
        if key==keys[0]:return member(raw[start:i],keys[1:])
        while raw[i].isspace():i+=1
        assert raw[i]==',';i+=1
p4=Path('/workspace/build/builderbob/syson-rust-parity-batch4-20261026');p5=Path('/workspace/build/builderbob/syson-rust-parity-batch5-20261026');w=Path('/workspace/worktrees/builderbob/syson')
pairs=[('graphql',w/'rp-c01c-graphql-contract/rust/tools/native-graphql-contract/tests/fixtures/native-official-label-introspection.json',['data','__schema'],p4/'c01c-cli-02/native-stdout.txt',['schema']),
 ('catalog-metadata',w/'rp-c01d-persistence-contract/rust/tools/native-db-contract/tests/fixtures/native-01-catalog.json',['capture'],p4/'c01d-cli-02/native-stdout.txt',['capture']),
 ('mappings',p5/'http-native-01/native-mappings.json',[],p5/'http-cli-02/mappings-stdout.json',['mappings']),
 ('probes',p5/'http-native-01/native-probes.json',[],p5/'http-cli-02/probes-stdout.json',['capture'])]
frozen={str(p):sha(p) for p in [binary,*[p for _,a,_,b,_ in pairs for p in [a,b]]]}
receipt={'status':'FAIL','inputs_before':frozen,'cases':[],'projections':[],'scope':'Complete supplied projected trees; full DB schema, Rust handlers and feature parity not inferred. No reordering, numeric conversion or ignored fields inside projected subtree.'}
(out/'executed-runner.py').write_bytes(Path(__file__).read_bytes())
def run(name,left,right,expected=None):
    r=subprocess.run([str(binary),str(left),str(right)],capture_output=True,timeout=30)
    (out/(name+'-stdout.json')).write_bytes(r.stdout);(out/(name+'-stderr.txt')).write_bytes(r.stderr)
    if expected is not None:assert r.returncode==expected,(name,r.stderr)
    assert r.returncode in [0,1],(name,r.stderr)
    result=json.loads(r.stdout);assert result['status']==('MATCH' if r.returncode==0 else 'MISMATCH')
    receipt['cases'].append({'name':name,'exit_code':r.returncode,'status':result['status'],'complete':result['complete'],'reported_differences':len(result['differences']),'left_sha256':sha(left),'right_sha256':sha(right)})
try:
    for name,a,ak,b,bk in pairs:
        left=out/(name+'-native.json');right=out/(name+'-reader.json')
        left.write_text(member(a.read_text(),ak));right.write_text(member(b.read_text(),bk))
        receipt['projections'].append({'name':name,'left_source':str(a),'left_members':ak,'right_source':str(b),'right_members':bk,'operation':'Original UTF8 value-token byte slices; parser used only to find boundaries, never to serialize or normalize.'})
        run(name+'-native-self',left,left,0)
        run(name+'-reader-observation',left,right)
    changed=out/'mutated-query-id.json';raw=(out/'graphql-native.json').read_bytes();assert b'"Query"' in raw;changed.write_bytes(raw.replace(b'"Query"',b'"Query_CHANGED"',1));run('actual-identity-mutation',out/'graphql-native.json',changed,1)
    bad=out/'duplicate.json';bad.write_text('{"id":1,"id":2}');link=out/'symlink.json';link.symlink_to(out/'graphql-native.json')
    for name,args in [('usage',[]),('duplicate',[str(bad),str(bad)]),('symlink',[str(link),str(link)])]:
        r=subprocess.run([str(binary),*args],capture_output=True,timeout=5);(out/(name+'-stdout.txt')).write_bytes(r.stdout);(out/(name+'-stderr.txt')).write_bytes(r.stderr)
        assert r.returncode==2 and not r.stdout
        receipt['cases'].append({'name':name,'exit_code':2,'no_partial_stdout':True})
    receipt['status']='PASS_STRICT_OBSERVATION_COMPARATOR;_ACTUAL_DIFFERENCES_RETAINED'
finally:
    receipt['inputs_after']={p:sha(Path(p)) for p in frozen};receipt['source_unchanged']=receipt['inputs_after']==frozen
    (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps(receipt['cases']))
