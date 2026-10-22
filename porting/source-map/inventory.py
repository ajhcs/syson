#!/usr/bin/env python3
"""Read pinned Git blobs only; never execute upstream build or import code."""
import collections, hashlib, json, pathlib, re, subprocess, xml.etree.ElementTree as ET
ROOT = pathlib.Path(__file__).resolve().parents[2]
OUT = ROOT / 'porting/source-map'
PIN = '6ba2ac64e6b16ee2b05752ad55395eebb87d4910'
def git(*args): return subprocess.check_output(['git', '-C', str(ROOT), *args])
assert git('rev-parse', PIN).decode().strip() == PIN
paths = git('ls-tree', '-r', '--name-only', PIN).decode().splitlines()
blobs = {}
def raw(p):
    if p not in blobs: blobs[p] = git('show', PIN + ':' + p)
    return blobs[p]
def text(p): return raw(p).decode('utf-8', errors='replace')
def line_at(s, offset): return s.count('\n', 0, offset) + 1
def write(name, rows):
    (OUT / name).write_text(''.join(json.dumps(r, ensure_ascii=False, sort_keys=True) + '\n' for r in rows))
def status(): return {'port_status':'unassessed', 'oracle_status':'not_run', 'rust_tests':[]}
ns={'m':'http://maven.apache.org/POM/4.0.0'}
def tag(el, name): return el.findtext('m:'+name, namespaces=ns)
modules=[]
for p in paths:
    if p.endswith('pom.xml'):
        el=ET.fromstring(raw(p)); base=p.rsplit('/',1)[0] if '/' in p else '.'
        owned=[q for q in paths if q.startswith(base+'/') and not any(q.startswith(str(pathlib.PurePosixPath(base)/child)+'/') for child in [x.text for x in el.findall('m:modules/m:module', ns)])] if base!='.' else [q for q in paths if '/' not in q]
        modules.append({'id':'maven:'+str(tag(el,'artifactId')), 'manifest':p,'kind':'maven','path':base,'artifact':tag(el,'artifactId'),'declared_version':tag(el,'version'), 'children':[x.text for x in el.findall('m:modules/m:module',ns)],'source_files':sum('/src/main/' in q for q in owned),'test_files':sum('/src/test/' in q for q in owned), **status()})
    elif p.endswith('package.json'):
        obj=json.loads(raw(p)); modules.append({'id':'npm:'+obj.get('name',p),'manifest':p,'path':str(pathlib.PurePosixPath(p).parent),'kind':'npm','declared_version':obj.get('version'),'scripts':obj.get('scripts',{}),'engines':obj.get('engines',{}),**status()})
write('modules.jsonl', modules)
deps=[]
for p in paths:
    if p.endswith('pom.xml'):
        root=ET.fromstring(raw(p))
        for node in root.findall('.//m:dependency',ns):
            deps.append({'manifest':p,'ecosystem':'maven', 'group':tag(node,'groupId'), 'name':tag(node,'artifactId'),'version_expression':tag(node,'version'),'scope':tag(node,'scope') or 'compile/default','type':tag(node,'type'),'resolved':False})
    elif p.endswith('package.json'):
        obj=json.loads(raw(p))
        for scope in ['dependencies','devDependencies','peerDependencies']:
            for name,version in obj.get(scope,{}).items(): deps.append({'manifest':p,'ecosystem':'npm','name':name,'version_expression':version,'scope':scope,'resolved':False})
write('declared-dependencies.jsonl',deps)
files=[]; licenses=[]; tests=[]; symbols=[]; schemas=[]; operations=[]; migrations=[]; imports=collections.defaultdict(set)
for p in paths:
    b=raw(p); s=b.decode('utf-8',errors='replace'); ext=pathlib.PurePosixPath(p).suffix
    spdx=sorted(set(re.findall(r'SPDX-License-Identifier:\s*([^\r\n*<>]+)', s)))
    files.append({'path':p,'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b),'extension':ext,'spdx_markers':spdx})
    if spdx or re.search(r'(^|/)(LICENSE[^/]*|NOTICE[^/]*|COPYING[^/]*)$',p,re.I):
        licenses.append({'path':p,'sha256':hashlib.sha256(b).hexdigest(),'spdx_markers':spdx,'review_status':'declared_text_only; no transitive_license_clearance'})
    if '/src/test/' in p or re.search(r'\.(spec|test|cy)\.[jt]sx?$',p):
        kind='java_test_source' if p.endswith('.java') else 'test_resource'
        if p.endswith('.cy.ts'): kind='cypress_spec'
        elif '.spec.' in p: kind='playwright_spec' if 'playwright' in p else 'frontend_spec'
        tests.append({'path':p,'kind':kind,'execution':'not_run','declared_cases':re.findall(r'(?:test|it|describe)\(\s*[\'\"]([^\'\"\n]+)',s) if ext in ['.ts','.tsx','.js'] else [],'java_test_annotations':len(re.findall(r'@(Test|ParameterizedTest|RepeatedTest|TestFactory)\b',s)) if ext=='.java' else None})
    if p.endswith('.java') and '/src/main/' in p:
        for m in re.finditer(r'(?m)^\s*public\s+(?:(?:static|final|abstract|default|synchronized)\s+)*(?:[\w<>?,.\[\] ]+\s+)?([A-Za-z_]\w*)\s*\(([^;{}]*)\)\s*(?:throws[^{};]*)?\{',s):
            symbols.append({'path':p,'line':line_at(s,m.start()),'name':m.group(1),'parameters':m.group(2).strip(),'kind':'public_java_callable','classification':'syntactic_candidate_not_feature',**status()})
        for name in re.findall(r'(?m)^import\s+(?:static\s+)?([^;]+);',s):
            if not name.startswith(('java.','javax.','org.eclipse.syson.')): imports['.'.join(name.split('.')[:4])].add(p)
    if p.startswith('frontend/') and '/src/' in p and ext in ['.ts','.tsx']:
        symbols.append({'path':p,'kind':'frontend_source_file','classification':'module_not_feature',**status()})
    if p.endswith('.graphqls'):
        for m in re.finditer(r'(?m)^(?:(extend)\s+)?(type|input|interface|enum|scalar|union)\s+(\w+)',s):
            schemas.append({'path':p,'line':line_at(s,m.start()),'kind':m.group(2),'name':m.group(3),'extension':bool(m.group(1)),**status()})
    if '/src/' in p and ext in ['.ts','.tsx']:
        for m in re.finditer(r'\b(query|mutation|subscription)\s+(\w+)\s*[(\{]',s): operations.append({'path':p,'line':line_at(s,m.start()),'kind':m.group(1),'name':m.group(2),**status()})
    if 'migration' in p.lower(): migrations.append({'path':p,**status()})
write('source-files.jsonl',files); write('license-evidence.jsonl',licenses); write('upstream-tests.jsonl',tests); write('code-surface.jsonl',symbols); write('graphql-types.jsonl',schemas); write('graphql-operations.jsonl',operations); write('migration-evidence.jsonl',migrations)
write('external-java-imports.jsonl',[{'prefix':k,'source_files':sorted(v),'resolution':'external implementation not vendored by this index'} for k,v in sorted(imports.items())])
features=[]
for p in paths:
    if p.startswith('doc/content/modules/user-manual/pages/') and p.endswith('.adoc') and '/release-notes/' not in p:
        for m in re.finditer(r'(?m)^(={1,4})\s+(.+)$',text(p)):
            features.append({'id':p+':'+str(line_at(text(p),m.start())),'title':m.group(2),'heading_level':len(m.group(1)),'evidence':{'path':p,'line':line_at(text(p),m.start())},'meaning':'documentation_heading; may describe absence or future work','upstream_implementation':'requires source and oracle reconciliation',**status()})
write('documented-features.jsonl',features)
metamodel=[]
for p in paths:
    if p.endswith('.ecore'):
        root=ET.fromstring(raw(p))
        for cls in root.findall('eClassifiers'):
            metamodel.append({'path':p,'name':cls.get('name'),'kind':cls.get('{http://www.w3.org/2001/XMLSchema-instance}type'),'abstract':cls.get('abstract'),'super_types':cls.get('eSuperTypes'),'features':[dict(f.attrib) for f in cls.findall('eStructuralFeatures')],'operations':[dict(o.attrib) for o in cls.findall('eOperations')],**status()})
write('metamodel.jsonl',metamodel)
api_path='doc/content/modules/developer-guide/assets/attachments/sirius-web-openapi.json'
api=json.loads(raw(api_path)); endpoints=[]
for path,item in api.get('paths',{}).items():
    for method,op in item.items():
        if method in ['get','post','put','patch','delete','options','head','trace']:
            endpoints.append({'evidence':api_path,'path':path,'method':method.upper(),'operation_id':op.get('operationId'),'summary':op.get('summary'),'parameters':op.get('parameters',[]),'request_body':op.get('requestBody'),'responses':op.get('responses',{}),'warning':'bundled Sirius OpenAPI; deployment support must be verified',**status()})
write('rest-endpoints.jsonl',endpoints)
summary={'upstream_sha':PIN,'tracked_files':len(paths),'maven_modules':sum(m['kind']=='maven' for m in modules),'npm_packages':sum(m['kind']=='npm' for m in modules),'documentation_headings':len(features),'code_surface_candidates':len(symbols),'metamodel_classifiers':len(metamodel),'rest_endpoints':len(endpoints),'graphql_types':len(schemas),'graphql_operations':len(operations),'test_files_and_resources':len(tests),'dependency_declarations':len(deps),'license_evidence_files':len(licenses),'all_runtime_checks':'NOT RUN','inventory_limit':'Finite pinned tracked-tree inventory; no claim of complete inherited behavior or resolved transitive dependencies.'}
(OUT/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps(summary,indent=2))
