"""Root-only selected-source compilation; no Maven, processors or network."""
import fcntl
import hashlib
import json
import os
from pathlib import Path
import subprocess

root=Path('/workspace/build/builderbob/syson-rust-parity-batch5-20261026')
staged=root/'reference-inputs'
out=root/'reference-compile-01';out.mkdir(exist_ok=False)
javac=Path('/workspace/cache/builderbob/global-tools/releases/jdk/26.0.2.1+1/bin/javac')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(javac)=='e21c219e342544cbf107d25e86ed1d809eac6cc57bce8f68750a42783f13e023'
packet=json.loads((staged/'packet-precompile.json').read_text())
declared=json.loads((staged/'javac-inputs.json').read_text())
sources=[Path(p) for p in declared['sources']]
assert len(sources)==7 and not any((staged/'classes').iterdir())
assert not any((staged/'empty-sourcepath').iterdir())
expected={str(staged/'lib'/Path(c['entry']).name):c['sha256'] for c in packet['containers']}
assert len(expected)==312 and set(expected)=={str(p) for p in (staged/'lib').iterdir()}
expected.update(declared['source_sha256'])
assert all(sha(Path(p))==h for p,h in expected.items())
argv=[str(javac),'-J-Xmx512m','-J-XX:ActiveProcessorCount=2',*declared['args']]
lock=open('/workspace/build/builderbob/syson-audit-20261026/build/compiler.lock','a')
fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
receipt={'status':'FAIL','argv':argv,'inputs_before':expected,'javac_sha256':sha(javac),
         'source_processors':'DISABLED','independent_build':False,'C01':'OPEN'}
(out/'executed-runner.py').write_bytes(Path(__file__).read_bytes())
try:
    result=subprocess.run(argv,capture_output=True,timeout=120,env=dict(os.environ,JDK_JAVAC_OPTIONS='',JAVA_TOOL_OPTIONS=''))
    (out/'stdout.txt').write_bytes(result.stdout);(out/'stderr.txt').write_bytes(result.stderr)
    receipt['exit_code']=result.returncode
    assert result.returncode==0,result.stderr.decode(errors='replace')
    compiled=[]
    for p in sorted((staged/'classes').rglob('*')):
        if not p.is_file():continue
        entry=p.relative_to(staged/'classes').as_posix();assert entry.endswith('.class')
        raw=p.read_bytes();assert raw[:4]==b'\xca\xfe\xba\xbe'
        candidates=[s for s in sources if entry==s.name.removesuffix('.java')+'.class' or
                    entry.endswith('/'+s.name.removesuffix('.java')+'.class') or
                    ('/'+s.name.removesuffix('.java')+'$') in '/'+entry]
        assert len(candidates)==1,(entry,candidates)
        source_path=candidates[0].relative_to(staged/'src').as_posix()
        major=int.from_bytes(raw[6:8],'big');assert major==65
        compiled.append({'source_path':source_path,'entry':entry,'sha256':sha(p),'bytes':len(raw),'major':major})
    assert 7<=len(compiled)<=128
    receipt['compiled']=compiled;receipt['status']='PASS_SELECTED_7_SOURCE_COMPILE'
    (out/'compiled.json').write_text(json.dumps(compiled,indent=2)+'\n')
finally:
    receipt['inputs_after']={p:sha(Path(p)) for p in expected}
    receipt['source_unchanged']=receipt['inputs_after']==expected
    (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(receipt['status'],len(receipt.get('compiled',[])))
