#!/usr/bin/env python3
"""Bounded Git/ZIP reading and owned input staging only; never compile or launch."""
from pathlib import Path
import hashlib,io,json,os,subprocess,zipfile
OUT=Path(__file__).resolve().parent
REPO=Path('/workspace/repos/builderbob/syson')
JAR=Path('/workspace/build/builderbob/syson-audit-20261026/syson-application-2026.9.3.jar')
LABEL='b380dada415ce05155150f23cb3b74ae21cc45e4'
TARGET='6ba2ac64e6b16ee2b05752ad55395eebb87d4910'
EXPECTED='85c031352695d6ac8f059f0215ca493c58a16805a111233998dd443d6990c9e1'
MODULES=['syson-sysml-metamodel-edit','syson-diagram-common-view','syson-standard-diagrams-view']
CHANGED={'BOOT-INF/lib/'+m+'-2026.9.3.jar' for m in MODULES}
sha=lambda b:hashlib.sha256(b).hexdigest()
canonical=lambda v:json.dumps(v,sort_keys=True,separators=(',',':')).encode()
def write(name,value):(OUT/name).write_text(json.dumps(value,indent=2)+'\n')
def safe(name):
    q=name[:-1] if name.endswith('/') else name
    assert len(name)<=1024 and '\\' not in name and ':' not in name
    assert all(p not in ('','.','..') for p in q.split('/'))
def git(*args):
    env=dict(os.environ,GIT_NO_LAZY_FETCH='1',GIT_OPTIONAL_LOCKS='0',GIT_TERMINAL_PROMPT='0')
    return subprocess.check_output(['git','--no-replace-objects','-c','core.fsmonitor=false','-C',str(REPO),*args],env=env,timeout=30)
def source(revision,path,target):
    info=git('ls-tree','-z',revision,'--',path).decode().split('\0')
    assert len(info)==2 and not info[-1]
    head,actual_path=info[0].split('\t');mode,kind,blob=head.split()
    assert actual_path==path and mode=='100644' and kind=='blob'
    raw=git('cat-file','blob',blob);assert len(raw)<=16*1024*1024
    assert hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()==blob
    data={'blob':blob,'mode':mode,'sha256':sha(raw),'bytes':len(raw)}
    if target:data['text']=raw.decode('utf-8') if path.endswith('.java') else None
    destination=OUT/('target' if target else 'label')/path
    destination.parent.mkdir(parents=True,exist_ok=True);destination.write_bytes(raw)
    if target and path.endswith('.java'):
        staged=OUT/'src'/path;staged.parent.mkdir(parents=True,exist_ok=True);staged.write_bytes(raw)
    return data
# Capture all stable ZipInfo fields. Physical offsets and compressed length are
# separate representation observations; uncompressed payload remains exact.
def metadata(i):
    return {'filename':i.filename,'orig_filename':i.orig_filename,'date_time':list(i.date_time),
      'compress_type':i.compress_type,'comment_hex':i.comment.hex(),'extra_hex':i.extra.hex(),
      'create_system':i.create_system,'create_version':i.create_version,'extract_version':i.extract_version,
      'reserved':i.reserved,'flag_bits':i.flag_bits,'volume':i.volume,'internal_attr':i.internal_attr,
      'external_attr':i.external_attr,'CRC':i.CRC,'file_size':i.file_size}
def row(z,i):
    assert i.file_size<=16*1024*1024
    raw=z.read(i);assert len(raw)==i.file_size
    return {'entry':i.filename,'sha256':sha(raw),'bytes':len(raw),'metadata_sha256':sha(canonical(metadata(i)))}
def index(z):
    infos=z.infolist();assert len(infos)<=100000
    names=[i.filename for i in infos];assert len(names)==len(set(names))
    for name in names:safe(name)
    return infos
def signed(z,infos):
    signatures=[];digests=[]
    for i in infos:
        u=i.filename.upper()
        if u.startswith('META-INF/') and (u.endswith(('.SF','.RSA','.DSA','.EC')) or u.split('/')[-1].startswith('SIG-')):signatures.append(i.filename)
        if u=='META-INF/MANIFEST.MF':
            assert i.file_size<=4*1024*1024
            for line in z.read(i).decode('ISO-8859-1').splitlines():
                if line and not line.startswith(' ') and ':' in line:
                    key=line.split(':',1)[0].lower()
                    if key.endswith('-digest') or '-digest-' in key:digests.append(i.filename+'#'+line.split(':',1)[0])
    return signatures,digests
def family(name,root):
    if not name.startswith(root):return False
    tail=name[len(root):]
    return tail=='.class' or (tail.startswith('$') and tail.endswith('.class') and '/' not in tail)
def equivalent(name,root):
    if family(name,root):return True
    if name.startswith('META-INF/versions/'):
        parts=name.split('/',3)
        return len(parts)==4 and family(parts[3],root)
    return False
for directory in ['lib','src','classes','empty-sourcepath']:(OUT/directory).mkdir()
with JAR.open('rb') as stream:assert hashlib.file_digest(stream,'sha256').hexdigest()==EXPECTED
rows=[];queries={};sources=[]
for line in git('diff','--no-ext-diff','--no-renames','--name-status',LABEL,TARGET).decode().splitlines():
    status,path=line.split('\t');safe(path);assert status in ['M','A']
    label=source(LABEL,path,False) if status=='M' else None
    target=source(TARGET,path,True)
    rows.append({'path':path,'status':status,'label':label,'target':target})
    if '/src/main/java/' in path:
        root=path.split('/src/main/java/',1)[1].removesuffix('.java');queries[path]=('family',root)
        sources.append(str(OUT/'src'/path))
    elif '/src/main/resources/' in path:queries[path]=('resource',path.split('/src/main/resources/',1)[1])
assert len(rows)==125 and len(sources)==7 and len(queries)==121
packet={'schema':1,'artifact':{'sha256':EXPECTED,'source_revision':LABEL,'target_revision':TARGET},'delta':rows,'containers':[],'scan':{'containers':['BOOT-INF/classes'],'queries':sorted(queries)},'hits':[],'compiled':[]}
inspection={'metadata_hash_fields':'All recorded ZipInfo metadata except header_offset and compress_size; those two retained separately','outer':[],'changed_containers':{},'unsupported_flag_bits':[]}
with zipfile.ZipFile(JAR) as outer:
    outer_infos=index(outer);signatures,digests=signed(outer,outer_infos)
    packet['artifact'].update(signatures=signatures,manifest_digests=digests)
    for i in outer_infos:
        inspection['outer'].append({'metadata':metadata(i),'header_offset':i.header_offset,'compress_size':i.compress_size})
        if i.flag_bits & ~2048:inspection['unsupported_flag_bits'].append(['outer',i.filename,i.flag_bits])
    # Scan the outer application classes alongside all nested library names.
    for i in outer_infos:
        if not i.filename.startswith('BOOT-INF/classes/'):continue
        relative=i.filename.removeprefix('BOOT-INF/classes/')
        for p,(kind,entry) in queries.items():
            if (kind=='resource' and relative==entry) or (kind=='family' and equivalent(relative,entry)):
                h=row(outer,i);h['entry']=relative;h.update(source_path=p,container='BOOT-INF/classes');packet['hits'].append(h)
    libs=[i for i in outer_infos if i.filename.startswith('BOOT-INF/lib/') and i.filename.endswith('.jar')]
    assert len(libs)==312 and len({Path(i.filename).name for i in libs})==312
    for i in libs:
        assert i.file_size<=128*1024*1024
        raw=outer.read(i);assert len(raw)==i.file_size
        (OUT/'lib'/Path(i.filename).name).write_bytes(raw)
        packet['scan']['containers'].append(i.filename)
        c={'entry':i.filename,'sha256':sha(raw),'compression':i.compress_type,'members':None}
        with zipfile.ZipFile(io.BytesIO(raw)) as nested:
            members=index(nested);s,d=signed(nested,members);c.update(signatures=s,manifest_digests=d)
            if i.filename in CHANGED:
                c['members']=[row(nested,m) for m in members]
                inspection['changed_containers'][i.filename]=[{'metadata':metadata(m),'header_offset':m.header_offset,'compress_size':m.compress_size} for m in members]
                for m in members:
                    if m.flag_bits & ~2048:inspection['unsupported_flag_bits'].append([i.filename,m.filename,m.flag_bits])
            for m in members:
                for p,(kind,entry) in queries.items():
                    if (kind=='resource' and m.filename==entry) or (kind=='family' and equivalent(m.filename,entry)):
                        h=row(nested,m);h.update(source_path=p,container=i.filename);packet['hits'].append(h)
        packet['containers'].append(c)
assert len(inspection['changed_containers'])==3
assert sum(len(c['members'] or []) for c in packet['containers'])<=20000
write('packet-precompile.json',packet);write('archive-inspection.json',inspection)
args=['--release','21','-proc:none','-implicit:none','-sourcepath',str(OUT/'empty-sourcepath'),'-classpath',os.pathsep.join(str(p) for p in sorted((OUT/'lib').iterdir())),'-d',str(OUT/'classes'),*sources]
write('javac-inputs.json',{'args':args,'sources':sources,'classpath_jars':312,'source_sha256':{str(p):sha(p.read_bytes()) for p in (OUT/'src').rglob('*.java')}})
write('staging-receipt.json',{'status':'INSPECTION_STAGED_COMPILE_PENDING','artifact_sha256':EXPECTED,'label':LABEL,'target':TARGET,'delta_rows':125,'runtime_queries':121,'java_sources':7,'nested_jars':312,'hits':len(packet['hits']),'changed_member_rows':sum(len(c['members'] or []) for c in packet['containers']),'unsupported_flag_bits':inspection['unsupported_flag_bits'],'packet_sha256':sha((OUT/'packet-precompile.json').read_bytes()),'stager_sha256':sha(Path(__file__).read_bytes()),'execution':'Git object/ZIP reading plus owned staging only; no compiler, app, native process or network execution'})
print(json.dumps({'status':'INSPECTION_STAGED_COMPILE_PENDING','sources':7,'jars':312,'hits':len(packet['hits']),'changed_members':sum(len(c['members'] or []) for c in packet['containers']),'unsupported_flag_entries':len(inspection['unsupported_flag_bits'])}))
