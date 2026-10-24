#!/usr/bin/env python3
"""Offline declaration/archive-data checks, never Syside parser execution."""
import argparse,collections,hashlib,importlib.util,json,pathlib,subprocess,sys,unittest,zipfile
OUT=pathlib.Path(__file__).resolve().parent;PIN='1326f234cae23b0e4777d5a48a069b17e9e332f1'
p=argparse.ArgumentParser();p.add_argument('--reference',required=True,type=pathlib.Path);p.add_argument('--assets',required=True,type=pathlib.Path);a,remaining=p.parse_known_args();sys.argv=[sys.argv[0],*remaining]
spec=importlib.util.spec_from_file_location('inventory_syside',OUT/'inventory_syside.py');audit=importlib.util.module_from_spec(spec);spec.loader.exec_module(audit)
def git(*args):return subprocess.check_output(['git','-C',str(a.reference),*args])
def raw(path):return git('show',PIN+':'+path)
def rows(name):return [json.loads(l) for l in (OUT/name).read_text().splitlines()]
class SysideSource(unittest.TestCase):
    def test_clean_exact_reference_and_complete_tracked_index(self):
        self.assertEqual(git('rev-parse','HEAD').decode().strip(),PIN);self.assertEqual(git('status','--porcelain'),b'')
        self.assertEqual([r['path'] for r in rows('source-files.jsonl')],git('ls-tree','-r','--name-only',PIN).decode().splitlines())
        self.assertEqual(len(rows('source-files.jsonl')),525)
    def test_five_original_grammars_exact_and_two_generated_documents(self):
        sources=rows('grammar-source-documents.jsonl');self.assertEqual(len(sources),5)
        for r in sources:self.assertEqual(r['source'].encode(),raw(r['path']))
        generated=raw('packages/syside-languageserver/src/generated/grammar.ts').decode()
        parsed=audit.extract_grammars(generated);retained=rows('generated-grammar-documents.jsonl')
        self.assertEqual([g['grammar'] for g in parsed],[g['grammar'] for g in retained]);self.assertEqual(len(retained),2)
        for r in retained:self.assertEqual(json.loads(r['decoded_json']),r['grammar'])
    def test_static_decoder_refuses_code_and_preserves_escapes(self):
        self.assertEqual(audit.decode_template(r'\\\"\n\u0041\x42\`'), '\\"\nAB`')
        self.assertEqual(audit.decode_template(r'\${literal}'),'${literal}')
        for value in ['${dynamic}',r'\q','\\']:
            with self.assertRaises(ValueError):audit.decode_template(value)
    def test_complete_rule_index_and_original_source_candidates(self):
        expected=[]
        for d in rows('generated-grammar-documents.jsonl'):
            expected.extend((d['grammar']['name'],i,r) for i,r in enumerate(d['grammar']['rules']))
        indexed=rows('grammar-rules.jsonl');self.assertEqual([(r['grammar'],r['ordinal'],r['declaration']) for r in indexed],expected)
        self.assertEqual(sum(r['kind']=='ParserRule' for r in indexed),711);self.assertEqual(sum(r['kind']=='TerminalRule' for r in indexed),24)
        self.assertTrue(all(r['source_declaration_candidates'] for r in indexed))
        self.assertTrue(all(r['parity_status']=='UNPORTED' for r in indexed))
    def test_ast_actions_assignments_crossrefs_and_interfaces_retained(self):
        features=rows('grammar-features.jsonl');counts=collections.Counter(r['kind'] for r in features)
        self.assertEqual(counts['Action'],61);self.assertEqual(counts['Assignment'],783);self.assertEqual(counts['CrossReference'],4)
        self.assertEqual(len(rows('grammar-ast-declarations.jsonl')),303)
        expected=[]
        kinds={'Action','Assignment','CrossReference','RuleCall','RegexToken','Keyword','PredicateCondition'}
        for doc in rows('generated-grammar-documents.jsonl'):
            for rule in doc['grammar']['rules']:
                expected.extend((doc['grammar']['name'],rule['name'],path,node) for path,node in audit.walk(rule) if isinstance(node,dict) and node.get('$type') in kinds)
        self.assertEqual([(r['grammar'],r['rule'],r['relative_path'],r['declaration']) for r in features],expected)
    def test_all_local_serialized_grammar_references_resolve(self):
        docs={r['grammar']['name']:r['grammar'] for r in rows('generated-grammar-documents.jsonl')}
        refs=rows('grammar-references.jsonl');self.assertEqual(len(refs),2720)
        for r in refs:
            target=audit.resolve(docs[r['grammar']],r['reference'])
            self.assertEqual(r['target'],{'name':target.get('name'),'kind':target.get('$type')})
        self.assertEqual(json.loads((OUT/'summary.json').read_text())['unresolved_local_grammar_references'],[])
    def test_semantic_and_fixture_source_bytes(self):
        for filename in ['semantic-source-documents.jsonl','test-fixture-documents.jsonl']:
            for r in rows(filename):self.assertEqual(r['source'].encode(),raw(r['path']))
        self.assertEqual(len(rows('test-fixture-documents.jsonl')),83)
        self.assertTrue(all(r['execution']=='NOT RUN' for r in rows('test-fixture-documents.jsonl')))
    def test_release_archives_and_notice_bytes(self):
        for archive in rows('release-archives.jsonl'):
            data=(a.assets/archive['name']).read_bytes();self.assertEqual(hashlib.sha256(data).hexdigest(),archive['sha256'])
        for r in rows('release-notice-documents.jsonl'):
            with zipfile.ZipFile(a.assets/r['archive']) as z:self.assertEqual(r['source'].encode(),z.read(r['path']))
        self.assertEqual(len(rows('release-notice-documents.jsonl')),6)
    def test_library_files_match_declared_names_but_source_pin_open(self):
        p=json.loads((OUT/'library-provenance.json').read_text());self.assertEqual(len(p['declared_files']),94);self.assertEqual(p['missing_declared_in_release'],[]);self.assertEqual(p['additional_release_files'],[])
        self.assertTrue(p['source_pin'].startswith('OPEN'))
        for r in rows('release-library-documents.jsonl'):
            with zipfile.ZipFile(a.assets/r['archive']) as z:self.assertEqual(r['source'].encode(),z.read(r['path']))
    def test_release_server_pair_matches_each_other_not_baseline_cli(self):
        members=rows('release-members.jsonl');indexed={(r['archive'],r['path']):r['sha256'] for r in members}
        self.assertEqual(indexed[('syside-languageserver.zip','syside-languageserver.js')],indexed[('sensmetry.sysml-2ls-0.9.0.vsix','extension/dist/node/language-server/main.js')])
        self.assertEqual(indexed[('syside-languageserver.zip','LICENSE')],indexed[('sensmetry.sysml-2ls-0.9.0.vsix','extension/dist/node/LICENSE')])
        self.assertFalse(any(r['matches_syson_cli_bundle'] for r in members))
if __name__=='__main__':unittest.main(verbosity=2)
