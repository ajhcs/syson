"""Root-only bounded CLI checks; no native app or network requests."""
import hashlib
import json
from pathlib import Path
import subprocess

root = Path('/workspace/build/builderbob/syson-rust-parity-batch5-20261026')
out = root / 'http-cli-01'
out.mkdir(exist_ok=False)
wt = Path('/workspace/worktrees/builderbob/syson/rp-c01f-http-contract')
binary = Path('/workspace/build/builderbob/syson-audit-20261026/build/target/debug/syson-native-http-contract')
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
inputs = [binary, *sorted((wt/'rust/tools/native-http-contract').rglob('*.rs')),
          wt/'porting/rust-parity-evidence/C01f/probes.json']
identity = {str(p): sha(p) for p in inputs}
receipt = {'status':'FAIL', 'inputs_before':identity, 'cases':[]}
(out/'executed-runner.py').write_bytes(Path(__file__).read_bytes())
try:
    for mode in ['mappings','probes']:
        source = root/'http-native-01'/('native-'+mode+'.json')
        result = subprocess.run([str(binary),mode,str(source)],capture_output=True,timeout=15)
        (out/(mode+'-stdout.json')).write_bytes(result.stdout)
        (out/(mode+'-stderr.txt')).write_bytes(result.stderr)
        assert result.returncode == 0, result.stderr
        actual = json.loads(result.stdout)
        expected = json.loads(source.read_bytes())
        assert actual['C01']=='OPEN'
        assert actual['mappings' if mode=='mappings' else 'capture']==expected
        receipt['cases'].append({'mode':mode,'exit_code':0,'exact_value_equal':True,
                                 'summary':actual['summary'], 'stdout_sha256':hashlib.sha256(result.stdout).hexdigest(),
                                 'input_sha256':sha(source),'input_bytes':source.stat().st_size,
                                 'output_bytes':len(result.stdout)})
    emitted = subprocess.run([str(binary),'--probes'],capture_output=True,timeout=5)
    (out/'manifest-stdout.json').write_bytes(emitted.stdout)
    assert emitted.returncode==0 and emitted.stdout==inputs[-1].read_bytes()
    receipt['cases'].append({'mode':'--probes','exit_code':0,'exact_bytes_equal':True})
    invalid=out/'invalid.json';invalid.write_text('{')
    link=out/'symlink.json';link.symlink_to(root/'http-native-01/native-mappings.json')
    for name,args in [('usage',[]),('invalid',['mappings',str(invalid)]),('symlink',['mappings',str(link)]),('directory',['mappings',str(out)])]:
        result=subprocess.run([str(binary),*args],capture_output=True,timeout=5)
        (out/(name+'-stdout.txt')).write_bytes(result.stdout)
        (out/(name+'-stderr.txt')).write_bytes(result.stderr)
        assert result.returncode!=0 and not result.stdout
        receipt['cases'].append({'mode':name,'exit_code':result.returncode,'no_partial_stdout':True})
    receipt['status']='PASS_ACTUAL_CLI_VALUE_PRESERVATION_AND_REFUSALS'
finally:
    receipt['inputs_after']={str(p):sha(p) for p in inputs}
    receipt['source_unchanged']=receipt['inputs_before']==receipt['inputs_after']
    (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps(receipt['cases']))
