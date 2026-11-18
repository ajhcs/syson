from pathlib import Path
import hashlib, importlib.util, io, json, os, sys, tempfile, unittest
from unittest.mock import patch

CORE = Path(__file__).resolve().parent
EVIDENCE = CORE.parents[2] / 'rust-parity-evidence/C01'
EPOCH = 'jacoco-binaries-01'
OUTPUT = os.environ.get('SYSON_BOOTSTRAP_TEST_ROOT') or os.environ.get('RUNNER_TEMP')
if not OUTPUT:
    raise RuntimeError('Set SYSON_BOOTSTRAP_TEST_ROOT or RUNNER_TEMP to a private output directory')
output = Path(OUTPUT)
if not output.is_absolute() or output.resolve() != output or not output.is_dir():
    raise RuntimeError('A canonical existing fixture output directory is required')
if os.environ.get('SYSON_BOOTSTRAP_TEST_ROOT') and not output.is_relative_to('/workspace/build/builderbob'):
    raise RuntimeError('Builder Bob fixture output must be on the Samsung workspace')
SCRATCH = Path(tempfile.mkdtemp(prefix='jacoco-binary-test-', dir=output))
DIGEST = lambda b: hashlib.sha256(b).hexdigest()
PREVIOUS = {
    'models-01': (175, '682c4b8569beeb95defb30b315496b5f474d133d99170b4eb79827466c117f49'),
    'models-02': (90, '7434575a1030365ba75b4f0650d36183a1e930efd714c367cf4e810db4445ef4'),
    'models-03': (25, '7eafb6e5436dd04692e43a1f3d87a204dc206ed3fe55a80f9cae2d128f1be428'),
    'models-04': (5, 'e0d4b420cdcdcf23023731af35511fb7fb78402e9d492a87840800c661814289'),
}

def audit(event,args):
    if event.startswith(('socket.','subprocess.')) or event in ('os.exec','os.posix_spawn','os.system','os.fork'):
        raise RuntimeError('unexpected transport/process')
    if ((event=='open' and isinstance(args[0],(str,bytes,os.PathLike))
         and args[2] & (os.O_WRONLY|os.O_RDWR|os.O_CREAT|os.O_TRUNC))
        or event=='os.mkdir') and not Path(os.fsdecode(args[0])).resolve().is_relative_to(SCRATCH):
        raise RuntimeError('write outside scratch')
sys.addaudithook(audit)

def module(path,name):
    spec=importlib.util.spec_from_file_location(name,path)
    result=importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result
acq=module(CORE/'acquire.py','candidate')
launch=module(EVIDENCE/'batch33-passive-closure/launch.py','launcher')
raw=(CORE/'requests'/(EPOCH+'.json')).read_bytes()
document=json.loads(raw)

class Response(io.BytesIO):
    def __init__(self,url,status,data):
        super().__init__(data)
        self.url,self.code,self.headers=url,status,{'Content-Type':'application/octet-stream'}
    def geturl(self):
        return self.url

class Fake:
    def __init__(self,rows,case):
        self.allowed={r['url']:r for r in rows}
        self.case=case
        self.sent=[]
    def open(self,request,timeout):
        url=request.full_url
        if request.get_method()!='GET' or timeout!=15 or url not in self.allowed:
            raise RuntimeError('non-roster request')
        self.sent.append(url)
        row=self.allowed[url]
        status=200
        artifact=('synthetic opaque bytes '+str(row['primary_index'])).encode()
        if row['role']=='jar':
            data=artifact
            if self.case=='redirect':url='https://different.invalid/data'
            if self.case=='oversize':data=b'x'*(row['maximum_bytes']+1)
            if self.case=='empty':data=b''
            if self.case=='missing-jar':status,data=404,b'absence'
        elif row['role']=='signature':
            data=b'arbitrary detached opaque bytes'
        elif row['algorithm']=='sha512' or self.case=='no-checksum':
            status,data=404,b'absent'
        else:
            source=b'wrong' if self.case=='checksum-drift' else artifact
            data=hashlib.new(row['algorithm'],source).hexdigest().encode()
            if self.case=='wrong-name':data+=b'  other.jar'
        return Response(url,status,data)

class Tests(unittest.TestCase):
    def test_exact_roster_and_old_epochs(self):
        self.assertEqual(acq.requests(EPOCH,raw),document['requests'])
        self.assertTrue(set(PREVIOUS).issubset(acq.EPOCHS))
        for key,(expected_count,expected_digest) in PREVIOUS.items():
            retained=(CORE/'requests'/(key+'.json')).read_bytes()
            self.assertEqual(DIGEST(retained),expected_digest)
            self.assertEqual(len(acq.requests(key,retained)),expected_count)
        self.assertEqual(len(document['ordered_mediated_coordinates']),13)
        self.assertEqual(len(document['excluded_existing_data_candidates']),3)
        self.assertEqual(len(document['requests']),50)
        self.assertEqual(sum(r['maximum_bytes'] for r in document['requests']),13107200)
        agent,=[p for p in document['new_primaries'] if p['coordinate']['artifactId']=='org.jacoco.agent']
        self.assertEqual(agent['coordinate']['classifier'],'runtime')
        self.assertTrue(agent['relative_artifact'].endswith('-runtime.jar'))
        self.assertFalse(any(document['authorization'].values()))

    def test_inert_spec_refused_by_both_programs(self):
        p=SCRATCH/'synthetic-outputless-proposal.json'
        spec={'epoch':EPOCH,'proposal':str(CORE/'requests'/(EPOCH+'.json')),
              'proposal_sha256':DIGEST(raw),'source_sha256':DIGEST((CORE/'acquire.py').read_bytes()),
              'proposed_out':str(SCRATCH/'would-be-output'),'acquisition_authorized':False}
        with p.open('x') as stream:
            json.dump(spec,stream)
        args=[str(p),DIGEST(p.read_bytes())]
        with patch.dict(os.environ,{'SYSON_BOOTSTRAP_SOURCE_SHA256':spec['source_sha256']}):
            with patch.object(sys,'argv',['candidate',*args,'bogus','bogus','bogus']):
                with self.assertRaisesRegex(KeyError,'out'):
                    acq.main()
        with patch.object(sys,'argv',['launcher',*args]):
            with self.assertRaisesRegex(KeyError,'out'):
                launch.main()
        self.assertFalse(Path(spec['proposed_out']).exists())

    def reject_reseal(self,obj,why,primary=False):
        data=json.dumps(obj,separators=(',',':')).encode()
        fixed=dict(acq.EPOCHS[EPOCH],sha256=DIGEST(data))
        if primary:
            projection=[(p['relative_artifact'],p['kind']) for p in obj['new_primaries']]
            fixed['primary_sha256']=DIGEST(json.dumps(projection,separators=(',',':')).encode())
        with patch.dict(acq.EPOCHS,{EPOCH:fixed}),self.assertRaisesRegex(ValueError,why):
            acq.requests(EPOCH,data)

    def test_manifest_tamper_and_semantic_reseal_controls(self):
        with self.assertRaisesRegex(ValueError,'manifest hash/size'):
            acq.requests(EPOCH,raw+b' ')
        for key,value in [('url','http://repo.maven.apache.org/maven2/foo'),
                          ('primary_index',True),('maximum_bytes',1048577),
                          ('relative_path','downloads/../outside.jar'),('allowed_status',[200,404])]:
            with self.subTest(key=key):
                obj=json.loads(raw)
                obj['requests'][0][key]=value
                self.reject_reseal(obj,'fixed ordered requests')
        obj=json.loads(raw)
        obj['limits']['per_JAR_bytes']+=1
        self.reject_reseal(obj,'fixed profile')
        obj=json.loads(raw)
        agent,=[p for p in obj['new_primaries'] if p['coordinate']['artifactId']=='org.jacoco.agent']
        agent['relative_artifact']=agent['relative_artifact'].replace('-runtime.jar','.jar')
        agent['coordinate']['classifier']=''
        self.reject_reseal(obj,'fixed JaCoCo runtime classifier',True)

    def collect(self,case,limit=None,budget=16*1024*1024):
        out=SCRATCH/case
        out.mkdir(mode=0o700)
        rows=acq.requests(EPOCH,raw)[:limit]
        fake=Fake(rows,case)
        captured=[]
        total=0
        metadata=acq.helper.Metadata()
        for row in rows:
            result=acq.base.transfer(fake,row,acq.base.destination(out,row),budget-total,metadata)
            if result['status'] not in row['allowed_status']:
                raise ValueError('HTTP status outside disposition')
            captured.append(result)
            total+=result['bytes']
        self.assertEqual(fake.sent,[r['url'] for r in rows])
        return acq.jar_checksums(out,captured)

    def test_50_opaque_responses(self):
        matches=self.collect('positive')
        self.assertEqual(len(matches),20)
        self.assertEqual({m['algorithm'] for m in matches},{'sha1','sha256'})
        self.assertTrue(all(m['match'] is True for m in matches))

    def test_transfer_and_checksum_negatives(self):
        cases=[('oversize','stream byte limit'),('redirect','redirect refused'),
               ('empty','empty captured JAR'),('missing-jar','HTTP status outside disposition'),
               ('checksum-drift','checksum mismatch'),('no-checksum','no available matching checksum'),
               ('wrong-name','checksum filename mismatch')]
        for case,why in cases:
            with self.subTest(case=case),self.assertRaisesRegex(ValueError,why):
                self.collect(case,5)
        with self.assertRaisesRegex(ValueError,'stream byte limit'):
            self.collect('small-aggregate-budget',5,1)

if __name__=='__main__':
    unittest.main(verbosity=2)
