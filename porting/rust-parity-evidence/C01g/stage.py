#!/usr/bin/env python3
"""Owned ZIP/Git read-and-extract only; no compiler/native execution."""
import pathlib,zipfile,io,hashlib,json,subprocess
ROOT=pathlib.Path(__file__).resolve().parent
REPO=pathlib.Path('/workspace/repos/builderbob/syson')
JAR=pathlib.Path('/workspace/build/builderbob/syson-audit-20261026/syson-application-2026.9.3.jar')
PIN='6ba2ac64e6b16ee2b05752ad55395eebb87d4910'
SOURCE='backend/application/syson-application-configuration/src/main/resources/'
CONTAINER='BOOT-INF/lib/syson-application-configuration-2026.9.3.jar'
sha=lambda raw:hashlib.sha256(raw).hexdigest()
with JAR.open('rb') as f: artifact=hashlib.file_digest(f,'sha256').hexdigest()
assert artifact=='85c031352695d6ac8f059f0215ca493c58a16805a111233998dd443d6990c9e1'
assert not (ROOT/'payload').exists()
(ROOT/'payload').mkdir()
rows=[]
with zipfile.ZipFile(JAR) as outer:
    nested=outer.read(CONTAINER)
    classpath=outer.read('BOOT-INF/classpath.idx')
    (ROOT/'classpath.idx').write_bytes(classpath)
    with zipfile.ZipFile(io.BytesIO(nested)) as z:
        names=[n for n in z.namelist() if n.startswith(('kerml.libraries/','sysml.libraries/')) and n.endswith('.json')]
        assert len(names)==len(set(names)) and len(names)<=128
        for name in names:
            assert len(name.split('/'))==2 and '..' not in name and '\\' not in name
            raw=z.read(name);assert len(raw)<=4*1024*1024
            source=SOURCE+name
            original=subprocess.check_output(['git','--no-replace-objects','-C',str(REPO),'show',PIN+':'+source],timeout=20)
            assert raw==original
            target=ROOT/'payload'/name;target.parent.mkdir(exist_ok=True);target.write_bytes(raw)
            rows.append(dict(classpath_path=name,file='payload/'+name,bytes=len(raw),sha256=sha(raw),source_path=source,source_revision=PIN,source_sha256=sha(original)))
json_output=dict(claim='GIT_ZIP_BYTES_STAGED_NO_NATIVE_EXECUTION',artifact_sha256=artifact,container=CONTAINER,container_sha256=sha(nested),source_revision=PIN,resources=rows,classpath_idx_sha256=sha(classpath),uri_bindings='AWAIT_ACTUAL_NATIVE_OBSERVER')
(ROOT/'staging.json').write_text(json.dumps(json_output,indent=2)+'\n')
print(json.dumps(dict(status='STAGED_SOURCE_EQUALS_ARTIFACT',resources=len(rows),bytes=sum(r['bytes'] for r in rows),stager_sha256=sha(pathlib.Path(__file__).read_bytes()))))
