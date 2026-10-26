#!/usr/bin/env python3
"""Root-owned fresh isolated native GraphQL/catalog observation; never a service."""
import datetime
import fcntl
import hashlib
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import sys
import time
import urllib.request
import urllib.error

base=Path('/workspace/build/builderbob/syson-rust-parity-batch4-20261026')
old=Path('/workspace/build/builderbob/syson-audit-20261026')
run_name=sys.argv[1] if len(sys.argv)>1 else 'native-01'
mode=sys.argv[2] if len(sys.argv)>2 else 'both'
assert run_name in ['native-01','native-02'] and mode in ['both','graphql-only']
run=base/run_name
run.mkdir(mode=0o700)
(run/'executed-runner.py').write_bytes(Path(__file__).read_bytes())
def sha(p):
    with p.open('rb') as stream:return hashlib.file_digest(stream,'sha256').hexdigest()
def now():return datetime.datetime.now(datetime.timezone.utc).isoformat()
def need(value,reason):
    if not value:raise RuntimeError(reason)
lock=(old/'build/compiler.lock').open('a')
fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
review=json.loads((old/'native-execution-review.json').read_text())
for p,h in review['tool_sha256'].items():need(sha(Path(p))==h,'native input changed: '+p)
pg=old/'postgres-root/usr/lib/postgresql/15/bin'
java=Path('/workspace/build/builderbob/ajhcs-sysml-java21-20261004/jdk-21.0.12.1+1-jre/bin/java')
node=Path('/workspace/build/builderbob/runtimes/node-v24.21.0-linux-x64/bin')
jar=old/'syson-application-2026.9.3.jar'
inputs={str(Path(p)):h for p,h in review['tool_sha256'].items()}
inputs[str(pg/'psql')]=sha(pg/'psql')
source=Path('/workspace/worktrees/builderbob/syson')
for filename,origin in [('capabilities.graphql',source/'rp-c01c-graphql-contract/porting/rust-parity-evidence/C01c/capabilities.graphql'),('introspection.graphql',source/'rp-c01c-graphql-contract/porting/rust-parity-evidence/C01c/introspection.graphql'),('catalog-query.sql',source/'rp-c01d-persistence-contract/porting/rust-parity-evidence/C01d/catalog-query.sql')]:
    data=origin.read_bytes();need(len(data)<32768,'query byte bound')
    (run/filename).write_bytes(data);inputs[str(origin)]=sha(origin)
if mode=='graphql-only':
    origin=source/'rp-c01c-graphql-contract/porting/rust-parity-evidence/C01c/capabilities-single.graphql'
    (run/'capabilities-single.graphql').write_bytes(origin.read_bytes());inputs[str(origin)]=sha(origin)
for port in [15448,18448]:
    with socket.socket() as sock:sock.bind(('127.0.0.1',port))
for name in ['data','socket','tmp','prefs']:(run/name).mkdir(mode=0o700)
config=run/'audit.properties'
config.write_text('server.address=127.0.0.1\nserver.port=18448\nspring.datasource.url=jdbc:postgresql://127.0.0.1:15448/postgres\nspring.datasource.username=builderbob\nspring.datasource.password=\nliquibase.analytics.enabled=false\nspring.liquibase.analytics-enabled=false\nmanagement.server.port=-1\nsirius.components.cors.allowedOriginPatterns=http://127.0.0.1:18448\nlogging.level.org.eclipse.sirius.web.infrastructure.configuration.graphql.GraphQLConfiguration=DEBUG\n')
inputs[str(config)]=sha(config)
env={'PATH':str(node)+':/usr/bin:/bin','HOME':os.environ['HOME'],'LANG':'C','LC_ALL':'C','TMPDIR':str(run/'tmp'),'LD_LIBRARY_PATH':str(old/'postgres-lib')}
receipt=dict(status='FAIL_SETUP',started_utc=now(),inputs=inputs,commands=[],http=[],binds=['127.0.0.1:15448','127.0.0.1:18448'],
             artifact_source_label='b380dada415ce05155150f23cb3b74ae21cc45e4',target_revision='6ba2ac64e6b16ee2b05752ad55395eebb87d4910',
             source_binding='OPEN:125 source-label paths differ; not independently rebuilt',
             context='Fresh empty owned PostgreSQL database; anonymous same-origin HTTP introspection, no credentials/session.',
             service_registration=False,review='Coordinator reviewed fixed argv/config/SQL/queries before first execution; fresh synthetic DB only.')
children=[]
def start(name,argv,childenv):
    out=(run/(name+'-stdout.txt')).open('wb');err=(run/(name+'-stderr.txt')).open('wb')
    p=subprocess.Popen(argv,cwd=run,env=childenv,stdout=out,stderr=err,start_new_session=True)
    entry=dict(name=name,argv=argv,pid=p.pid,started_utc=now())
    receipt['commands'].append(entry);children.append((name,p,out,err,entry));return p
def completed(name,argv,childenv,timeout=30):
    p=start(name,argv,childenv);code=p.wait(timeout=timeout);receipt['commands'][-1]['exit_code']=code;need(code==0,name+' failed')
def post(name,operation,query,variables=None):
    payload=json.dumps(dict(operationName=operation,query=query,variables=variables or {})).encode()
    (run/(name+'-request.json')).write_bytes(payload)
    request=urllib.request.Request('http://127.0.0.1:18448/api/graphql',payload,{'Content-Type':'application/json','Accept':'application/json'},method='POST')
    start_time=time.monotonic()
    try:response=urllib.request.urlopen(request,timeout=30)
    except urllib.error.HTTPError as error:response=error
    with response:
        body=response.read(32*1024*1024+1)
        (run/(name+'-response.json')).write_bytes(body)
        entry=dict(name=name,url=request.full_url,method='POST',request_headers=dict(request.header_items()),response_headers=dict(response.headers),status=response.status,elapsed_ms=(time.monotonic()-start_time)*1000,request_sha256=hashlib.sha256(payload).hexdigest(),response_sha256=hashlib.sha256(body).hexdigest(),response_bytes=len(body))
        receipt['http'].append(entry)
        need(response.status==200 and len(body)<=32*1024*1024,'HTTP status/byte bound')
    parsed=json.loads(body);need(not parsed.get('errors'),'GraphQL returned errors: '+name);return parsed
try:
    completed('postgres-version',[str(pg/'postgres'),'--version'],env)
    completed('initdb',[str(pg/'initdb'),'-D',str(run/'data'),'-L',str(old/'postgres-root/usr/share/postgresql/15'),'--locale=C','--encoding=UTF8','--auth=trust','--no-instructions'],env,40)
    db=start('postgres',[str(pg/'postgres'),'-D',str(run/'data'),'-p','15448','-h','127.0.0.1','-k',str(run/'socket'),'-c','unix_socket_permissions=0700','-c','shared_buffers=64MB','-c','max_connections=30','-c','dynamic_library_path='+str(old/'postgres-root/usr/lib/postgresql/15/lib')],env)
    deadline=time.monotonic()+30
    while time.monotonic()<deadline:
        need(db.poll() is None,'owned PostgreSQL exited')
        try:
            with socket.create_connection(('127.0.0.1',15448),timeout=.2):break
        except OSError:time.sleep(.2)
    else:raise RuntimeError('PostgreSQL readiness timeout')
    jenv={k:v for k,v in env.items() if k!='LD_LIBRARY_PATH'};jenv['JAVA_TOOL_OPTIONS']='-Dliquibase.analytics.enabled=false'
    app=start('syson',[str(java),'-Xms256m','-Xmx1536m','-XX:ActiveProcessorCount=2','-Djava.io.tmpdir='+str(run/'tmp'),'-Djava.util.prefs.userRoot='+str(run/'prefs'),'-jar',str(jar),'--spring.config.location=classpath:/application.properties','--spring.config.additional-location=file:'+str(config)],jenv)
    deadline=time.monotonic()+180
    while time.monotonic()<deadline:
        need(app.poll() is None,'owned SysON exited')
        try:
            with urllib.request.urlopen('http://127.0.0.1:18448/',timeout=2) as response:
                if response.status==200:break
        except OSError:time.sleep(.5)
    else:raise RuntimeError('SysON readiness timeout')
    print(json.dumps({'status':'NATIVE_READY','run':str(run)}),flush=True)
    receipt['scope_failures']={}
    full=None
    try:
        if mode=='graphql-only':
            for name in ['__Type','__Field','__InputValue','__Directive','__Schema']:
                caps=post('capabilities-'+name[2:],'C01cCapabilitiesSingle',(run/'capabilities-single.graphql').read_text(),{'typeName':name})
                need(caps.get('data',{}).get('__type') is not None,'missing meta-type capability '+name)
        else:
            caps=post('capabilities','C01cCapabilities',(run/'capabilities.graphql').read_text())
            for key in ['type','field','inputValue','directive','schema']:need(caps.get('data',{}).get(key) is not None,'missing meta-type capability '+key)
        full=post('introspection','C01cEffectiveSchema',(run/'introspection.graphql').read_text())
        need(full.get('data',{}).get('__schema') is not None,'missing effective schema')
    except Exception as error:receipt['scope_failures']['GraphQL']=repr(error)
    argv=[str(pg/'psql'),'-X','-q','-A','-t','-v','ON_ERROR_STOP=1','-h','127.0.0.1','-p','15448','-U','builderbob','-d','postgres']
    for key,value in {'artifact_sha256':sha(jar),'target_revision':receipt['target_revision'],'source_label_revision':receipt['artifact_source_label'],'query_sha256':sha(run/'catalog-query.sql')}.items():argv+=['-v',key+'='+value]
    argv+=['-f',str(run/'catalog-query.sql')]
    catalog=None
    if mode!='graphql-only':
        try:
            completed('catalog',argv,env,30)
            raw=(run/'catalog-stdout.txt').read_bytes();need(len(raw)<=8*1024*1024,'catalog byte bound')
            catalog=json.loads(raw);need(catalog.get('tables') and catalog.get('changelog'),'native catalog/history empty')
        except Exception as error:receipt['scope_failures']['catalog']=repr(error)
    receipt.update(status='PARTIAL_NATIVE_CAPTURE' if receipt['scope_failures'] else ('PASS_NATIVE_GRAPHQL_CAPTURE' if mode=='graphql-only' else 'PASS_NATIVE_SCHEMA_AND_CATALOG_CAPTURE'),native_counts={'types':len(full['data']['__schema']['types']) if full else None,'tables':len(catalog['tables']) if catalog else None,'columns':len(catalog['columns']) if catalog else None,'constraints':len(catalog['constraints']) if catalog else None,'indexes':len(catalog['indexes']) if catalog else None,'applied_changelog':len(catalog['changelog']) if catalog else None},database_catalog='NOT_REPLAYED_ALREADY_CAPTURED_NATIVE01' if mode=='graphql-only' else 'RUN_THIS_EPOCH')
except BaseException as error:
    receipt.update(status='FAIL',error=repr(error))
finally:
    for name,p,out,err,entry in reversed(children):
        if p.poll() is None:
            try:os.killpg(p.pid,signal.SIGINT if name=='postgres' else signal.SIGTERM)
            except ProcessLookupError:pass
            try:p.wait(timeout=15)
            except subprocess.TimeoutExpired:
                os.killpg(p.pid,signal.SIGKILL);p.wait(timeout=5)
        entry.update(exit_code=p.returncode,completed_utc=now());out.close();err.close()
    receipt.update(all_owned_processes_stopped=all(p.poll() is not None for _,p,_,_,_ in children),completed_utc=now(),inputs_after={p:sha(Path(p)) for p in inputs})
    receipt['source_unchanged']=receipt['inputs_after']==inputs
    if not receipt['source_unchanged']:receipt['status']='INPUT_DRIFT'
    (run/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({k:receipt[k] for k in ['status','all_owned_processes_stopped','source_unchanged']}),flush=True)
raise SystemExit(0 if receipt['status'].startswith('PASS_NATIVE_') else 1)
