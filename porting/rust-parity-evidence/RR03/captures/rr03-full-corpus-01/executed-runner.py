from pathlib import Path
import subprocess,json,hashlib,os,fcntl,signal
b=Path(__file__).parent;o=b/'rr03-full-corpus-01';o.mkdir(exist_ok=False);prior=Path('/workspace/build/builderbob/syson-rust-parity-batch6-20261026');root=prior/'library-inputs';manifest=root/'manifest.json';native=prior/'library-native-01/native-libraries.json';binary=b/'build/target/release/syson-native-library-catalog';sha=lambda f:hashlib.sha256(f.read_bytes()).hexdigest();assert sha(manifest)=='b0f439f0d314e0dccf60dff70ee3d6bb38583f43ed11afce06da6ad383c9f456';assert sha(native)=='1da0ffd1377d0aad9bbcfb1a277393592329a885d955ea7a2b32dc12b3a7b964';rows=json.loads(manifest.read_text());assert len(rows)==94
for row in rows:
 data=subprocess.check_output(['git','show','6ba2ac64e6b16ee2b05752ad55395eebb87d4910:backend/application/syson-application-configuration/src/main/resources/'+row['classpath_path']],cwd='/workspace/repos/builderbob/syson');assert hashlib.sha256(data).hexdigest()==sha(root/row['file']),row['classpath_path']
inputs=[binary,manifest,native,Path('/usr/bin/time'),*[root/row['file'] for row in rows]];before={str(f):sha(f) for f in inputs};receipt={'status':'FAILED_ATTEMPT','inputs_before':before,'scope':'Fresh final RR03 Rust CLI against retained hash-verified native identity capture; no new native epoch, traversal/reference/EMF/clean-load equivalence claim','all_94_input_files_match_pinned_target_git':True};(o/'executed-runner.py').write_bytes(Path(__file__).read_bytes());lock=open('/workspace/build/builderbob/syson-audit-20261026/build/compiler.lock','a');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
try:
 args=['/usr/bin/time','-v','-o',str(o/'time.txt'),str(binary),str(root),'manifest.json'];receipt['argv']=args
 with (o/'index.json').open('wb') as stdout,(o/'stderr.txt').open('wb') as stderr:
  child=subprocess.Popen(args,stdout=stdout,stderr=stderr,start_new_session=True)
  try:code=child.wait(timeout=180)
  except subprocess.TimeoutExpired:os.killpg(child.pid,signal.SIGKILL);child.wait(timeout=5);code=124
 receipt['exit_code']=code;assert code==0,(o/'stderr.txt').read_text()
 actual=json.loads((o/'index.json').read_bytes());oracle=json.loads(native.read_bytes());left={r['resource_uri']:r for r in actual['resources']};right={r['resource_uri']:r for r in oracle['resources']};assert len(left)==len(actual['resources'])==94 and set(left)==set(right);count=0
 for uri,r in left.items():
  a={n['node_id']:n for n in r['nodes']};z={n['node_id']:n for n in right[uri]['nodes']};assert len(a)==len(r['nodes']) and set(a)==set(z)
  for key,value in a.items():
   expected=z[key];assert value['semantic_id']==expected['semantic_id'] and value['declared_name']==expected['declared_name'];assert value['e_class']=='sysml:'+expected['e_class'] and expected['package_uri']=='http://www.eclipse.org/syson/sysml';count+=1
 assert count==109673;receipt.update(status='PASS_94_RESOURCES_109673_PHYSICAL_TUPLES',resources=94,physical_nodes=count,native_diagnostics_retained=sum(len(r['errors']) for r in oracle['resources']),index_sha256=sha(o/'index.json'),index_bytes=(o/'index.json').stat().st_size,measurement_scope='One complete release process including read/index/49MB output; not p95/SLA/native ratio')
finally:
 receipt['inputs_after']={str(f):sha(f) for f in inputs};receipt['source_unchanged']=before==receipt['inputs_after'];(o/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({k:v for k,v in receipt.items() if k not in ['inputs_before','inputs_after','argv']},indent=2))
