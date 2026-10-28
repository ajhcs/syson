#!/usr/bin/env python3
"""Static reflection source preparation. Never imports/executes upstream code."""
import argparse
import hashlib
import json
import re
import subprocess
from pathlib import Path
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
CANDIDATE = '1326f234cae23b0e4777d5a48a069b17e9e332f1'
BASELINE = '6ba2ac64e6b16ee2b05752ad55395eebb87d4910'
BUNDLE = 'backend/application/syson-sysml-import/src/main/resources/syside-cli.js'
CANDIDATE_INVENTORY_SHA = '1d51961e3f76efc4a3eecfff72b9a8aea3246d6ce54465c8cf134d19dcdbc324'
BUNDLE_SHA = '47d8fe11fda1b58b6257463087f663be953c2ab6c2b5e1e2ccc44161ed5cd0bd'
def sha(raw): return hashlib.sha256(raw).hexdigest()
def dump(value): return json.dumps(value,sort_keys=True,ensure_ascii=True)+'\n'
def collect():
    inventory = ROOT/'porting/source-map/syside-source/semantic-source-documents.jsonl'
    assert sha(inventory.read_bytes()) == CANDIDATE_INVENTORY_SHA, 'candidate source inventory drift'
    all_docs = [json.loads(line) for line in inventory.read_text().splitlines()]
    docs = {r['path']: r for r in all_docs}
    chosen = ['generated/ast.ts','model/types.ts','services/sysml-ast-reflection.ts','services/parser/parser.ts','utils/ast-util.ts','model/__tests__/type-index.test.ts','services/references/linker.ts','model/implicits.ts','model/metamodel.ts']
    retained=[]
    for suffix in chosen:
        matches=[r for path,r in docs.items() if path.endswith('/'+suffix)]
        assert len(matches)==1,suffix
        r=matches[0];assert sha(r['source'].encode())==r['sha256']
        retained.append(dict(r,source_commit=CANDIDATE,qualification='candidate source; runtime correspondence is not proven'))
    def source(suffix): return next(r['source'] for r in retained if r['path'].endswith('/'+suffix))
    ast=source('generated/ast.ts');reflection=ast[ast.index('export class SysMlAstReflection'):]
    all_types=re.findall(r"'([^']+)'",re.search(r'getAllTypes\(\).*?return \[(.*?)\];',reflection,re.S).group(1))
    assert len(set(all_types))==len(all_types)
    subtype=reflection[reflection.index('protected override computeIsSubtype'):reflection.index('getReferenceType')]
    parents={};clauses=[]
    for match in re.finditer(r'((?:\s*case \w+:)+)\s*\{\s*return (.*?);\s*\}',subtype,re.S):
        cases=re.findall(r'case (\w+):',match.group(1));expression=match.group(2)
        bases=re.findall(r'this\.isSubtype\((\w+), supertype\)',expression)
        assert re.sub(r'this\.isSubtype\(\w+, supertype\)|\s|\|','',expression)=='',expression
        assert bases
        for name in cases: assert name not in parents;parents[name]=bases
        clauses.append(dict(cases=cases,parents=bases,exact_source=match.group(0)))
    assert set(re.findall(r'case (\w+):',subtype))==set(parents)
    assert re.search(r'default:\s*\{\s*return false;',subtype)
    assert all(p in all_types for values in parents.values() for p in values)
    metadata=reflection[reflection.index('getTypeMetaData'):]
    mandatory={}
    for match in re.finditer(r"case '(\w+)':\s*\{\s*return \{\s*name: '\1',\s*mandatory: \[(.*?)\]\s*\};\s*\}",metadata,re.S):
        name,body=match.groups();props=re.findall(r"\{ name: '(\w+)', type: '(array|boolean)' \}",body)
        assert re.sub(r"\{ name: '\w+', type: '(?:array|boolean)' \}|\s|,",'',body)=='',body
        assert len({n for n,t in props})==len(props), 'duplicate local mandatory property'
        mandatory[name]=[dict(name=n,type=t) for n,t in props]
    assert set(re.findall(r"case '(\w+)':",metadata))==set(mandatory)
    assert 'mandatory: []' in metadata
    references=reflection[reflection.index('getReferenceType'):reflection.index('getTypeMetaData')]
    refids=re.findall(r"case '([^']+)':",references);assert len(refids)==8 and 'return Element;' in references
    types=source('model/types.ts');non_types=re.findall(r'ast\.(\w+)',re.search(r'const NON_TYPES = new Set<string>\(\[(.*?)\]\)',types,re.S).group(1))
    assert len(non_types)==5 and all(n in all_types for n in non_types)
    declarations=[dict(name=n,generated_order=i,generated_direct_supertypes=parents.get(n,[]),generated_local_mandatory=mandatory.get(n,[]),filtered_by_NON_TYPES=n in non_types,status='SOURCE_PREPARATION_ONLY; effective inheritance ordering/defaults not runtime-qualified') for i,n in enumerate(all_types)]
    calls=[]
    for r in all_docs:
        for number,line in enumerate(r['source'].splitlines(),1):
            for match in re.finditer(r'typeIndex\.(expandToDerivedTypes|expandAndMerge|chain|isUnion|getAllTypes|getInheritanceChain|getSubtypes|isSubtype)\b',line):
                calls.append(dict(path=r['path'],source_commit=CANDIDATE,source_sha256=r['sha256'],line=number,method=match.group(1),exact_line=line,qualification='literal static occurrence; comments/tests/dynamic calls are not semantic runtime coverage'))
    bundle=subprocess.check_output(['git','-C',str(ROOT),'show',BASELINE+':'+BUNDLE]);assert sha(bundle)==BUNDLE_SHA
    text=bundle.decode();headers=list(re.finditer(r'^// (.+)$',text,re.M));modules=[]
    wanted=['langium/lib/syntax-tree.js','langium/lib/utils/ast-util.js','syside-languageserver/lib/generated/ast.js','syside-languageserver/lib/model/types.js','syside-languageserver/lib/services/sysml-ast-reflection.js','syside-languageserver/lib/services/parser/parser.js']
    for suffix in wanted:
        matches=[(i,m) for i,m in enumerate(headers) if m.group(1).endswith(suffix)];assert len(matches)==1,(suffix,matches)
        i,m=matches[0];end=headers[i+1].start() if i+1<len(headers) else len(text);body=text[m.start():end]
        modules.append(dict(component='embedded bundle source',source_commit=BASELINE,source_path=BUNDLE,bundle_sha256=BUNDLE_SHA,module_label=m.group(1),start_line=text.count('\n',0,m.start())+1,source=body,source_sha256=sha(body.encode()),qualification='exact embedded module text; source label does not prove separate artifact/source correspondence'))
    summary=dict(item_id='PARSER-REFLECTION-PREP-01',source_candidate=CANDIDATE,source_baseline=BASELINE,bundle_sha256=BUNDLE_SHA,candidate_inventory_sha256=CANDIDATE_INVENTORY_SHA,counts=dict(generated_type_names=len(all_types),local_subtype_cases=len(parents),local_mandatory_cases=len(mandatory),local_mandatory_properties=sum(map(len,mandatory.values())),generated_reference_ids=len(refids),NON_TYPES=len(non_types),candidate_documents=len(retained),embedded_modules=len(modules),literal_typeindex_occurrences=len(calls)),generated_reference_ids=refids,NON_TYPES=non_types,generated_type_order=all_types,status='PREPARATION; no runtime execution or reflection implementation',limits=['Counts describe this finite pinned source inventory, not tests, coverage or semantic compatibility.','Embedded module bytes and candidate documents retained separately; equality is not asserted.','No inheritance order is inferred from a different language sort implementation.'])
    return {'candidate-source-documents.jsonl':''.join(dump(r) for r in retained),'embedded-module-documents.jsonl':''.join(dump(r) for r in modules),'generated-type-declarations.jsonl':''.join(dump(r) for r in declarations),'generated-subtype-clauses.jsonl':''.join(dump(r) for r in clauses),'typeindex-call-sites.jsonl':''.join(dump(r) for r in calls),'summary.json':json.dumps(summary,indent=2,sort_keys=True)+'\n'}
def main():
    parser=argparse.ArgumentParser();parser.add_argument('--write',action='store_true');args=parser.parse_args()
    for name,body in collect().items():
        path=HERE/name
        if args.write:path.write_text(body)
        else:assert path.read_text()==body,name+' drift'
    print('Static reflection preparation '+('generated' if args.write else 'verified')+'; no upstream code executed')
if __name__=='__main__':main()
