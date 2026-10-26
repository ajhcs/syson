#!/usr/bin/env python3
"""Fresh owned native HTTP observations; no redirects, mutation or service."""
import datetime
import fcntl
import hashlib
import http.client
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import time

base=Path('/workspace/build/builderbob/syson-rust-parity-batch5-20261026')
old=Path('/workspace/build/builderbob/syson-audit-20261026')
run=base/'http-native-01';run.mkdir(mode=0o700)
(run/'executed-runner.py').write_bytes(Path(__file__).read_bytes())
def sha(p):
    with p.open('rb') as stream:return hashlib.file_digest(stream,'sha256').hexdigest()
def now():return datetime.datetime.now(datetime.timezone.utc).isoformat()
def need(ok,why):
    if not ok:raise RuntimeError(why)
lock=(old/'build/compiler.lock').open('a');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
review=json.loads((old/'native-execution-review.json').read_text())
inputs=dict(review['tool_sha256'])
for path,digest in inputs.items():need(sha(Path(path))==digest,'native input changed')
manifest=Path('/workspace/worktrees/builderbob/syson/rp-c01f-http-contract/porting/rust-parity-evidence/C01f/probes.json')
inputs[str(manifest)]=sha(manifest);packet=json.loads(manifest.read_text());need(len(packet['cases'])==22,'probe scope')
(run/'probes.json').write_bytes(manifest.read_bytes())
for port in [15449,18449,18459]:
    with socket.socket() as s:s.bind(('127.0.0.1',port))
for name in ['data','socket','tmp','prefs','bodies']:(run/name).mkdir(mode=0o700)
app_origin='http://127.0.0.1:18449';management_origin='http://127.0.0.1:18459'
properties='server.address=127.0.0.1\nserver.port=18449\nspring.datasource.url=jdbc:postgresql://127.0.0.1:15449/postgres\nspring.datasource.username=builderbob\nspring.datasource.password=\nliquibase.analytics.enabled=false\nspring.liquibase.analytics-enabled=false\nsirius.components.cors.allowedOriginPatterns='+app_origin+'\nmanagement.server.address=127.0.0.1\nmanagement.server.port=18459\nmanagement.endpoints.access.default=none\nmanagement.endpoint.mappings.access=read-only\nmanagement.endpoints.web.exposure.include=mappings\nmanagement.endpoints.web.discovery.enabled=false\n'
config=run/'audit.properties';config.write_text(properties);inputs[str(config)]=sha(config)
context=dict(application_origin=app_origin,management_origin=management_origin,diagnostic_mappings=True,artifact_sha256=inputs[str(old/'syson-application-2026.9.3.jar')],source_label_revision='b380dada415ce05155150f23cb3b74ae21cc45e4',target_revision='6ba2ac64e6b16ee2b05752ad55395eebb87d4910',configuration_sha256=sha(config),configuration_properties=properties,captured_at=now(),anonymous=True,boundary='Diagnostic mappings enabled on separate loopback context. Fresh synthetic DB; no auth/cookies/redirects/multipart/SSE/websocket/mutations. HTTP body is entity bytes after framing, no content decoding; identity requested. Source/artifact equivalence and global API behavior OPEN.')
receipt=dict(status='FAIL_SETUP',started_utc=now(),context=context,inputs=inputs,commands=[],observations=[],service_registration=False)
pg=old/'postgres-root/usr/lib/postgresql/15/bin';java=Path('/workspace/build/builderbob/ajhcs-sysml-java21-20261004/jdk-21.0.12.1+1-jre/bin/java')
env={'PATH':'/workspace/build/builderbob/runtimes/node-v24.21.0-linux-x64/bin:/usr/bin:/bin','HOME':os.environ['HOME'],'LANG':'C','LC_ALL':'C','TMPDIR':str(run/'tmp'),'LD_LIBRARY_PATH':str(old/'postgres-lib')}
children=[]
def start(name,argv,e):
    out=(run/(name+'-stdout.txt')).open('wb');err=(run/(name+'-stderr.txt')).open('wb');p=subprocess.Popen(argv,cwd=run,env=e,stdout=out,stderr=err,start_new_session=True)
    row=dict(name=name,argv=argv,pid=p.pid);receipt['commands'].append(row);children.append((name,p,out,err,row));return p
def request(port,case,cap=2*1024*1024,retain=True):
    body=case['body_utf8'].encode();headers=[['Host','127.0.0.1:'+str(port)],['Connection','close'],['Accept-Encoding','identity']]
    if case['method']=='POST':headers.append(['Content-Length',str(len(body))])
    headers += [[k,v.replace('{application_origin}',app_origin)] for k,v in case['headers']]
    need(case['method'] in ['GET','HEAD','OPTIONS','POST'] and case['path'].startswith('/') and not case['path'].startswith('//'),'unsafe request')
    need(all('\r' not in k+v and '\n' not in k+v for k,v in headers),'header delimiter')
    connection=http.client.HTTPConnection('127.0.0.1',port,timeout=10);started=time.monotonic()
    try:
        connection.putrequest(case['method'],case['path'],skip_host=True,skip_accept_encoding=True)
        for key,value in headers:connection.putheader(key,value)
        connection.endheaders(body if body else None)
        response=connection.getresponse();raw=response.read(cap+1);need(len(raw)<=cap,'response byte cap')
        data=dict(status=response.status,headers=[list(pair) for pair in response.getheaders()],body_bytes=len(raw),http_version=response.version)
        try:data['body_utf8']=raw.decode('utf-8',errors='strict')
        except UnicodeDecodeError:data['body_hex']=raw.hex()
        observation=dict(id=case['id'],request=dict(method=case['method'],path=case['path'],headers=headers,body_utf8=case['body_utf8']),response=data,elapsed_ms=(time.monotonic()-started)*1000)
        if retain:
            (run/'bodies'/(case['id']+'.bin')).write_bytes(raw);receipt['observations'].append(observation)
        return observation,raw
    finally:connection.close()
try:
    init=start('initdb',[str(pg/'initdb'),'-D',str(run/'data'),'-L',str(old/'postgres-root/usr/share/postgresql/15'),'--locale=C','--encoding=UTF8','--auth=trust','--no-instructions'],env);need(init.wait(timeout=40)==0,'initdb failed')
    db=start('postgres',[str(pg/'postgres'),'-D',str(run/'data'),'-p','15449','-h','127.0.0.1','-k',str(run/'socket'),'-c','unix_socket_permissions=0700','-c','shared_buffers=64MB','-c','max_connections=30','-c','dynamic_library_path='+str(old/'postgres-root/usr/lib/postgresql/15/lib')],env)
    deadline=time.monotonic()+30
    while time.monotonic()<deadline:
        need(db.poll() is None,'PG exited')
        try:
            with socket.create_connection(('127.0.0.1',15449),timeout=.2):break
        except OSError:time.sleep(.2)
    else:raise RuntimeError('PG readiness timeout')
    jenv={k:v for k,v in env.items() if k!='LD_LIBRARY_PATH'};jenv['JAVA_TOOL_OPTIONS']='-Dliquibase.analytics.enabled=false'
    app=start('syson',[str(java),'-Xms256m','-Xmx1536m','-XX:ActiveProcessorCount=2','-Dstdout.encoding=UTF-8','-Dstderr.encoding=UTF-8','-Djava.io.tmpdir='+str(run/'tmp'),'-Djava.util.prefs.userRoot='+str(run/'prefs'),'-jar',str(old/'syson-application-2026.9.3.jar'),'--spring.config.location=classpath:/application.properties','--spring.config.additional-location=file:'+str(config)],jenv)
    deadline=time.monotonic()+180
    while time.monotonic()<deadline:
        need(app.poll() is None,'SysON exited')
        try:
            row,_=request(18449,dict(id='readiness',method='GET',path='/',headers=[['Accept','text/html']],body_utf8=''),retain=False)
            if row['response']['status']==200:break
        except OSError:time.sleep(.5)
    else:raise RuntimeError('SysON readiness timeout')
    mapping,raw=request(18459,dict(id='diagnostic-mappings',method='GET',path='/actuator/mappings',headers=[['Accept','application/json']],body_utf8=''),cap=32*1024*1024)
    (run/'native-mappings.json').write_bytes(raw);need(mapping['response']['status']==200,'mappings unavailable');parsed=json.loads(raw);need(parsed.get('contexts'),'missing actual contexts')
    receipt['probe_errors']=[]
    for case in packet['cases']:
        try:request(18449,case)
        except Exception as error:receipt['probe_errors'].append(dict(id=case['id'],error=repr(error)))
    actual=[r for r in receipt['observations'] if r['id']!='diagnostic-mappings']
    (run/'native-probes.json').write_text(json.dumps(dict(context=context,observations=actual),ensure_ascii=False)+'\n')
    need(len(actual)==22 and not receipt['probe_errors'],'incomplete probe matrix')
    receipt.update(status='PASS_ACTUAL_MAPPINGS_AND_HTTP_PROBES',counts=dict(contexts=len(parsed['contexts']),probes=len(actual)),status_matrix={r['id']:r['response']['status'] for r in actual})
except BaseException as error:receipt.update(status='FAIL',error=repr(error))
finally:
    for name,p,out,err,row in reversed(children):
        if p.poll() is None:
            try:os.killpg(p.pid,signal.SIGINT if name=='postgres' else signal.SIGTERM)
            except ProcessLookupError:pass
            try:p.wait(timeout=15)
            except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);p.wait(timeout=5)
        row['exit_code']=p.returncode;out.close();err.close()
    receipt.update(completed_utc=now(),all_owned_processes_stopped=all(p.poll() is not None for _,p,_,_,_ in children),inputs_after={p:sha(Path(p)) for p in inputs})
    receipt['source_unchanged']=receipt['inputs_after']==inputs
    if not receipt['source_unchanged']:receipt['status']='INPUT_DRIFT'
    (run/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({k:receipt.get(k) for k in ['status','counts','status_matrix','all_owned_processes_stopped','source_unchanged','error']}),flush=True)
raise SystemExit(0 if receipt['status']=='PASS_ACTUAL_MAPPINGS_AND_HTTP_PROBES' else 1)
