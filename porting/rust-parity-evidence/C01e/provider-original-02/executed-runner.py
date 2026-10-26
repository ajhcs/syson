"""Owned dev-only selected native provider smoke, exact current classpath."""
import fcntl,hashlib,json,os,subprocess,sys
from pathlib import Path
root=Path('/workspace/build/builderbob/syson-rust-parity-batch5-20261026')
derived=len(sys.argv)>1 and sys.argv[1]=='derived'
out=root/('provider-derived-01' if derived else 'provider-original-02');out.mkdir(exist_ok=False)
source=Path(__file__).with_name('ProviderProbe.java')
inputs=root/'reference-inputs'
lib=root/'reference-repack-01/lib' if derived else inputs/'lib'
javac=Path('/workspace/cache/builderbob/global-tools/releases/jdk/26.0.2.1+1/bin/javac')
java=Path('/workspace/build/builderbob/ajhcs-sysml-java21-20261004/jdk-21.0.12.1+1-jre/bin/java')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(javac)=='e21c219e342544cbf107d25e86ed1d809eac6cc57bce8f68750a42783f13e023'
assert sha(java)=='999c141a6e620c71aa4230ccb62341a89162f37818753ff006d766bf935dcbe1'
jars=sorted(lib.glob('*.jar'));assert len(jars)==312
classpath=os.pathsep.join(map(str,jars));classes=out/'classes';classes.mkdir()
frozen={str(p):sha(p) for p in [source,javac,java,*jars]}
lock=open('/workspace/build/builderbob/syson-audit-20261026/build/compiler.lock','a');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
commands=[[str(javac),'-J-Xmx512m','-J-XX:ActiveProcessorCount=2','--release','21','-proc:none','-implicit:none','-sourcepath',str(inputs/'empty-sourcepath'),'-classpath',classpath,'-d',str(classes),str(source)],
          [str(java),'-Xmx512m','-XX:ActiveProcessorCount=2','-Dstdout.encoding=UTF-8','-Dstderr.encoding=UTF-8','-classpath',str(classes)+os.pathsep+classpath,'ProviderProbe',str(derived).lower()]]
(out/'executed-runner.py').write_bytes(Path(__file__).read_bytes());(out/'ProviderProbe.java').write_bytes(source.read_bytes())
receipt={'status':'FAIL','inputs_before':frozen,'commands':[],'derived':derived,'scope':'26 selected real provider/resource cases; no rendered UI or whole-artifact acceptance'}
try:
    for i,argv in enumerate(commands):
        result=subprocess.run(argv,capture_output=True,timeout=90,env=dict(os.environ,JDK_JAVAC_OPTIONS='',JAVA_TOOL_OPTIONS=''))
        (out/f'{i:02d}-stdout.txt').write_bytes(result.stdout);(out/f'{i:02d}-stderr.txt').write_bytes(result.stderr)
        receipt['commands'].append({'argv':argv,'exit_code':result.returncode})
        assert result.returncode==0,result.stderr.decode(errors='replace')
    assert b'PASS_SELECTED_PROVIDER_CASES=26' in result.stdout
    receipt['status']='PASS_26_SELECTED_NATIVE_PROVIDER_CASES'
finally:
    receipt['inputs_after']={p:sha(Path(p)) for p in frozen};receipt['source_unchanged']=frozen==receipt['inputs_after']
    (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(receipt['status'])
