from pathlib import Path
import subprocess,os,json,time,hashlib,fcntl,signal,statistics
b=Path(__file__).parent;w=Path('/workspace/worktrees/builderbob/syson/batch14-history-summary-joint');o=b/'history-profiles-01';o.mkdir();scratch=o/'scratch';scratch.mkdir();(o/'executed-runner.py').write_bytes(Path(__file__).read_bytes());lock=open('/workspace/build/builderbob/syson-audit-20261026/build/compiler.lock','r');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
for name in ['joint-browser-01','joint-callers-01','joint-callers-remainder-01']:
 r=json.loads((b/name/'receipt.json').read_text());assert not r['final_descendant_cleanup']['remaining']
env=dict(os.environ,CARGO_BUILD_JOBS='1',CARGO_INCREMENTAL='0',RUST_TEST_THREADS='1',CARGO_TARGET_DIR=str(b/'target'),TMPDIR=str(scratch),PYTHONDONTWRITEBYTECODE='1');sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
paths=[*sorted((w/'rust/crates/syson-model').rglob('*.rs')),*[w/'rust/crates/syson-model'/n for n in ['Cargo.toml','Cargo.lock']],*sorted((b/'history-profile').rglob('*'))];paths=[p for p in paths if p.is_file()]
snap=lambda:{str(p):sha(p) for p in paths};r={'status':'RUNNING','source_before':snap(),'commands':[],'samples':[],'scope':'Unchanged host-only requested/live allocation probe, freshprocess per profile; conservative historycharge, measuredrequestedstorageandLinuxRSS areseparate. Whole edittrajectory timings only, no native ratio/p95/per-event/UI claim.'}
def run(argv,timeout=300):
 t=time.monotonic();p=subprocess.Popen(argv,cwd=w,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
 try:out,err=p.communicate(timeout=timeout);timed=False
 except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);out,err=p.communicate();timed=True
 i=len(r['commands']);(o/f'{i:02}-stdout.txt').write_bytes(out);(o/f'{i:02}-stderr.txt').write_bytes(err);r['commands'].append({'argv':argv,'exit_code':p.returncode,'timed_out':timed,'wall_seconds':time.monotonic()-t});(o/'receipt.json').write_text(json.dumps(r,indent=2)+'\n');assert p.returncode==0 and not timed,(argv,err.decode()[-1000:]);return out
try:
 run(['cargo','build','--offline','--locked','--release','--jobs','1','--manifest-path',str(b/'history-profile/Cargo.toml')])
 binary=b/'target/release/rr20-history-probe';r['binary_sha256']=sha(binary)
 for nodes in [1000,4096]:
  for commits in [64,256]:
   cell=[]
   for index in range(9):
    sample=json.loads(run([str(binary),str(nodes),str(commits)],180));assert sample['cancel_live_delta']==0 and sample['undo_steps']==sample['redo_steps'];r['samples'].append({'warmup':index<2,'index':index,**sample});cell.append(sample)
   measured=cell[2:];summary={'nodes':nodes,'commits':commits,'warmups':2,'measured':7,'edit_trajectory_us':{'min':min(x['edit_total_us'] for x in measured),'median':statistics.median(x['edit_total_us'] for x in measured),'max':max(x['edit_total_us'] for x in measured)},'retained_live_bytes_range':[min(x['retained_live'] for x in measured),max(x['retained_live'] for x in measured)],'rss_kib_range':[min(x['rss_after_edits']['vm_rss_kib'] for x in measured),max(x['rss_after_edits']['vm_rss_kib'] for x in measured)]};r.setdefault('cells',[]).append(summary);print(json.dumps(summary),flush=True)
 r['status']='PASS_BOUNDED_HOST_HISTORY_COUNTER_PROFILES;DENSE_UI_OUTLIER_AND_CAPACITY_OPEN'
finally:
 r['source_after']=snap();r['source_unchanged']=r['source_before']==r['source_after'];assert r['source_unchanged'];r['artifacts']={p.name:sha(p) for p in o.iterdir() if p.is_file() and p.name!='receipt.json'};(o/'receipt.json').write_text(json.dumps(r,indent=2)+'\n')
