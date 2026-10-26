"""Actual stopped native-template capture and selection/refusal CLI checks."""
import hashlib,json,subprocess
from pathlib import Path
base=Path('/workspace/build/builderbob/syson-rust-parity-batch6-20261026');out=base/'templates-cli-01';out.mkdir(exist_ok=False)
binary=Path('/workspace/build/builderbob/syson-audit-20261026/build/target/debug/syson-native-project-templates')
capture=base/'templates-native-01/native-templates.json';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
frozen={str(p):sha(p) for p in [binary,capture]};expected=json.loads(capture.read_bytes())
(out/'executed-runner.py').write_bytes(Path(__file__).read_bytes())
receipt={'status':'FAIL','inputs_before':frozen,'cases':[],'scope':'Actual captured template/control choices only; no instantiation, paging reconstruction or UI acceptance'}
try:
    for name,args,selection in [('all',[str(capture)],None),('template',[str(capture),'all-templates','sysmlv2-template'],{'id':'sysmlv2-template','kind':'template'}),('blank',[str(capture),'browser-large','blank-project'],{'id':'blank-project','kind':'template'}),('upload',[str(capture),'browser-large','upload-project'],{'id':'upload-project','kind':'upload'}),('browse',[str(capture),'browser-large','browse-all-project-templates'],{'id':'browse-all-project-templates','kind':'browse'})]:
        r=subprocess.run([str(binary),*args],capture_output=True,timeout=10);(out/(name+'-stdout.json')).write_bytes(r.stdout);(out/(name+'-stderr.txt')).write_bytes(r.stderr)
        assert r.returncode==0,(name,r.stderr)
        actual=json.loads(r.stdout);assert actual['capture']==expected and actual['selection']==selection
        if name=='all':
            observations={o['id']:o for o in actual['observations']}
            for id in ['browser-small-first','browser-small-next','invalid-context','invalid-page-type']:assert observations[id]['choices'] is None and observations[id]['outcome']!='SUCCESS'
            assert observations['modal-small-next']['outcome']=='SUCCESS' and observations['modal-small-next']['choices']==[]
        receipt['cases'].append({'name':name,'exit_code':0,'entire_capture_value_equal':True,'selection':selection,'stdout_sha256':hashlib.sha256(r.stdout).hexdigest()})
    invalid=out/'invalid.json';invalid.write_text('{');link=out/'symlink.json';link.symlink_to(capture)
    for name,args in [('usage',[]),('invalid',[str(invalid)]),('symlink',[str(link)]),('unknown',[str(capture),'all-templates','missing']),('failed-context',[str(capture),'browser-small-first','blank-project']),('wrong-context-control',[str(capture),'modal-large','upload-project'])]:
        r=subprocess.run([str(binary),*args],capture_output=True,timeout=10);(out/(name+'-stdout.txt')).write_bytes(r.stdout);(out/(name+'-stderr.txt')).write_bytes(r.stderr)
        assert r.returncode!=0 and not r.stdout,(name,r.stderr)
        receipt['cases'].append({'name':name,'exit_code':r.returncode,'no_partial_stdout':True})
    receipt['status']='PASS_ACTUAL_TEMPLATE_CHOICES_AND_REFUSALS'
finally:
    receipt['inputs_after']={p:sha(Path(p)) for p in frozen};receipt['source_unchanged']=receipt['inputs_after']==frozen
    (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps(receipt['cases']))
