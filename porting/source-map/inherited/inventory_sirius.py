#!/usr/bin/env python3
"""Read an existing exact detached Sirius source checkout; no network/builds."""
import argparse, collections, hashlib, json, pathlib, re, subprocess, xml.etree.ElementTree as ET
PIN='3191e2b8990373afee3415f6b26f8c5039a28321'
OUT=pathlib.Path(__file__).resolve().parent
parser=argparse.ArgumentParser();parser.add_argument('--reference',required=True,type=pathlib.Path); args=parser.parse_args(); REF=args.reference.resolve()
def git(*a):return subprocess.check_output(['git','-C',str(REF),*a])
assert git('rev-parse','HEAD').decode().strip()==PIN, 'wrong source reference'
assert not git('status','--porcelain'), 'reference checkout must be untouched'
paths=git('ls-tree','-r','--name-only',PIN).decode().splitlines(); available=set(paths)
# Pinned git batch blobs avoid executing source and ignore checkout filters.
proc=subprocess.Popen(['git','-C',str(REF),'cat-file','--batch'],stdin=subprocess.PIPE,stdout=subprocess.PIPE)
def raw(p):
    proc.stdin.write((PIN+':'+p+'\n').encode());proc.stdin.flush(); header=proc.stdout.readline().decode().split(); assert header[1]=='blob',header
    blob=proc.stdout.read(int(header[2]));assert proc.stdout.read(1)==b'\n'; return blob
def write(name,rows): (OUT/name).write_text(''.join(json.dumps(r,sort_keys=True,ensure_ascii=False)+'\n' for r in rows))
def sha(b):return hashlib.sha256(b).hexdigest()
def xml_tree(n): return {'tag':n.tag,'attributes':dict(n.attrib),'text':n.text,'tail':n.tail,'children':[xml_tree(c) for c in n]}
files=[]; sdls=[]; surfaces=[]; tests=[]; modules=[]; licenses=[]
for p in paths:
    blob=raw(p); ext=pathlib.PurePosixPath(p).suffix; s=blob.decode('utf-8',errors='replace')
    files.append({'path':p,'sha256':sha(blob),'bytes':len(blob),'source_commit':PIN})
    if p.endswith('.graphqls'): sdls.append({'path':p,'sha256':sha(blob),'sdl':s,'schema_resolution':'unparsed source declaration; runtime composition unresolved'})
    is_test='/src/test/' in p or re.search(r'\.(test|spec|cy)\.[jt]sx?$',p)
    if is_test: tests.append({'path':p,'sha256':sha(blob),'execution':'NOT RUN','source_commit':PIN,'case_count_is_coverage':False})
    if p.endswith('.java') and '/src/main/' in p:
        categories=[]
        if '/datafetchers/' in p or re.search(r'(DataFetcher|GraphQL|Subscription)',pathlib.PurePosixPath(p).name): categories.append('graphql_source_candidate')
        if '/controllers/' in p or '/rest/' in p or re.search(r'(RestService|RestController)',p): categories.append('rest_http_source_candidate')
        if re.search(r'(serializ|serializer|deserializ|objectmapper|jsonresource)',p,re.I): categories.append('serialization_source_candidate')
        if '/editingcontext/' in p or '/collaborative/' in p: categories.append('editing_context_collaboration_candidate')
        if categories:
            signatures=[{'line':s.count('\n',0,m.start())+1,'text':m.group().strip()} for m in re.finditer(r'(?m)^\s*@(?:RequestMapping|GetMapping|PostMapping|PutMapping|PatchMapping|DeleteMapping)\b[^\n]*',s)]
            surfaces.append({'path':p,'sha256':sha(blob),'categories':categories,'http_annotation_candidates':signatures,'behavior_resolution':'source candidate only; routing/injection/overrides unassessed'})
    if p.endswith(('pom.xml','package.json')):
        modules.append({'manifest':p,'sha256':sha(blob),'kind':'maven' if p.endswith('pom.xml') else 'npm','source_commit':PIN,'runtime_inclusion':'unresolved'})
    if re.search(r'(^|/)(LICENSE[^/]*|NOTICE[^/]*|COPYING[^/]*)$',p,re.I): licenses.append({'path':p,'sha256':sha(blob),'text':s,'resolution':'supplied notice text; transitive obligations unresolved'})
write('sirius-source-files.jsonl',files);write('sirius-graphql-documents.jsonl',sdls);write('sirius-source-surfaces.jsonl',surfaces);write('sirius-tests.jsonl',tests);write('sirius-modules.jsonl',modules);write('sirius-notice-documents.jsonl',licenses)
# Follow the actual master include order, classpath resources by default, and
# record ordered declarations. Do not interpret SQL, evaluate preconditions,
# substitute properties, or execute migrations.
BASE='packages/sirius-web/backend/sirius-web-infrastructure/src/main/resources/'
MASTER=BASE+'db/db.changelog-master.xml'; docs=[]; steps=[]; includes=[]; unresolved=[]; seen=set()
ns={'l':'http://www.liquibase.org/xml/ns/dbchangelog'}
def resolve(parent,target,relative):
    return str(pathlib.PurePosixPath(parent).parent/target) if relative=='true' else BASE+target

def visit(p,ancestry=()):
    if p in ancestry: unresolved.append({'path':p,'reason':'include_cycle'});return
    if p not in available: unresolved.append({'path':p,'reason':'missing_local_resource'});return
    if p in seen: return
    seen.add(p); blob=raw(p);root=ET.fromstring(blob)
    docs.append({'path':p,'sha256':sha(blob),'xml':blob.decode(),'parsed_root':xml_tree(root),'source_commit':PIN})
    for ordinal,child in enumerate(root):
        tag=child.tag.rsplit('}',1)[-1]
        if tag=='include':
            target=resolve(p,child.get('file'),child.get('relativeToChangelogFile'))
            includes.append({'from':p,'ordinal':ordinal,'attributes':dict(child.attrib),'resolved_path':target})
            visit(target,(*ancestry,p))
        elif tag=='includeAll': unresolved.append({'path':p,'reason':'includeAll_not_interpreted','attributes':dict(child.attrib)})
        elif tag=='changeSet':
            steps.append({'path':p,'ordinal':ordinal,'attributes':dict(child.attrib),'declaration':xml_tree(child),'execution':'NOT RUN'})
            for sqlfile in child.findall('.//l:sqlFile',ns):
                target=resolve(p,sqlfile.get('path'),sqlfile.get('relativeToChangelogFile'))
                if target not in available: unresolved.append({'path':target,'reason':'sqlFile_missing'})
                else: docs.append({'path':target,'sha256':sha(raw(target)),'sql':raw(target).decode(),'execution':'NOT RUN'})
visit(MASTER)
write('sirius-liquibase-documents.jsonl',docs);write('sirius-liquibase-includes.jsonl',includes);write('sirius-liquibase-changesets.jsonl',steps)
root_includes=[r for r in includes if r['from']==MASTER]
summary={'source_commit':PIN,'tracked_source_files':len(files),'module_manifest_locations':len(modules),'graphql_sdl_documents':len(sdls),'surface_candidate_files':len(surfaces),'test_source_resource_locations':len(tests),'liquibase_master':MASTER,'liquibase_version_roots':len(root_includes),'liquibase_documents':len(docs),'liquibase_changesets':len(steps),'liquibase_include_edges':len(includes),'liquibase_unresolved':unresolved,'runtime_artifact_correspondence':'OPEN: candidate source only','runtime_oracle':'NOT RUN','count_meaning':'source/declaration locations; not unique runtime dependencies or feature coverage'}
(OUT/'sirius-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary,indent=2))
proc.stdin.close();proc.wait();assert proc.returncode==0
