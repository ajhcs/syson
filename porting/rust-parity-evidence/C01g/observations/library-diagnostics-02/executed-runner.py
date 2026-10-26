"""Coordinator-only existing native library configuration probe; no DB/network."""
import fcntl,hashlib,json,os,signal,subprocess
from pathlib import Path
base=Path('/workspace/build/builderbob/syson-rust-parity-batch6-20261026')
out=base/'library-diagnostics-02';out.mkdir(exist_ok=False)
source=Path('/workspace/repos/builderbob/syson/porting/follow-ons/parity-batch6-20261026/LibraryDiagnostics.java')
classpath=json.loads((base/'native-classpath.json').read_text())
javac=Path('/workspace/cache/builderbob/global-tools/releases/jdk/26.0.2.1+1/bin/javac')
java=Path('/workspace/build/builderbob/ajhcs-sysml-java21-20261004/jdk-21.0.12.1+1-jre/bin/java')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(javac)=='e21c219e342544cbf107d25e86ed1d809eac6cc57bce8f68750a42783f13e023'
assert sha(java)=='999c141a6e620c71aa4230ccb62341a89162f37818753ff006d766bf935dcbe1'
frozen={r['path']:r['sha256'] for r in classpath['classpath']}
frozen.update({str(p):sha(p) for p in [source,javac,java,base/'native-classpath.json']})
assert all(sha(Path(p))==h for p,h in frozen.items())
classes=out/'classes';classes.mkdir();empty=out/'empty-sourcepath';empty.mkdir()
(out/'tmp').mkdir();cp=os.pathsep.join(r['path'] for r in classpath['classpath'])
lock=open('/workspace/build/builderbob/syson-audit-20261026/build/compiler.lock','a');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
commands=[[str(javac),'-J-Xmx512m','-J-XX:ActiveProcessorCount=2','--release','21','-proc:none','-implicit:none','-sourcepath',str(empty),'-classpath',cp,'-d',str(classes),str(source)],
          [str(java),'-Xmx1536m','-XX:ActiveProcessorCount=2','-Dstdout.encoding=UTF-8','-Dstderr.encoding=UTF-8','-Djava.io.tmpdir='+str(out/'tmp'),'-classpath',str(classes)+os.pathsep+cp,'LibraryDiagnostics',str(out/'native-diagnostics.json')]]
(out/'executed-runner.py').write_bytes(Path(__file__).read_bytes());(out/'LibraryDiagnostics.java').write_bytes(source.read_bytes())
receipt={'status':'FAIL','inputs_before':frozen,'commands':[],'scope':'Real default config/ResourceSet only; no full Spring application, database, URI loading parity or independent source build','owned_processes_stopped':False}
try:
    for i,argv in enumerate(commands):
        env={k:os.environ[k] for k in ['HOME','PATH']};env.update(LANG='C.UTF-8',LC_ALL='C.UTF-8')
        with (out/f'{i:02d}-stdout.txt').open('wb') as stdout,(out/f'{i:02d}-stderr.txt').open('wb') as stderr:
            child=subprocess.Popen(argv,stdout=stdout,stderr=stderr,env=env,start_new_session=True)
            try:code=child.wait(timeout=180)
            except subprocess.TimeoutExpired:
                os.killpg(child.pid,signal.SIGKILL);child.wait(timeout=5);code=124
            receipt['commands'].append({'argv':argv,'pid':child.pid,'exit_code':code})
            assert code==0,'native command failed; retain raw streams'
    observation=json.loads((out/'native-diagnostics.json').read_bytes())
    receipt.update(status='PASS_ACTUAL_LIBRARY_DIAGNOSTIC_DETAILS',resources=len(observation['resources']),errors=sum(len(r['errors']) for r in observation['resources']),warnings=sum(len(r['warnings']) for r in observation['resources']),reason='Native01 contained12162opaquePackageNotFoundError identities; this freshconfiguration run observes previously uncaptured diagnostic details only, no node parity rerun.',native_json_sha256=sha(out/'native-diagnostics.json'))
finally:
    receipt['inputs_after']={p:sha(Path(p)) for p in frozen};receipt['source_unchanged']=receipt['inputs_after']==frozen
    receipt['owned_processes_stopped']=all(not Path('/proc',str(c['pid'])).exists() for c in receipt['commands'])
    (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({k:v for k,v in receipt.items() if k not in ['inputs_before','inputs_after','commands']}))
