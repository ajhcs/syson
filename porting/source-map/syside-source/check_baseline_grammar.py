#!/usr/bin/env python3
"""Compare pinned serialized declarations as DATA; never execute upstream JS."""
import argparse, hashlib, json, pathlib, subprocess
from inventory_syside import decode_template, extract_grammars, resolve, walk
HERE=pathlib.Path(__file__).resolve().parent
ROOT=HERE.parents[2]
BASELINE='6ba2ac64e6b16ee2b05752ad55395eebb87d4910'
BUNDLE_PATH='backend/application/syson-sysml-import/src/main/resources/syside-cli.js'
BUNDLE_SHA='47d8fe11fda1b58b6257463087f663be953c2ab6c2b5e1e2ccc44161ed5cd0bd'
CANDIDATE='1326f234cae23b0e4777d5a48a069b17e9e332f1'
CANDIDATE_FILE_SHA='fca1a64377f64c1aefb5476e89227cbc24f064ca40b7a58d6608646cae66149d'
EXPECTED={'KerML':('2b439aeca16aa8f5247f74c541278f93dfefea44098ec8fe1b6f85ac31013829',282),'SysML':('84e3deb1cbcc89c400b56e7b2a0a20eaa64ff6305146bf34edd982ab292898f8',453)}
class CorrespondenceError(ValueError):pass

def canonical_hash(value):
    return hashlib.sha256(json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=True).encode()).hexdigest()

def validate_shape(grammar,name):
    required={'$type','definesHiddenTokens','hiddenTokens','imports','interfaces','isDeclared','name','rules','types','usedGrammars'}
    if not isinstance(grammar,dict) or set(grammar)!=required or grammar.get('$type')!='Grammar' or grammar.get('name')!=name:
        raise CorrespondenceError('grammar root shape drift: '+name)
    for group in ['rules','interfaces','types','hiddenTokens','imports','usedGrammars']:
        if not isinstance(grammar[group],list):raise CorrespondenceError('grammar list shape drift: '+name+'/'+group)
    rules=grammar['rules']
    if len(rules)!=EXPECTED[name][1] or any(not isinstance(r,dict) or r.get('$type') not in ['ParserRule','TerminalRule'] or not isinstance(r.get('name'),str) for r in rules):
        raise CorrespondenceError('grammar rule shape drift: '+name)
    if len({r['name'] for r in rules})!=len(rules):raise CorrespondenceError('duplicate rule: '+name)
    try:
        for path,node in walk(grammar):
            if isinstance(node,dict) and '$ref' in node:resolve(grammar,node['$ref'])
    except (AssertionError,KeyError,TypeError,IndexError) as error:
        raise CorrespondenceError('grammar local reference drift: '+name) from error

def compare(bundle,candidates):
    actual=hashlib.sha256(bundle).hexdigest()
    if actual!=BUNDLE_SHA:raise CorrespondenceError('baseline bundle SHA256 drift')
    try:
        # Esbuild turns the call into `(0, module.loadGrammarFromJson)(`...`)`.
        # This spelling-only normalization changes no newline or template data.
        original=bundle.decode('utf-8')
        decoded=extract_grammars(original.replace('loadGrammarFromJson)(`','loadGrammarFromJson(`'))
    except (AssertionError,ValueError,KeyError,UnicodeError) as error:
        raise CorrespondenceError('baseline static template decode failed') from error
    names=[d['grammar'].get('name') for d in decoded]
    if len(names)!=3 or set(names)!={'LangiumGrammar','KerML','SysML'}:
        raise CorrespondenceError('baseline grammar set drift')
    baseline={d['grammar']['name']:d for d in decoded if d['grammar']['name'] in EXPECTED}
    indexed={}
    for record in candidates:
        if record.get('source_commit')!=CANDIDATE or record.get('source_path')!='packages/syside-languageserver/src/generated/grammar.ts' or record.get('source_sha256')!=CANDIDATE_FILE_SHA:
            raise CorrespondenceError('candidate source metadata drift')
        grammar=record.get('grammar');name=grammar.get('name') if isinstance(grammar,dict) else None
        if name not in EXPECTED or name in indexed:raise CorrespondenceError('candidate grammar set/duplicate drift')
        validate_shape(grammar,name)
        try:
            if json.loads(decode_template(record['encoded_template']))!=grammar or json.loads(record['decoded_json'])!=grammar:
                raise CorrespondenceError('candidate retained representations disagree: '+name)
        except (AssertionError,KeyError,ValueError,TypeError) as error:
            raise CorrespondenceError('candidate static declaration drift: '+name) from error
        indexed[name]=record
    if set(indexed)!=set(EXPECTED):raise CorrespondenceError('candidate grammar set drift')
    rows=[]
    for name,(wanted,_) in EXPECTED.items():
        a=baseline[name]['grammar'];b=indexed[name]['grammar'];validate_shape(a,name)
        ah=canonical_hash(a);bh=canonical_hash(b)
        if ah!=wanted or bh!=wanted or a!=b:raise CorrespondenceError('canonical declaration drift: '+name)
        rows.append({'grammar':name,'bundle_source_line':baseline[name]['line'],'bundle_declaration_sha256':ah,'candidate_declaration_sha256':bh,'equal':True})
    return {'baseline_bundle_sha256':actual,'source_candidate_commit':CANDIDATE,'kind':'static grammar declaration comparison; no JS executed','grammars':rows}

def inputs(reference=ROOT):
    bundle=subprocess.check_output(['git','-C',str(reference),'show',BASELINE+':'+BUNDLE_PATH])
    candidates=[json.loads(line) for line in (HERE/'generated-grammar-documents.jsonl').read_text().splitlines()]
    return bundle,candidates

def main():
    p=argparse.ArgumentParser();p.add_argument('--syson-reference',type=pathlib.Path,default=ROOT);p.add_argument('--write',action='store_true');a=p.parse_args()
    report=compare(*inputs(a.syson_reference));output=HERE/'baseline-grammar-correspondence.json';rendered=json.dumps(report,indent=2)+'\n'
    if a.write:output.write_text(rendered)
    elif output.read_text()!=rendered:raise CorrespondenceError('recorded correspondence output drift')
    print('KerML and SysML declarations match pinned bundle; full artifact/semantic/notice provenance remains OPEN')
if __name__=='__main__':main()
