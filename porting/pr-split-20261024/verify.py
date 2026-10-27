#!/usr/bin/env python3
"""Verify split tree identities and dependency topology without compiling or writing Git."""
import hashlib
import json
import pathlib
import subprocess

OUT=pathlib.Path(__file__).resolve().parent
ROOT=OUT.parents[1]

def tree(ref):
    entries={}
    raw=subprocess.check_output(['git','-C',str(ROOT),'ls-tree','-rz',ref])
    for row in raw.split(b'\0'):
        if not row:
            continue
        meta,path=row.split(b'\t',1)
        mode,kind,sha=meta.decode().split()
        entries[path.decode()]={'path':path.decode(),'mode':mode,'type':kind,'sha':sha}
    return entries

def tree_hash(entries):
    root={}
    for path,e in entries.items():
        parts=path.split('/')
        cursor=root
        for part in parts[:-1]:
            cursor=cursor.setdefault(part,{})
        cursor[parts[-1]]=e
    def digest(node):
        records=[]
        for name,value in node.items():
            if 'sha' in value and 'mode' in value:
                mode,sha,key=value['mode'],value['sha'],name.encode()
            else:
                mode,sha,key='40000',digest(value),name.encode()+b'/'
            records.append((key,mode.encode()+b' '+name.encode()+b'\0'+bytes.fromhex(sha)))
        body=b''.join(v for _,v in sorted(records))
        return hashlib.sha1(b'tree '+str(len(body)).encode()+b'\0'+body).hexdigest()
    return digest(root)

def main():
    manifest=json.loads((OUT/'existing-split.json').read_text())
    publication=json.loads((OUT/'published-prs.json').read_text())
    published={r['id']:r for r in publication['pull_requests']}
    baseline=tree(manifest['upstream_baseline'])
    target=tree(manifest['source_revision'])
    reconstructed=dict(baseline)
    overlays={}
    assert tree_hash(baseline)==manifest['baseline_tree']
    for unit in manifest['units']:
        overlay={}
        for dep in unit['dependencies']:
            for path,e in overlays[dep].items():
                assert path not in overlay or overlay[path]==e,(unit['id'],path)
                overlay[path]=e
        if len(unit['dependencies'])>1:
            assert published[unit['id']]['base'].startswith('codex/syson-base-')
        for e in unit['entries']:
            overlay[e['path']]=e
            reconstructed[e['path']]=e
        overlays[unit['id']]=overlay
        assert tree_hash({**baseline,**overlay})==published[unit['id']]['tree'],unit['id']
    assert reconstructed==target
    assert len(published)==len(manifest['units'])==20
    future=json.loads((OUT/'future-prs.json').read_text())
    by_id={r['id']:r for r in future['units']}
    assert len(by_id)==len(future['units'])==47
    state={}
    def visit(sid):
        assert state.get(sid)!=1,('cycle',sid)
        if state.get(sid)==2:
            return
        state[sid]=1
        for dep in by_id[sid]['dependencies']:
            assert dep in by_id
            visit(dep)
        state[sid]=2
    for sid in by_id:
        visit(sid)
    expected={'DOMAIN-NAME','DOMAIN-ID','DOMAIN-QUERY','EDITOR-ROTATE','EDITOR-GEOMETRY','MIGRATION-ID','MODEL','PARSER','UI','LAYOUT','API','DB','RECOVERY','HISTORY','ACCESSIBILITY','EXPORT','EXTENSIONS','DEPS'}
    assert set(future['area_coverage'])==expected
    for ids in future['area_coverage'].values():
        assert ids and set(ids)<=set(by_id)
    for r in by_id.values():
        assert r['state']=='PLANNED_NOT_IMPLEMENTED'
        assert r['lease']=='NOT_GRANTED'
        assert (OUT/'briefs'/f"{r['id']}.md").is_file()
        assert r['paths'] and r['acceptance'] and r['excluded']
    # The dependency tree alone does not make arbitrary overlapping writers safe.
    def ancestors(sid):
        return {d for dep in by_id[sid]['dependencies'] for d in {dep,*ancestors(dep)}}
    for i,a in enumerate(future['units']):
        for b in future['units'][i+1:]:
            for p in a['paths']:
                for q in b['paths']:
                    if p==q or p.startswith(q.rstrip('/')+'/') or q.startswith(p.rstrip('/')+'/'):
                        assert a['id'] in ancestors(b['id']) or b['id'] in ancestors(a['id']),(a['id'],b['id'],p,q)
    result=dict(split_prs=20,frozen_changed_files=manifest['source_changed_files'],
                exact_final_tree_reconstruction=True, independently_computed_split_tree_hashes=20,
                rust_path_dependency_references=manifest['validation']['path_dependency_checks'],
                literal_include_references=manifest['validation']['literal_include_checks'],
                future_pr_briefs=47,future_dependency_graph='ACYCLIC',permanent_areas_covered=18,
                unordered_proposed_write_scope_overlaps=0,
                native_browser_upstream_tests='NOT_RERUN',new_leases='NOT_GRANTED')
    (OUT/'verification.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(result,indent=2))

if __name__=='__main__':
    main()
