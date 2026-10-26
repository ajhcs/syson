"""Actual full-corpus physical-index comparison; no semantic resolution claim."""
import hashlib,json,os,signal,subprocess
from pathlib import Path
base=Path('/workspace/build/builderbob/syson-rust-parity-batch6-20261026')
root=base/'library-inputs';out=base/'library-cli-01';out.mkdir(exist_ok=False)
binary=Path('/workspace/build/builderbob/syson-audit-20261026/build/target/release/syson-native-library-catalog')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
manifest=root/'manifest.json';rows=json.loads(manifest.read_bytes())
native=base/'library-native-01/native-libraries.json'
inputs=[binary,manifest,native,Path('/usr/bin/time'),*[root/r['file'] for r in rows]]
frozen={str(p):sha(p) for p in inputs}
(out/'executed-runner.py').write_bytes(Path(__file__).read_bytes())
receipt={'status':'FAIL','inputs_before':frozen,'scope':'Physical stored identity/name/eClass tuple mapping only, no native traversal order, resolved references or semantic fragment equivalence'}
try:
    argv=['/usr/bin/time','-v','-o',str(out/'time.txt'),str(binary),str(root),'manifest.json']
    with (out/'index.json').open('wb') as stdout,(out/'stderr.txt').open('wb') as stderr:
        child=subprocess.Popen(argv,stdout=stdout,stderr=stderr,start_new_session=True)
        try:code=child.wait(timeout=180)
        except subprocess.TimeoutExpired:os.killpg(child.pid,signal.SIGKILL);child.wait(timeout=5);code=124
    receipt.update(argv=argv,pid=child.pid,exit_code=code);assert code==0,'CLI failed; retain streams'
    a=json.loads(native.read_bytes());b=json.loads((out/'index.json').read_bytes())
    raw={r['resource_uri']:r for r in b['resources']};actual={r['resource_uri']:r for r in a['resources']}
    assert len(raw)==len(b['resources']) and set(raw)==set(actual)
    total=0
    for uri,r in raw.items():
        left={n['node_id']:n for n in r['nodes']};right={n['node_id']:n for n in actual[uri]['nodes']}
        assert len(left)==len(r['nodes']) and set(left)==set(right),(uri,'identity set mismatch')
        for id,n in left.items():
            m=right[id]
            assert n['semantic_id']==m['semantic_id'] and n['declared_name']==m['declared_name'],(uri,id,'stored data mismatch')
            assert n['e_class']=='sysml:'+m['e_class'] and m['package_uri']=='http://www.eclipse.org/syson/sysml',(uri,id,'namespace-qualified class mismatch')
            total+=1
    lookups=[n for r in a['resources'] for n in r['nodes'] if 'node_lookup_same' in n]
    receipt.update(status='PASS_94_RESOURCE_109673_PHYSICAL_NODE_MAPPING',resources=len(raw),physical_nodes=total,
                   native_diagnostics=sum(len(r['errors']) for r in a['resources']),index_sha256=sha(out/'index.json'),index_bytes=(out/'index.json').stat().st_size,
                   native_fragment_lookup_samples=len(lookups),semantic_fragment_lookup_false=sum(not n['semantic_lookup_same'] for n in lookups),
                   namespace_projection='Raw sysml:EClass prefix paired with native package_uri http://www.eclipse.org/syson/sysml; keyed by resourceURI and stored nodeID; no ordering or reference normalization.')
finally:
    receipt['inputs_after']={p:sha(Path(p)) for p in frozen};receipt['source_unchanged']=receipt['inputs_after']==frozen
    (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({k:v for k,v in receipt.items() if k not in ['inputs_before','inputs_after','argv']}))
