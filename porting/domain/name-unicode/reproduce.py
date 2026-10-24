#!/usr/bin/env python3
"""Verify F44 pinned source identity; execute no Java or Rust converter."""
import hashlib,json,pathlib,re,subprocess
ROOT=pathlib.Path(__file__).resolve().parents[3]
OUT=pathlib.Path(__file__).resolve().parent
contract=json.loads((OUT/'SOURCE-CONTRACT.json').read_text())
for item in contract['upstream_sources']:
    raw=subprocess.check_output(['git','-C',str(ROOT),'show',item['revision']+':'+item['path']])
    assert hashlib.sha256(raw).hexdigest()==item['sha256'],item['path']
    assert (ROOT/item['path']).read_bytes()==raw,item['path']
implementation=contract['implementation']
assert hashlib.sha256((ROOT/implementation['path']).read_bytes()).hexdigest()==implementation['sha256']
tests=(ROOT/'rust/crates/syson-domain/tests/name_unicode.rs').read_text()
assert len(re.findall(r'^#\[test\]',tests,re.M))==contract['prepared_native_test_functions']
print(json.dumps({'pinned_source_files':len(contract['upstream_sources']),'implementation_bytes':'UNCHANGED','prepared_test_functions':contract['prepared_native_test_functions'],'upstream_runtime':'NOT_RUN'}))
