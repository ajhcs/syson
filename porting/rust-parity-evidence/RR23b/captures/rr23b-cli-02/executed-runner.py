from pathlib import Path
import json,subprocess,hashlib,fcntl,sys
b=Path(__file__).parent;uid=sys.argv[1];a=json.loads((b/'batch-admission.json').read_text());u=next(u for u in a['units'] if u['id']==uid);w=Path(u['worktree']);o=b/(uid.lower()+'-cli-'+(sys.argv[2] if len(sys.argv)>2 else '01'));o.mkdir();scratch=o/'scratch';scratch.mkdir();lock=open(a['compiler_lock'],'a');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB);sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();r={'unit':uid,'status':'FAILED_ATTEMPT','commands':[],'inputs':{}};exe=b/'build/target/debug'/('syson-native-graphql-contract' if uid=='RR23a' else 'syson-native-db-contract')
def run(name,args,expected=0):
 for arg in args:
  p=Path(arg)
  if p.is_file():r['inputs'][str(p)]=sha(p)
 p=subprocess.run([str(exe),*map(str,args)],cwd=w,capture_output=True,timeout=30);(o/(name+'-stdout.txt')).write_bytes(p.stdout);(o/(name+'-stderr.txt')).write_bytes(p.stderr);r['commands'].append(dict(name=name,argv=[str(exe),*map(str,args)],exit_code=p.returncode));assert p.returncode==expected,(name,p.stderr[-300:]);assert expected==0 or not p.stdout;return p.stdout
try:
 r['executable_sha256']=sha(exe)
 if uid=='RR23a':
  folder=w/'rust/tools/native-graphql-contract';native=folder/'tests/fixtures/native-official-label-introspection.json';bad=folder/'tests/fixtures/native-badfaith-introspection.json';v=json.loads(native.read_text());out=json.loads(run('native',[native]));assert out['schema']==v['data']['__schema'];r['native_counts']=out['counts']
  for arg,rel in [('--query','porting/rust-parity-evidence/C01c/introspection.graphql'),('--capabilities','porting/rust-parity-evidence/C01c/capabilities-single.graphql')]:
   f=w/rel;r['inputs'][str(f)]=sha(f);assert run(arg[2:],[arg])==f.read_bytes()
  run('badfaith',[bad],1)
  # Independent ordinary-object and exact numeric-token caller, injected as raw source text.
  raw=native.read_text();needle='"__schema":{';member='"vendor":{"$serde_json::private::Number":"123","$serde_json::private::RawValue":"{\\"nested\\":true}","n":-0,"huge":1234567890123456789012345678901234567890},'
  # json.dumps handles only the ordinary string; source number tokens are inserted literally.
  members='"vendor":'+json.dumps({'$serde_json::private::Number':'123','$serde_json::private::RawValue':'{"nested":true}'},separators=(',',':'))[:-1]+',"n":-0,"huge":1234567890123456789012345678901234567890},'
  f=scratch/'markers.json';f.write_text(raw.replace(needle,needle+members,1));wire=run('markers',[f]);obj=json.loads(wire,parse_int=str,parse_float=str)['schema']['vendor'];assert obj=={'$serde_json::private::Number':'123','$serde_json::private::RawValue':'{"nested":true}','n':'-0','huge':'1234567890123456789012345678901234567890'}
 else:
  native=w/'rust/tools/native-db-contract/tests/fixtures/native-01-catalog.json';reference=w/'porting/rust-parity-evidence/C01d/c01d-cli-02/native-stdout.txt';r['inputs'][str(reference)]=sha(reference);out=json.loads(run('native',[native]));assert out==json.loads(reference.read_text());r['native_obligations']=len(out['obligations']);r['native_migrations']=len(out['migration_order']);assert (r['native_obligations'],r['native_migrations'])==(164,20)
  v=json.loads(native.read_text());v['tables'][0]['oid']={'$serde_json::private::Number':str(v['tables'][0]['oid'])};f=scratch/'marker-oid.json';f.write_text(json.dumps(v));run('marker-oid',[f],1)
  v=json.loads(native.read_text());v['capture']={'$serde_json::private::RawValue':json.dumps(v['capture'])};f=scratch/'marker-capture.json';f.write_text(json.dumps(v));run('marker-capture',[f],1)
  v=json.loads(native.read_text());v['changelog'][0]['orderexecuted']='RR23B_TOKEN';f=scratch/'negative-zero.json';f.write_text(json.dumps(v).replace('"RR23B_TOKEN"','-0'));run('negative-zero',[f],1)
 assert all(sha(Path(f))==h for f,h in r['inputs'].items());r['source_inputs_unchanged']=True;r['status']='PASS_ACTUAL_READER_CLI;_NO_FRESH_NATIVE_EPOCH'
finally:
 (o/'executed-runner.py').write_bytes(Path(__file__).read_bytes());(o/'receipt.json').write_text(json.dumps(r,indent=2)+'\n')
print(json.dumps({k:v for k,v in r.items() if k not in ['inputs','commands']}))
