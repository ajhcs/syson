#!/usr/bin/env python3
"""Static pinned Syside grammar/source audit; never loads or runs JS/TypeScript."""
import argparse,collections,hashlib,json,pathlib,re,subprocess,zipfile
PIN='1326f234cae23b0e4777d5a48a069b17e9e332f1';OUT=pathlib.Path(__file__).resolve().parent

def decode_template(value):
    """Decode only static JS template escapes; reject interpolation/unknowns."""
    out=[];i=0; escapes={'n':'\n','r':'\r','t':'\t','b':'\b','f':'\f','v':'\v','0':'\0','\\':'\\','`':'`','"':'"',"'":"'",'$':'$'}
    while i<len(value):
        c=value[i]
        if c!='\\':
            if value[i:i+2]=='${': raise ValueError('dynamic interpolation is forbidden')
            out.append(c);i+=1;continue
        i+=1
        if i==len(value):raise ValueError('dangling escape')
        c=value[i]
        if c in escapes:out.append(escapes[c]);i+=1
        elif c=='u':
            assert re.fullmatch(r'[0-9a-fA-F]{4}',value[i+1:i+5]);out.append(chr(int(value[i+1:i+5],16)));i+=5
        elif c=='x':
            assert re.fullmatch(r'[0-9a-fA-F]{2}',value[i+1:i+3]);out.append(chr(int(value[i+1:i+3],16)));i+=3
        elif c=='\n':i+=1
        elif c=='\r':i+=1+(value[i+1:i+2]=='\n')
        else:raise ValueError('unsupported template escape '+repr(c))
    return ''.join(out)

def extract_grammars(source):
    docs=[]
    for m in re.finditer(r'loadGrammarFromJson\(`',source):
        start=m.end();end=start;escaped=False
        while end<len(source):
            c=source[end]
            if c=='`' and not escaped:break
            if c=='\\' and not escaped:escaped=True
            else:escaped=False
            end+=1
        assert end<len(source),'unterminated template'
        encoded=source[start:end];decoded=decode_template(encoded);grammar=json.loads(decoded)
        assert grammar['$type']=='Grammar'
        docs.append({'line':source.count('\n',0,m.start())+1,'encoded_template':encoded,'decoded_json':decoded,'grammar':grammar})
    return docs

def walk(node,path='#'):
    yield path,node
    if isinstance(node,dict):
        for key,value in node.items():yield from walk(value,path+'/'+key)
    elif isinstance(node,list):
        for i,value in enumerate(node):yield from walk(value,path+'@'+str(i))
def resolve(root,reference):
    assert reference.startswith('#/'),reference
    value=root
    for segment in reference[2:].split('/'):
        m=re.fullmatch(r'([^@]+)(?:@(\d+))?',segment);assert m,reference
        value=value[m[1]]
        if m[2] is not None:value=value[int(m[2])]
    return value

def write(name,rows):(OUT/name).write_text(''.join(json.dumps(r,sort_keys=True,ensure_ascii=False)+'\n' for r in rows))
def run(reference,assets,syson_reference):
    def git(*a):return subprocess.check_output(['git','-C',str(reference),*a])
    assert git('rev-parse','HEAD').decode().strip()==PIN
    assert not git('status','--porcelain'),'reference must stay untouched'
    paths=git('ls-tree','-r','--name-only',PIN).decode().splitlines()
    def raw(p):return git('show',PIN+':'+p)
    def sha(b):return hashlib.sha256(b).hexdigest()
    files=[];sources=[];tests=[];grammar_inputs=[];validation=[];notices_source=[];fixture_docs=[]
    for p in paths:
        blob=raw(p);s=blob.decode('utf-8',errors='replace')
        files.append({'path':p,'bytes':len(blob),'sha256':sha(blob),'source_commit':PIN})
        if '__tests__' in p or '/testing/' in p or p.endswith(('.test.ts','.snap')):
            tests.append({'path':p,'sha256':sha(blob),'execution':'NOT RUN','case_label_candidates':re.findall(r'(?:test|it|describe)\(\s*[\'\"]([^\'\"\n]+)',s),'interpretation':'source test/fixture index; not coverage'})
        if re.search(r'(^|/)(LICENSE[^/]*|NOTICE[^/]*|COPYING[^/]*)$',p,re.I):
            notices_source.append({'path':p,'sha256':sha(blob),'source':s,'source_commit':PIN})
        if '__tests__' in p or '/testing/' in p or p.endswith(('.test.ts','.snap')):
            fixture_docs.append({'path':p,'sha256':sha(blob),'source':s,'execution':'NOT RUN'})
        if p.endswith('.langium'):
            grammar_inputs.append({'path':p,'sha256':sha(blob),'source':s})
        if p.endswith(('.ts','.json','.mjs')) and any(part in p for part in ['/src/services/','/src/model/','/src/testing/','/src/utils/','src/generated/ast.ts','langium-config.json','src/stdlib.ts','scripts/clone-sysml-release','scripts/update-stdlib','packages/syside-cli/src/']):
            sources.append({'path':p,'sha256':sha(blob),'source':s,'semantic_execution':'NOT RUN','source_status':'exact candidate source text; Rust behavior unported'})
        if '/services/validation/' in p and p.endswith('.ts') and '__tests__' not in p:
            for m in re.finditer(r'(?m)^[ \t]*@(validateKerML|validateSysML)\(',s):
                pos=m.end();depth=1;quote=None;escaped=False
                while pos<len(s) and depth:
                    c=s[pos]
                    if quote:
                        if c==quote and not escaped:quote=None
                        if c=='\\' and not escaped:escaped=True
                        else:escaped=False
                    elif c in ['"',"'",'`']:quote=c
                    elif c=='(':depth+=1
                    elif c==')':depth-=1
                    pos+=1
                validation.append({'path':p,'line':s.count('\n',0,m.start())+1,'decorator':m[1],'arguments_source':s[m.end():pos-1],'implementation_source_retained':True,'registration_resolution':'syntactic decorator candidate; bounds/defaults/inheritance execution unresolved'})
    write('test-fixture-documents.jsonl',fixture_docs);write('source-notice-documents.jsonl',notices_source);write('source-files.jsonl',files);write('grammar-source-documents.jsonl',grammar_inputs);write('semantic-source-documents.jsonl',sources);write('tests-and-fixtures.jsonl',tests);write('validation-decorators.jsonl',validation)
    dependency_paths=['pnpm-lock.yaml','package.json','packages/syside-languageserver/package.json','packages/syside-cli/package.json','packages/syside-languageserver/langium-config.json']
    write('parser-dependency-source-documents.jsonl',[{'path':p,'source_commit':PIN,'sha256':sha(raw(p)),'source':raw(p).decode(),'artifact_resolution':'source declaration/lock metadata only; not fetched installed dependency proof'} for p in dependency_paths])
    generated='packages/syside-languageserver/src/generated/grammar.ts';gs=raw(generated).decode();docs=extract_grammars(gs)
    rules=[];declarations=[];features=[];refs=[];unresolved=[];node_counts=collections.Counter()
    for doc in docs:
        g=doc['grammar'];name=g['name']
        for i,r in enumerate(g.get('rules',[])):
            dependencies=[]
            for path,node in walk(r):
                if not isinstance(node,dict):continue
                kind=node.get('$type');node_counts[kind]+=bool(kind)
                if kind in ['Action','Assignment','CrossReference','RuleCall','RegexToken','Keyword','PredicateCondition']:
                    features.append({'grammar':name,'rule':r['name'],'relative_path':path,'kind':kind,'declaration':node})
                if kind in ['RuleCall','TerminalRuleCall']:
                    ref=node.get('rule',{}).get('$ref')
                    if ref:dependencies.append(resolve(g,ref)['name'])
            origins=[]
            for source in grammar_inputs:
                pattern=r'(?m)^[ \t]*(?:(?:entry|fragment|terminal|hidden)\s+)*'+re.escape(r['name'])+r'\s*(?:<[^>]*>)?\s*(?:(?:returns|infers)\b|:)'
                for match in re.finditer(pattern,source['source']):
                    origins.append({'path':source['path'],'line':source['source'].count('\n',0,match.start())+1})
            rules.append({'source_declaration_candidates':origins,'source_fragment':'#/rules@'+str(i),'grammar':name,'ordinal':i,'name':r['name'],'kind':r['$type'],'entry':r.get('entry',False),'fragment':r.get('fragment',False),'hidden':r.get('hidden',False),'rule_dependencies':sorted(set(dependencies)),'declaration':r,'source':generated,'source_commit':PIN,'parity_status':'UNPORTED'})
        for group in ['interfaces','types']:
            for i,d in enumerate(g.get(group,[])):
                declarations.append({'grammar':name,'group':group,'ordinal':i,'declaration':d,'parity_status':'UNPORTED'})
        for path,node in walk(g):
            if isinstance(node,dict) and '$ref' in node:
                grammar_reference=node['$ref']
                try:target=resolve(g,grammar_reference); resolved={'name':target.get('name'),'kind':target.get('$type')}
                except (AssertionError,KeyError,TypeError,IndexError) as e:
                    resolved={'error':type(e).__name__};unresolved.append({'grammar':name,'path':path,'reference':grammar_reference})
                refs.append({'grammar':name,'path':path,'reference':grammar_reference,'target':resolved})
        doc['source_path']=generated;doc['source_sha256']=sha(raw(generated));doc['source_commit']=PIN
    write('generated-grammar-documents.jsonl',docs);write('grammar-rules.jsonl',rules);write('grammar-ast-declarations.jsonl',declarations);write('grammar-features.jsonl',features);write('grammar-references.jsonl',refs)
    # Release ZIPs are read as bounded data only, never extracted or loaded.
    syson_pin='6ba2ac64e6b16ee2b05752ad55395eebb87d4910'
    syson_paths=subprocess.check_output(['git','-C',str(syson_reference),'ls-tree','-r','--name-only',syson_pin]).decode().splitlines(); adapters=[]
    for p in syson_paths:
        if p.startswith('backend/application/syson-sysml-import/src/main/') and p.endswith(('.java','.graphqls')):
            b=subprocess.check_output(['git','-C',str(syson_reference),'show',syson_pin+':'+p]);adapters.append({'path':p,'source_commit':syson_pin,'sha256':sha(b),'source':b.decode(),'role':'baseline AST import/translation/diagnostics/API contract; source only'})
    write('syson-import-adapter-source.jsonl',adapters)
    baseline_path='backend/application/syson-sysml-import/src/main/resources/syside-cli.js'
    baseline=subprocess.check_output(['git','-C',str(syson_reference),'show','6ba2ac64e6b16ee2b05752ad55395eebb87d4910:'+baseline_path]);baseline_sha=sha(baseline)
    asset_records=[];members=[];notices=[];libraries=[]
    for archive in sorted(assets.iterdir()):
        if not archive.name.endswith(('.zip','.vsix')):continue
        with zipfile.ZipFile(archive) as z:
            assert sum(i.file_size for i in z.infolist())<50_000_000,'unexpectedly large archive'
            asset_records.append({'name':archive.name,'bytes':archive.stat().st_size,'sha256':sha(archive.read_bytes()),'release_url':'https://github.com/sensmetry/sysml-2ls/releases/tag/0.9.0','source_matching':'candidate release artifact only','members':len(z.infolist())})
            for info in z.infolist():
                if info.is_dir():continue
                b=z.read(info);h=sha(b)
                members.append({'archive':archive.name,'path':info.filename,'bytes':len(b),'sha256':h,'matches_syson_cli_bundle':h==baseline_sha})
                if re.search(r'(^|/)(LICENSE[^/]*|NOTICE[^/]*|COPYING[^/]*)$',info.filename,re.I):
                    notices.append({'archive':archive.name,'path':info.filename,'sha256':h,'source':b.decode('utf-8'),'correspondence':'notice for this release archive; baseline CLI coverage unresolved'})
                if info.filename.endswith(('.sysml','.kerml')):
                    libraries.append({'archive':archive.name,'path':info.filename,'sha256':h,'bytes':len(b),'source':b.decode('utf-8'),'correspondence':'release library bytes; SysON bundled JSON library equivalence unresolved'})
    write('release-archives.jsonl',asset_records);write('release-members.jsonl',members);write('release-notice-documents.jsonl',notices);write('release-library-documents.jsonl',libraries)
    std=raw('packages/syside-base/src/stdlib.ts').decode();declared_files=re.findall(r'^\s*"([^"\n]+\.(?:sysml|kerml))",?\s*$',std,re.M)
    actual_files={r['path'].split('sysml.library/',1)[1] for r in libraries if 'sysml.library/' in r['path']}
    library={'source_path':'packages/syside-base/src/stdlib.ts','declared_version':'2024-12','declared_tree_url':'https://github.com/daumantas-kavolis-sensmetry/SysML-v2-Release/blob/fixes/sysml.library/','declared_raw_url':'https://raw.githubusercontent.com/daumantas-kavolis-sensmetry/SysML-v2-Release/fixes/sysml.library/','source_pin':'OPEN: fixes is mutable; no library Git commit established','declared_files':declared_files,'release_files':sorted(actual_files),'missing_declared_in_release':sorted(set(declared_files)-actual_files),'additional_release_files':sorted(actual_files-set(declared_files)),'license_evidence':'release archive contains sysml.library/LICENSE.GPLv3 and LICENSE.LGPLv3; individual source headers retained; precise baseline JSON relationship unresolved'}
    (OUT/'library-provenance.json').write_text(json.dumps(library,indent=2)+'\n')
    summary={'source_candidate_commit':PIN,'tracked_files':len(files),'grammar_source_files':len(grammar_inputs),'generated_grammars':len(docs),'parser_rules':sum(r['kind']=='ParserRule' for r in rules),'terminal_rules':sum(r['kind']=='TerminalRule' for r in rules),'ast_interface_type_declarations':len(declarations),'grammar_feature_node_entries':len(features),'grammar_reference_entries':len(refs),'unresolved_local_grammar_references':unresolved,'grammar_node_kinds':dict((str(k),v) for k,v in node_counts.items() if k),'validation_decorator_candidates':len(validation),'exact_semantic_source_files':len(sources),'test_fixture_source_locations':len(tests),'release_library_files':len(libraries),'release_notice_documents':len(notices),'baseline_cli_sha256':baseline_sha,'matching_release_members':[r for r in members if r['matches_syson_cli_bundle']],'runtime_execution':'NOT RUN','parity':'OPEN: declarations are not a Rust parser or behavioral proof'}
    (OUT/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary,indent=2))
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--reference',required=True,type=pathlib.Path);p.add_argument('--assets',required=True,type=pathlib.Path);p.add_argument('--syson-reference',required=True,type=pathlib.Path);a=p.parse_args();run(a.reference,a.assets,a.syson_reference)
