#!/usr/bin/env python3
"""New composition observations using identified existing pure native oracles."""
import copy
import fcntl
import hashlib
import json
import math
import os
from pathlib import Path
import re
import signal
import statistics
import subprocess

base=Path('/workspace/build/builderbob/syson-rust-parity-batch4-20261026')
out=base/'project-observations-01';out.mkdir()
(out/'executed-runner.py').write_bytes(Path(__file__).read_bytes())
(out/'tmp').mkdir()
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
root=Path('/workspace/worktrees/builderbob/syson/rp-c03b-project-preparation')
capture=json.loads((base/'captures/c03b-01/receipt.json').read_text())
assert capture['status']=='PASS' and capture['source_unchanged']
source_freeze=lambda:{p:sha(root/p) for p in capture['source_after']}
before=source_freeze();assert before==capture['source_after']
lock=Path('/workspace/build/builderbob/syson-audit-20261026/build/compiler.lock').open('a');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
program=Path('/workspace/build/builderbob/syson-audit-20261026/build/target/release/examples/observe')
java=Path('/workspace/build/builderbob/ajhcs-sysml-java21-20261004/jdk-21.0.12.1+1-jre/bin/java')
seq=Path('/workspace/build/builderbob/syson-rust-parity-batch2-20261026/sequence-01')
action=Path('/workspace/build/builderbob/syson-rust-parity-batch3-20261026/native/run-01')
sem_review=json.loads(Path('/workspace/build/builderbob/syson-rust-parity-batch3-20261026/C01b/final-scoped-review.json').read_text())
action_review=json.loads((action/'review.json').read_text())
oracle_inputs={str(program):sha(program),str(java):'999c141a6e620c71aa4230ccb62341a89162f37818753ff006d766bf935dcbe1', '/usr/bin/time':sha(Path('/usr/bin/time'))}
for key in ['source_sha256','tools','dependency_jar_sha256']:oracle_inputs.update(sem_review[key])
sem_harness=Path('/workspace/build/builderbob/syson-rust-parity-batch2-20261026/native/src-03/MigrationOracle.java');oracle_inputs[str(sem_harness)]=sem_review['harness_sha256']
for path,digest in json.loads((seq/'receipt.json').read_text())['class_sha256'].items():oracle_inputs[str(seq/path)]=digest
for path,digest in json.loads((action/'receipt.json').read_text())['classes'].items():oracle_inputs[str(action/path)]=digest
oracle_inputs.update(action_review['source_sha256'])
oracle_inputs[str(action.parent/'src/ActionOracle.java')]=action_review['harness_sha256']
for name,digest in action_review['lib_sha256'].items():oracle_inputs[str(action.parent/'lib'/name)]=digest
for p,digest in oracle_inputs.items():assert sha(Path(p))==digest,p
for classes in [seq/'classes',action/'classes']:
    actual={str(p) for p in classes.rglob('*') if p.is_file()}
    expected={p for p in oracle_inputs if p.startswith(str(classes)+'/')}
    assert actual==expected and not any(p.is_symlink() for p in classes.rglob('*'))
for jars in [Path('/workspace/build/builderbob/syson-rust-parity-batch2-20261026/native/lib'),action.parent/'lib']:
    assert {str(p) for p in jars.iterdir()}=={p for p in oracle_inputs if p.startswith(str(jars)+'/')}
packet=json.loads((root/'porting/rust-parity-evidence/C03b/project-fixtures.json').read_text())
projects={case['id']:copy.deepcopy(case['input']) for case in packet['cases']}
for project in projects.values():
    for doc in project['documents']:doc['json']=doc['json'].replace('sysmllibrary:/','local:/')
    for rep in project['representations']:
        value=json.loads(rep['json'])
        def local(v):
            if isinstance(v,dict):
                for k,x in v.items():
                    if k=='targetObjectId' and x=='old':v[k]='local'
                    else:local(x)
            elif isinstance(v,list):
                for x in v:local(x)
        local(value);rep['json']=json.dumps(value,separators=(',',':'),ensure_ascii=False)
projects.pop('foreign-kind')
projects['late-array']=copy.deepcopy(projects['combined'])
rep=projects['late-array']['representations'][0];value=json.loads(rep['json']);value['nodes'].append([{'descriptionId':value['nodes'][0]['descriptionId']}]);rep['json']=json.dumps(value,separators=(',',':'),ensure_ascii=False)
semantic={};controls={}
for name,project in projects.items():
    for doc in project['documents']:semantic.setdefault(hashlib.sha256(doc['json'].encode()).hexdigest(),{'id':hashlib.sha256(doc['json'].encode()).hexdigest(),'input':doc['json']})
    for rep in project['representations']:controls[name]={'id':name,'input':rep['json'],'mode':'traversal'}
for name,data in [('semantic-inputs.json',{'cases':list(semantic.values())}),('control-inputs.json',{'cases':list(controls.values())})]:(out/name).write_text(json.dumps(data,ensure_ascii=False)+'\n')
receipt={'status':'FAIL_SETUP','source_before':before,'oracle_inputs':oracle_inputs,'commands':[],'scope':'New local-reference projects exercise whole-value b/a/c and d composition; B01 is active with no matching library IDs. Native library-ID/history/persistence equivalence is not claimed.'}
env={'PATH':'/usr/bin:/bin','HOME':os.environ['HOME'],'LANG':'C','LC_ALL':'C','TMPDIR':str(out/'tmp')}
def execute(name,argv):
    child=subprocess.Popen(argv,cwd=out,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
    try:stdout,stderr=child.communicate(timeout=60);code=child.returncode
    except subprocess.TimeoutExpired:
        os.killpg(child.pid,signal.SIGKILL);stdout,stderr=child.communicate(timeout=3);code=124
    (out/(name+'-stdout.txt')).write_bytes(stdout);(out/(name+'-stderr.txt')).write_bytes(stderr)
    receipt['commands'].append(dict(name=name,argv=argv,exit_code=code,stdout_sha256=hashlib.sha256(stdout).hexdigest(),stderr_sha256=hashlib.sha256(stderr).hexdigest()))
    assert code==0,(name,code,stderr.decode()[:1000]);return json.loads(stdout)
try:
    flags=[str(java),'-Xmx256m','-XX:ActiveProcessorCount=2','-Djava.io.tmpdir='+str(out/'tmp'),'-cp']
    sem=execute('native-semantic',flags+[str(seq/'classes')+':/workspace/build/builderbob/syson-rust-parity-batch2-20261026/native/lib/*','MigrationOracle','sequence',str(out/'semantic-inputs.json')])
    ctl=execute('native-control',flags+[str(action/'classes')+':'+str(action.parent/'lib')+'/*','ActionOracle',str(out/'control-inputs.json')])
    sem={row['id']:row for row in sem['results']};ctl={row['id']:row for row in ctl['results']}
    comparisons=[]
    for name,project in projects.items():
        path=out/(name+'.json');path.write_text(json.dumps(project,ensure_ascii=False)+'\n')
        result=execute('rust-'+name,[str(program),str(path),'1'])
        refused=name in ctl and ctl[name]['status']=='REFUSED'
        if refused:
            assert result['wire'] is None and result['error'] is not None
        else:
            assert result['error'] is None
            for original,prepared in zip(project['documents'],result['wire']['documents']):
                observed=sem[hashlib.sha256(original['json'].encode()).hexdigest()]
                assert json.loads(prepared['json'])==observed['json']
                assert next(iter(json.loads(original['json'])['ns']))==next(iter(json.loads(prepared['json'])['ns']))
            for prepared in result['wire']['representations']:assert json.loads(prepared['json'])==ctl[name]['json']
        comparisons.append(dict(project=name,outcome='MATCHED_REFUSAL_RUST_ATOMIC_NATIVE_PARTIAL_RETAINED' if refused else 'MATCHED_WHOLE_JSON_VALUES_AND_NAMESPACE_FIRST_KEY'))
    measurements=[]
    for count in [256,1024,4096]:
        records=[]
        for i in range(count):
            cls='Annotation' if i%2==0 else 'Comment';data={'elementId':f'e{i}',('annotatingElement' if i%2==0 else 'annotation'):{'legacy':True}}
            if cls=='Comment':data['body']=f'note{i}'
            records.append({'id':f'n{i}','eClass':'sysml:'+cls,'data':data})
        value={'json':{'version':'1.0','encoding':'utf-8'},'ns':{'sysml':'http://www.eclipse.org/syson/sysml'},'content':records}
        text=json.dumps(value,separators=(',',':'))
        project={'documents':[{'json':text,'versions':{},'references_version':None}],'representations':copy.deepcopy(projects['combined']['representations'])}
        path=out/f'capacity-{count}.json';path.write_text(json.dumps(project)+'\n');rss=out/f'capacity-{count}-rss.txt'
        result=execute(f'capacity-{count}',['/usr/bin/time','-v','-o',str(rss),str(program),str(path),'12'])
        assert result['error'] is None and result['wire'] is not None
        samples=[row['elapsed_us'] for row in result['samples'][2:]];ordered=sorted(samples)
        peak=int(re.search(r'Maximum resident set size \(kbytes\): (\d+)',rss.read_text()).group(1))
        measurements.append(dict(semantic_nodes=count,diagram_nodes=6,document_input_bytes=len(text.encode()),input_sha256=sha(path),raw_prepare_us=samples,warmup_samples_discarded=2,median_us=statistics.median(samples),p95_us=ordered[math.ceil(.95*len(ordered))-1],process_peak_RSS_KiB=peak,resolver_calls=[s['resolver_calls'] for s in result['samples']]))
    receipt.update(status='PASS_SELECTED_COMPOSITION_AND_MODEST_CAPACITY',comparisons=comparisons,measurements=measurements,measurement_boundary='Synthetic flat inputs; prepare-only timing excludes decode/output, RSS covers entire observer process. Ten measured samples after two warmups per process; no before/native ratio or user/UI capacity claim.')
except BaseException as error:receipt.update(status='FAIL',error=repr(error))
finally:
    receipt['source_after']=source_freeze();receipt['source_unchanged']=receipt['source_before']==receipt['source_after'];receipt['oracle_inputs_unchanged']=all(sha(Path(p))==h for p,h in oracle_inputs.items())
    if not receipt['source_unchanged'] or not receipt['oracle_inputs_unchanged']:receipt['status']='INPUT_DRIFT'
    (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({k:receipt.get(k) for k in ['status','source_unchanged','oracle_inputs_unchanged','comparisons','measurements','error']}))
raise SystemExit(0 if receipt['status']=='PASS_SELECTED_COMPOSITION_AND_MODEST_CAPACITY' else 1)
