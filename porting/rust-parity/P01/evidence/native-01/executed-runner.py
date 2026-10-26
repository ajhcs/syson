import hashlib,json,os,signal,subprocess,time
from pathlib import Path
root=Path(__file__).parent; plan_path=root/'P01/fixtures.json'; plan=json.loads(plan_path.read_text());out=root/'P01/native-01';out.mkdir(exist_ok=False);scratch=out/'tmp';scratch.mkdir()
sha=lambda b:hashlib.sha256(b).hexdigest()
for key in ['node','script']:assert sha(Path(plan[key]).read_bytes())==plan[key+'_sha256']
(out/'executed-runner.py').write_bytes(Path(__file__).read_bytes());(out/'fixtures.json').write_bytes(plan_path.read_bytes())
env={'PATH':str(Path(plan['node']).parent)+':/usr/bin:/bin','HOME':os.environ['HOME'],'TMPDIR':str(scratch),'LANG':'C.UTF-8','LC_ALL':'C.UTF-8','FORCE_COLOR':'0'}
rows=[]
for f in plan['fixtures']:
 source=Path(f['path']).read_bytes();assert sha(source)==f['sha256'] and source==f['source'].encode();(out/(f['id']+'.sysml')).write_bytes(source)
 argv=[plan['node'],'--max-old-space-size=512',plan['script'],'dump',f['path'],'--stdlib','none'];start=time.perf_counter();child=subprocess.Popen(argv,cwd=out,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
 try:stdout,stderr=child.communicate(timeout=30);code=child.returncode
 except subprocess.TimeoutExpired:
  os.killpg(child.pid,signal.SIGKILL);stdout,stderr=child.communicate(timeout=5);code=124
 assert len(stdout)<=4194304 and len(stderr)<=1048576
 (out/(f['id']+'.stdout.txt')).write_bytes(stdout);(out/(f['id']+'.stderr.txt')).write_bytes(stderr)
 ast=None;text=stdout.decode('utf-8',errors='replace');decoder=json.JSONDecoder()
 for i,ch in enumerate(text):
  if ch!='{':continue
  try:
   obj,end=decoder.raw_decode(text,i)
   if not text[end:].strip():ast=obj;break
  except json.JSONDecodeError:pass
 declarations=[];comments=[]
 def visit(v):
  if isinstance(v,dict):
   if v.get('$type') in ['Package','PartDefinition','PartUsage']:declarations.append({'kind':v['$type'],'raw_declaredName':v.get('declaredName')})
   if v.get('$type')=='Comment':comments.append({k:x for k,x in v.items() if k in ['body','$type']})
   for k,x in v.items():
    if k not in ['$meta','$cstNode']:visit(x)
  elif isinstance(v,list):
   for x in v:visit(x)
 if ast is not None:visit(ast);(out/(f['id']+'.ast.json')).write_text(json.dumps(ast,indent=2)+'\n')
 expected=[{k:v for k,v in d.items() if k in ['kind','raw_declaredName']} for d in f['expected_declarations_preorder']]
 row={'id':f['id'],'argv':argv,'exit_code':code,'elapsed_ms':(time.perf_counter()-start)*1000,'source_sha256':sha(source),'stdout_sha256':sha(stdout),'stderr_sha256':sha(stderr),'ast_emitted':ast is not None,'declarations':declarations,'comments':comments,'selected_declaration_projection_matches_prepared_expectation':declarations==expected,'prepared_native_expectation':f['native_expected'],'rust_expectation':f['rust_expected']}
 rows.append(row);print(json.dumps(row),flush=True)
(out/'receipt.json').write_text(json.dumps({'status':'CAPTURED_NOT_FULL_MODEL_EQUIVALENCE','plan_sha256':sha(plan_path.read_bytes()),'runner_sha256':sha(Path(__file__).read_bytes()),'results':rows},indent=2)+'\n')
