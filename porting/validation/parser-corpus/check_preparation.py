#!/usr/bin/env python3
"""Verify pending corpus bytes/source refs only; never invoke an upstream/runtime parser."""
import argparse
import hashlib
import json
import pathlib
import re
import prepare


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--repository',required=True,type=pathlib.Path)
    a=p.parse_args()
    root=pathlib.Path(__file__).resolve().parent
    expected=prepare.build(a.repository)
    for name,data in expected.items():
        if (root/name).read_bytes()!=data:raise ValueError('prepared source/fixture mismatch: '+name)
    manifest=json.loads((root/'manifest.json').read_bytes())
    dispositions={}
    path=a.repository/'porting/syntax/RULE-DISPOSITION.jsonl'
    for line in path.open():
        row=json.loads(line);dispositions[row['id']]=row
    ids=set();files=set();assertions=set()
    for case in manifest['fixtures']:
        if case['id'] in ids:raise ValueError('duplicate case id')
        ids.add(case['id']);files.add(case['path'])
        rel=pathlib.PurePosixPath(case['path'])
        if len(rel.parts)!=2 or rel.parts[0]!='fixtures' or '..' in rel.parts:raise ValueError('unexpected fixture path')
        data=(root/rel).read_bytes();text=data.decode('utf-8')
        if not 0<len(data)<=1024 or len(data)!=case['bytes']:raise ValueError('fixture byte bound/count')
        if hashlib.sha256(data).hexdigest()!=case['sha256']:raise ValueError('fixture hash mismatch')
        if re.search(r'\b(?:import|expose)\b',text):raise ValueError('imports/exposes excluded from this prepared corpus')
        if case['observation']['status']!='PENDING_NOT_RUN' or case['observation']['run_artifacts'] or case['expected_wire_golden'] is not None:raise ValueError('preparation cannot invent observations/goldens')
        for field in ['reference_exit_code','lexer_diagnostics','parser_diagnostics','semantic_diagnostics','serialized_ast']:
            if case['observation'][field] is not None:raise ValueError('preparation cannot invent observation '+field)
        for ref in case['provenance']['source_refs']:
            rule=dispositions[ref['language']+'::'+ref['rule']]
            if rule['source_fragment']!=ref['fragment']:raise ValueError('wrong source fragment')
            if ref['activation']=='automatic_hidden_terminal':
                if ref['rule'] not in ['ML_NOTE','SL_NOTE','WS'] or rule['kind']!='TerminalRule':raise ValueError('not a fixed hidden terminal')
            elif ref['activation']!='entry_reachable' or not rule['entry_reachable']:raise ValueError('rule not entry reachable')
        for item in case['layer_assertions']:
            if item['id'] in assertions:raise ValueError('duplicate assertion id')
            assertions.add(item['id'])
    if len(ids)!=12 or sum(c['bytes'] for c in manifest['fixtures'])!=manifest['bounds']['total_fixture_bytes']:raise ValueError('corpus bounds differ')
    if {str(p.relative_to(root)) for p in (root/'fixtures').iterdir() if p.is_file()}!=files:raise ValueError('unlisted fixture files')
    print(json.dumps({'status':'PREPARATION_CHECK_PASS','fixtures':len(ids),'assertions':len(assertions),'total_bytes':manifest['bounds']['total_fixture_bytes'],'upstream_execution':'NOT_RUN','goldens':0}))


if __name__=='__main__':main()
