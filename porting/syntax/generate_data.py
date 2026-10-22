#!/usr/bin/env python3
"""Generate Rust-readable grammar DATA from the exact audited source JSON."""
import argparse,hashlib,json,pathlib
ROOT=pathlib.Path(__file__).resolve().parents[2];OUT=ROOT/'rust/crates/syson-syntax/data';PIN='1326f234cae23b0e4777d5a48a069b17e9e332f1'
def normalized(node,root):
 if isinstance(node,dict):
  if '$ref' in node:
   group,index=node['$ref'][2:].split('@');return {'target':root[group][int(index)]['name']}
  return {k:normalized(v,root) for k,v in node.items()}
 if isinstance(node,list):return [normalized(v,root) for v in node]
 return node
p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');a=p.parse_args();generated={};contracts=None
for line in (ROOT/'porting/source-map/syside-source/generated-grammar-documents.jsonl').open():
 doc=json.loads(line);assert doc['source_commit']==PIN;g=doc['grammar']
 generated[g['name'].lower()+'.json']=json.dumps(g,ensure_ascii=False,sort_keys=True,separators=(',',':'))+'\n'
 current={r['name']:{k:normalized(r[k],g) for k in ['definition','fragment','hidden']} for r in g['rules'] if r['$type']=='TerminalRule'}
 if contracts is None:contracts=current
 else:assert contracts==current
 generated.setdefault('terminals.json',json.dumps(current,ensure_ascii=False,sort_keys=True,separators=(',',':'))+'\n')
notices=[json.loads(l) for l in (ROOT/'porting/source-map/syside-source/source-notice-documents.jsonl').open()]
generated['LICENSE-SYSIDE']=next(r['source'] for r in notices if r['path']=='LICENSE')
generated['PROVENANCE.json']=json.dumps({'source_candidate_commit':PIN,'source_path':'packages/syside-languageserver/src/generated/grammar.ts','artifact_correspondence':'OPEN','runtime_parser_conversion':'UNPORTED','data_sha256':{n:hashlib.sha256(v.encode()).hexdigest() for n,v in generated.items()}},indent=2)+'\n'
for name,value in generated.items():
 target=OUT/name
 if a.check:assert target.read_text()==value,name
 else:target.write_text(value)
print('Grammar data '+('verified' if a.check else 'generated')+'; no upstream code executed')
# This is a declaration/engine disposition, never a parity PASS ledger.
rows=[]
for line in (ROOT/'porting/source-map/syside-source/generated-grammar-documents.jsonl').open():
 g=json.loads(line)['grammar'];rules=g['rules'];seen=set();pending=[i for i,r in enumerate(rules) if r.get('entry')]
 def refs(n):
  if isinstance(n,dict):
   if '$ref' in n and n['$ref'].startswith('#/rules@'):yield int(n['$ref'].split('@')[1])
   for v in n.values():yield from refs(v)
  elif isinstance(n,list):
   for v in n:yield from refs(v)
 while pending:
  i=pending.pop()
  if i in seen:continue
  seen.add(i);pending.extend(refs(rules[i]))
 for i,r in enumerate(rules):
  rows.append({'id':g['name']+'::'+r['name'],'kind':r['$type'],'source_candidate_commit':PIN,'source_fragment':'#/rules@'+str(i),'entry_reachable':i in seen,'engine_disposition':'terminal_contract_checked' if r['$type']=='TerminalRule' else ('CST_recognizer_candidate' if i in seen else 'UnsupportedEntry'),'ast_linking_validation_status':'UNPORTED','runtime_parity_status':'UNPORTED','individual_rule_qualification':'NOT ESTABLISHED','test_refs':[]})
value=''.join(json.dumps(r,sort_keys=True)+'\n' for r in rows);target=ROOT/'porting/syntax/RULE-DISPOSITION.jsonl'
if a.check:assert target.read_text()==value
else:target.write_text(value)
