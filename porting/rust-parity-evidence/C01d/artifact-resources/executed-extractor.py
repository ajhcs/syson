#!/usr/bin/env python3
"""Bounded read-only ZIP observations; retain resources under ordinal paths."""
import hashlib
import io
import json
from pathlib import Path
import subprocess
import zipfile

base=Path('/workspace/build/builderbob/syson-rust-parity-batch4-20261026')
out=base/'artifact-resources'
out.mkdir()
(out/'executed-extractor.py').write_bytes(Path(__file__).read_bytes())
jar=Path('/workspace/build/builderbob/syson-audit-20261026/syson-application-2026.9.3.jar')
digest=lambda data:hashlib.sha256(data).hexdigest()
with jar.open('rb') as stream:actual=hashlib.file_digest(stream,'sha256').hexdigest()
assert actual=='85c031352695d6ac8f059f0215ca493c58a16805a111233998dd443d6990c9e1'
repos={'syson':(Path('/workspace/repos/builderbob/syson'),'6ba2ac64e6b16ee2b05752ad55395eebb87d4910'),'sirius':(Path('/workspace/worktrees/builderbob/syson/reference-sirius'),'3191e2b8990373afee3415f6b26f8c5039a28321')}
paths={key:subprocess.check_output(['git','-C',str(repo),'ls-tree','-r','--name-only',rev]).decode().splitlines() for key,(repo,rev) in repos.items()}
resources=[];containers=[];metadata=[]
def scan(container,z,prefix='',module=''):
    infos=z.infolist();assert len(infos)<=100_000
    names=[i.filename for i in infos];assert len(names)==len(set(names)),'duplicate archive member'
    for info in infos:
        name=info.filename
        if not name.startswith(prefix):continue
        relative=name[len(prefix):]
        selected=(relative.startswith('schema/') and relative.endswith('.graphqls')) or (relative.startswith('db/') and relative.endswith(('.xml','.sql','.json','.yaml','.yml'))) or relative=='application.properties'
        props=relative.endswith(('pom.properties','build-info.properties'))
        if not (selected or props):continue
        assert info.file_size<=8*1024*1024
        raw=z.read(info);assert len(raw)==info.file_size
        if props:
            metadata.append(dict(container=container,entry=relative,sha256=digest(raw),text=raw.decode('ISO-8859-1')))
        if selected:
            retained=f'{len(resources):04d}'+Path(relative).suffix
            (out/retained).write_bytes(raw)
            row=dict(container=container,entry=relative,sha256=digest(raw),bytes=len(raw),retained=retained,source_matches=[])
            for key,(repo,rev) in repos.items():
                suffix=f'/{module}/src/main/resources/{relative}' if module else '/src/main/resources/'+relative
                for path in paths[key]:
                    if path.endswith(suffix):
                        supplied=subprocess.check_output(['git','-C',str(repo),'show',rev+':'+path])
                        row['source_matches'].append(dict(component=key,revision=rev,path=path,sha256=digest(supplied),bytes_equal=supplied==raw))
            resources.append(row)
with zipfile.ZipFile(jar) as outer:
    scan('BOOT-INF/classes',outer,'BOOT-INF/classes/','syson-application')
    jars=[i for i in outer.infolist() if i.filename.startswith('BOOT-INF/lib/') and i.filename.endswith('.jar')]
    assert len(jars)<=1024
    for info in jars:
        assert info.file_size<=100*1024*1024
        raw=outer.read(info)
        containers.append(dict(entry=info.filename,sha256=digest(raw),bytes=len(raw)))
        stem=Path(info.filename).name.removesuffix('-2026.9.3.jar')
        with zipfile.ZipFile(io.BytesIO(raw)) as nested:scan(info.filename,nested,module=stem)
loaded=[]
for line in (base/'native-02/syson-stdout.txt').read_text().splitlines():
    marker='Processing the GraphQL schema: '
    if marker in line:loaded.append(line.split(marker,1)[1])
report=dict(artifact_sha256=actual,source_label_revision='b380dada415ce05155150f23cb3b74ae21cc45e4',target_revision=repos['syson'][1],nested_jars=containers,metadata=metadata,resources=resources,observed_loaded_graphql_urls=loaded,boundary='Exact selected resource byte correspondence only; Maven coordinate metadata are declarations, not source-build attestation. Effective schema is the native introspection response. Whole C01 remains OPEN.')
(out/'inventory.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({'nested_jars':len(containers),'metadata_entries':len(metadata),'resources':len(resources),'graphql_resources':sum(r['entry'].endswith('.graphqls') for r in resources),'observed_loaded_graphql':len(loaded),'selected_source_matches':sum(len(r['source_matches']) for r in resources),'mismatched_source_resources':sum(not m['bytes_equal'] for r in resources for m in r['source_matches'])}))
