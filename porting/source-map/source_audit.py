#!/usr/bin/env python3
"""Preserve exact pinned declaration inputs and provenance, without downloads."""
import collections, hashlib, json, pathlib, re, subprocess, xml.etree.ElementTree as ET
ROOT=pathlib.Path(__file__).resolve().parents[2]
OUT=ROOT/'porting/source-map'
PIN='6ba2ac64e6b16ee2b05752ad55395eebb87d4910'
def git(*args): return subprocess.check_output(['git','-C',str(ROOT),*args])
def raw(p): return git('show',PIN+':'+p)
def text(p): return raw(p).decode('utf-8')
def digest(b): return hashlib.sha256(b).hexdigest()
def write(name,rows): (OUT/name).write_text(''.join(json.dumps(r,sort_keys=True,ensure_ascii=False)+'\n' for r in rows))
def tree(node):
    return {'tag':node.tag,'attributes':dict(node.attrib),'text':node.text,'tail':node.tail,'children':[tree(c) for c in node]}
paths=git('ls-tree','-r','--name-only',PIN).decode().splitlines()
write('ecore-documents.jsonl',[{'path':p,'sha256':digest(raw(p)),'xml':text(p),'parsed_root':tree(ET.fromstring(raw(p))), 'external_references':'unresolved; raw and parsed declarations are not executable semantics'} for p in paths if p.endswith('.ecore')])
# Preserve whole TS/TSX source alongside each tagged-template candidate. This is
# a source-retention fallback, NOT a substitute GraphQL or TypeScript parser.
gql=[]
for p in paths:
    if p.endswith(('.ts','.tsx','.js','.jsx')) and '/src/' in p:
        s=text(p); candidates=[]
        for m in re.finditer(r'\bgql\s*`',s):
            start=m.end(); pos=start; escaped=False
            while pos<len(s):
                if s[pos]=='`' and not escaped: break
                if s[pos]=='\\' and not escaped: escaped=True
                else: escaped=False
                pos+=1
            candidates.append({'line':s.count('\n',0,m.start())+1,'body':s[start:pos],'dynamic_interpolation':'${' in s[start:pos],'terminated':pos<len(s),'parser_status':'not_parsed'})
        if candidates: gql.append({'path':p,'sha256':digest(raw(p)),'source':s,'candidates':candidates,'resolution':'exact whole source retained; field arguments/nullability/fragments and interpolation semantics unparsed'})
write('graphql-client-documents.jsonl',gql)
ns={'m':'http://maven.apache.org/POM/4.0.0'}
def mt(el,name): return el.findtext('m:'+name,namespaces=ns)
poms={p:ET.fromstring(raw(p)) for p in paths if p.endswith('pom.xml')}
def props_for(p,seen=()):
    if p in seen: return {}
    root=poms[p]; parent=root.find('m:parent',ns); props={}
    if parent is not None:
        rel=mt(parent,'relativePath')
        if rel:
            target=str((ROOT/pathlib.PurePosixPath(p).parent/rel).resolve().relative_to(ROOT))
            if target in poms: props.update(props_for(target,(*seen,p)))
    for prop in root.findall('m:properties/*',ns): props[prop.tag.split('}',1)[-1]]=prop.text
    return props
maven=[]
for p,root in poms.items():
    props=props_for(p)
    for ordinal,node in enumerate(root.findall('.//m:dependency',ns)):
        group=mt(node,'groupId') or ''; name=mt(node,'artifactId') or ''; expr=mt(node,'version')
        if group.startswith(('org.eclipse.sirius','org.eclipse.emf')) or 'sirius' in name:
            resolved=expr
            if resolved:
                for _ in range(10):
                    nxt=re.sub(r'\$\{([^}]+)\}',lambda m:props.get(m[1]) or m[0],resolved)
                    if nxt==resolved: break
                    resolved=nxt
            maven.append({'manifest':p,'dependency_ordinal':ordinal,'group':group,'artifact':name,'version_expression':expr,'local_property_expansion':resolved,'scope':mt(node,'scope') or 'compile/default','source_git_sha':None,'artifact_checksum':None,'snapshot_timestamp':None,'closure':'OPEN: version declaration only; no effective Maven resolution or source artifact audit'})
write('inherited-maven-inputs.jsonl',maven)
locks=[]
for p in paths:
    if p.endswith('package-lock.json'):
        obj=json.loads(raw(p))
        for loc,package in obj.get('packages',{}).items():
            if not loc: continue
            locks.append({'lockfile':p,'package_location':loc,'name':package.get('name') or loc.rsplit('node_modules/',1)[-1],'version':package.get('version'),'resolved':package.get('resolved'),'integrity':package.get('integrity'),'license_declaration':package.get('license'),'dev':package.get('dev',False),'optional':package.get('optional',False),'link':package.get('link',False),'source_git_sha':None,'audit':'lock metadata only; bytes and source provenance not independently verified'})
write('npm-locked-inputs.jsonl',locks)
bundle='backend/application/syson-sysml-import/src/main/resources/syside-cli.js'; s=text(bundle)
modules=[]
for m in re.finditer(r'(?m)^// (.+)$',s):
    name=m[1]
    if 'node_modules/' in name or name.startswith(('../syside-','src/')) or name=='package.json':
        modules.append({'line':s.count('\n',0,m.start())+1,'source_label':name})
package_versions=collections.defaultdict(list)
for row in modules:
    m=re.search(r'\.pnpm/([^/]+?@[^/]+)/node_modules/',row['source_label'])
    if m: package_versions[m[1]].append(row['line'])
markers=[{'line':i,'text':line} for i,line in enumerate(s.splitlines(),1) if re.search(r'copyright|licen[cs]e|SPDX',line,re.I)]
write('syside-bundle-modules.jsonl',modules)
write('syside-bundled-package-labels.jsonl',[{'pnpm_label':name,'module_label_lines':lines,'source_resolution':'embedded bundler label only; not a lockfile or source commit'} for name,lines in sorted(package_versions.items())])
provenance={'path':bundle,'sha256':digest(raw(bundle)),'bytes':len(raw(bundle)),'embedded_cli_version':'0.9.0','embedded_languageserver_version':'0.9.0','embedded_repository_url':'https://gitlab.com/sensmetry/public/sysml-2ls','embedded_repository_directory':'packages/syside-cli','embedded_cli_package_source_lines':[113992,114044],'embedded_language_version_lines':[113565,113572],'source_git_sha':None,'source_map_reference':'index.js.map','source_map_tracked':any(p.endswith('/index.js.map') for p in paths),'inline_notice_marker_matches':markers,'closure':'OPEN: bundled bytes pinned; build source revision, transformation inputs and complete third-party notices unresolved'}
(OUT/'syside-provenance.json').write_text(json.dumps(provenance,indent=2)+'\n')
summary={'pinned_sha':PIN,'ecore_documents':sum(p.endswith('.ecore') for p in paths),'graphql_source_files_retained':len(gql),'gql_tagged_template_candidates':sum(len(g['candidates']) for g in gql),'sirius_emf_maven_declarations':len(maven),'locked_package_locations':len(locks),'sirius_locked_package_locations':sum(r['name'].startswith('@eclipse-sirius/') for r in locks),'bundle_module_labels':len(modules),'bundle_pnpm_package_labels':len(package_versions),'bundle_notice_marker_matches':len(markers),'runtime_oracle':'NOT RUN','source_closure':'OPEN; counts measure declarations or locations, not unique runtime dependencies or coverage'}
(OUT/'audit-summary.json').write_text(json.dumps(summary,indent=2)+'\n'); print(json.dumps(summary,indent=2))
