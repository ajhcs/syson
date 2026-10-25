#!/usr/bin/env python3
"""Check exact retained F10 source inputs without executing upstream code."""
import hashlib, json, pathlib, subprocess
ROOT=pathlib.Path(__file__).resolve().parents[3]
OUT=pathlib.Path(__file__).resolve().parent
contract=json.loads((OUT/'source-contract.json').read_text())
candidate={}
for line in (ROOT/'porting/source-map/syside-source/semantic-source-documents.jsonl').read_text().splitlines():
    record=json.loads(line)
    candidate[record['path']]=record
for source in contract['sources']:
    if source['revision'].startswith('1326'):
        raw=candidate[source['path']]['source'].encode()
    else:
        raw=subprocess.check_output(['git','-C',str(ROOT),'show',source['revision']+':'+source['path']])
    assert hashlib.sha256(raw).hexdigest()==source['sha256'], source['path']
print(json.dumps({'source_files':len(contract['sources']),'exact_source_hashes':'PASS','upstream_execution':'NOT_RUN'}))
