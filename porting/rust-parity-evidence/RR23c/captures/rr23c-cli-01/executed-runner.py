from pathlib import Path
import json,subprocess,hashlib,fcntl,sys
b=Path(__file__).parent;uid=sys.argv[1];attempt=sys.argv[2] if len(sys.argv)>2 else '01';a=json.loads((b/'batch-admission.json').read_text());u=next(u for u in a['units'] if u['id']==uid);w=Path(u['worktree']);o=b/(uid.lower()+'-cli-'+attempt);o.mkdir();scratch=o/'scratch';scratch.mkdir();lock=open(a['compiler_lock'],'a');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB);sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();tool={'RR23c':'native-http-contract','RR23d':'native-project-templates'}[uid];exe=b/'build/target/debug'/('syson-'+tool);r={'unit':uid,'status':'FAILED_ATTEMPT','commands':[],'inputs':{},'native_epoch':'NOT_RUN;PRESERVED_CAPTURE_ONLY'};tokens=lambda raw:json.loads(raw,parse_int=str,parse_float=str)
def run(name,args,expected=0):
 for arg in args:
  p=Path(arg)
  if p.is_file():r['inputs'][str(p)]=sha(p)
 p=subprocess.run([str(exe),*map(str,args)],cwd=w,capture_output=True,timeout=30);(o/(name+'-stdout.txt')).write_bytes(p.stdout);(o/(name+'-stderr.txt')).write_bytes(p.stderr);r['commands'].append(dict(name=name,argv=[str(exe),*map(str,args)],exit_code=p.returncode));assert p.returncode==expected,(name,p.stderr[-400:]);assert expected==0 or not p.stdout;return p.stdout
try:
 r['executable_sha256']=sha(exe)
 if uid=='RR23c':
  folder=w/'rust/tools/native-http-contract';paths={mode:folder/'tests/fixtures'/('native-'+mode+'.json') for mode in ['mappings','probes']}
  for mode,path in paths.items():
   result=run('native-'+mode,[mode,path]);plain=json.loads(result);lane='mappings' if mode=='mappings' else 'capture';assert tokens(result)[lane]==tokens(path.read_bytes());r['native_'+mode+'_summary']=plain['summary']
  assert r['native_mappings_summary']=={'contexts':1,'handlers':54,'mvc_handlers':33};assert r['native_probes_summary']['probes']==22
  manifest=w/'porting/rust-parity-evidence/C01f/probes.json';r['inputs'][str(manifest)]=sha(manifest);assert run('manifest',['--probes'])==manifest.read_bytes()
  for mode,path in paths.items():
   member='"vendor":'+json.dumps({'$serde_json::private::Number':'123','$serde_json::private::RawValue':'{"trap":true}'},separators=(',',':'))[:-1]+',"n":-0,"huge":1234567890123456789012345678901234567890},'
   f=scratch/(mode+'-literals.json');f.write_text('{'+member+path.read_text()[1:]);result=run(mode+'-literals',[mode,f]);lane='mappings' if mode=='mappings' else 'capture';value=tokens(result)[lane]['vendor'];assert value=={'$serde_json::private::Number':'123','$serde_json::private::RawValue':'{"trap":true}','n':'-0','huge':'1234567890123456789012345678901234567890'};assert isinstance(json.loads(result)[lane]['vendor']['n'],int)
  raw=json.loads(paths['probes'].read_bytes());zero=next(x for x in raw['observations'] if x['response']['body_bytes']==0);zero['response']['body_bytes']='RR23C_ZERO';f=scratch/'zero-body.json';f.write_text(json.dumps(raw).replace('"RR23C_ZERO"','-0'));result=run('zero-body',['probes',f]);value=next(x for x in tokens(result)['capture']['observations'] if x['id']==zero['id']);assert value['response']['body_bytes']=='-0'
  raw=json.loads(paths['mappings'].read_bytes());entries=next(iter(next(iter(raw['contexts'].values()))['mappings']['dispatcherServlets'].values()));entries[0]['handler']={'$serde_json::private::RawValue':'"fabricated handler"'};f=scratch/'handler-marker.json';f.write_text(json.dumps(raw));run('typed-handler-marker',['mappings',f],1)
  raw=json.loads(paths['probes'].read_bytes());raw['context']['anonymous']={'$serde_json::private::RawValue':'true'};f=scratch/'context-marker.json';f.write_text(json.dumps(raw));run('typed-context-marker',['probes',f],1)
  # Status200 HTML shells remain frontend observations, not API successes.
  native=json.loads(paths['probes'].read_bytes());r['observed_status_200_frontend_shells']=sum(x['response']['status']==200 and '<!doctype html' in x['response'].get('body_utf8','').lower() for x in native['observations']);assert r['observed_status_200_frontend_shells']==9
 else:
  native=w/'rust/tools/native-project-templates/tests/fixtures/native-templates.json';queries=w/'porting/rust-parity-evidence/C01h/queries.json';r['inputs'][str(queries)]=sha(queries)
  selections=[('all',[],None),('template',['all-templates','sysmlv2-template'],{'id':'sysmlv2-template','kind':'template'}),('blank',['browser-large','blank-project'],{'id':'blank-project','kind':'template'}),('upload',['browser-large','upload-project'],{'id':'upload-project','kind':'upload'}),('browse',['browser-large','browse-all-project-templates'],{'id':'browse-all-project-templates','kind':'browse'})]
  for name,extra,selection in selections:
   result=run(name,[native,*extra]);actual=json.loads(result);assert tokens(result)['capture']==tokens(native.read_bytes()) and actual['selection']==selection
   if name=='all':
    views={x['id']:x for x in actual['observations']};assert len(views)==10
    for item in ['browser-small-first','browser-small-next','invalid-context','invalid-page-type']:assert views[item]['choices'] is None and views[item]['outcome']=='GRAPHQL_ERROR'
    assert views['modal-small-next']['outcome']=='SUCCESS' and views['modal-small-next']['choices']==[];r['native_observations']=len(views);r['native_graphql_errors']=4
  invalid=scratch/'invalid.json';invalid.write_text('{');link=scratch/'symlink.json';link.symlink_to(native)
  for name,args in [('usage',[]),('invalid',[invalid]),('symlink',[link]),('unknown',[native,'all-templates','missing']),('failed-context',[native,'browser-small-first','blank-project']),('wrong-control',[native,'modal-large','upload-project'])]:run(name,args,1)
  raw=json.loads(native.read_bytes());raw['context']['vendor']={'$serde_json::private::Number':'opaque','$serde_json::private::RawValue':'{"trap":true}'};raw['context']['numeric']='RR23D_ZERO'
  row=next(x for x in raw['observations'] if x['id']=='modal-large');row['request']['variables']['page']='RR23D_REQUEST';body=json.loads(row['body_utf8']);body['data']['viewer']['projectTemplates']['pageInfo']['count']='RR23D_COUNT';row['body_utf8']=json.dumps(body).replace('"RR23D_COUNT"','-0');saved_body=row['body_utf8'];f=scratch/'literal-and-typed-zero.json';f.write_text(json.dumps(raw).replace('"RR23D_ZERO"','-0').replace('"RR23D_REQUEST"','-0'));result=run('literal-and-typed-zero',[f,'all-templates','sysmlv2-template']);v=tokens(result)['capture'];assert v['context']['vendor']=={'$serde_json::private::Number':'opaque','$serde_json::private::RawValue':'{"trap":true}'} and v['context']['numeric']=='-0';row=next(x for x in v['observations'] if x['id']=='modal-large');assert row['body_utf8']==saved_body and row['request']['variables']['page']=='-0'
  raw=json.loads(native.read_bytes());row=next(x for x in raw['observations'] if x['id']=='all-templates');body=json.loads(row['body_utf8']);body['data']['viewer']['allProjectTemplates'][0]['id']={'$serde_json::private::RawValue':'"sysmlv2-template"'};row['body_utf8']=json.dumps(body);f=scratch/'nested-id-marker.json';f.write_text(json.dumps(raw));run('nested-id-marker',[f],1)
 assert all(sha(Path(f))==h for f,h in r['inputs'].items());r['source_inputs_unchanged']=True;r['status']='PASS_ACTUAL_READER_CLI;_NO_FRESH_NATIVE_EPOCH'
finally:
 (o/'executed-runner.py').write_bytes(Path(__file__).read_bytes());(o/'receipt.json').write_text(json.dumps(r,indent=2)+'\n')
print(json.dumps({k:v for k,v in r.items() if k not in ['inputs','commands']}))
