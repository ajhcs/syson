#!/usr/bin/env python3
"""Offline checks of pinned inherited source retention; no product execution."""
import argparse, hashlib, json, pathlib, re, subprocess, sys, unittest, xml.etree.ElementTree as ET
OUT=pathlib.Path(__file__).resolve().parent
p=argparse.ArgumentParser(); p.add_argument('--reference',required=True,type=pathlib.Path); args,rest=p.parse_known_args(); sys.argv=[sys.argv[0],*rest]
REF=args.reference;PIN='3191e2b8990373afee3415f6b26f8c5039a28321'
def git(*a):return subprocess.check_output(['git','-C',str(REF),*a])
def rows(n):return [json.loads(l) for l in (OUT/n).read_text().splitlines()]
def blob(path):return git('show',PIN+':'+path)
class InheritedAudit(unittest.TestCase):
    def test_reference_commit_and_clean_state(self):
        self.assertEqual(git('rev-parse','HEAD').decode().strip(),PIN);self.assertEqual(git('status','--porcelain'),b'')
    def test_every_tracked_source_is_indexed(self):
        expected=git('ls-tree','-r','--name-only',PIN).decode().splitlines()
        records=rows('sirius-source-files.jsonl');self.assertEqual([r['path'] for r in records],expected)
        self.assertEqual(len(expected),10683)
    def test_all_graphql_sdl_exact_bytes(self):
        expected={p for p in git('ls-tree','-r','--name-only',PIN).decode().splitlines() if p.endswith('.graphqls')}
        records=rows('sirius-graphql-documents.jsonl');self.assertEqual({r['path'] for r in records},expected)
        for r in records:self.assertEqual(r['sdl'].encode(),blob(r['path']))
    def test_liquibase_recursive_sources_and_order(self):
        records=rows('sirius-liquibase-documents.jsonl');self.assertEqual(len(records),29)
        includes=rows('sirius-liquibase-includes.jsonl');self.assertEqual(len(includes),28)
        paths={r['path'] for r in records};self.assertTrue(all(r['resolved_path'] in paths for r in includes))
        master='packages/sirius-web/backend/sirius-web-infrastructure/src/main/resources/db/db.changelog-master.xml'
        roots=[r for r in includes if r['from']==master];self.assertEqual(len(roots),8)
        self.assertEqual([r['attributes']['file'].split('/')[2] for r in roots],['2024.3','2024.5','2024.11','2025.1','2025.2','2025.4','2025.10','2026.1'])
        ns={'l':'http://www.liquibase.org/xml/ns/dbchangelog'}
        expected_changes=0; expected_declarations=[]
        def flatten_xml(n):
            yield (n.tag,dict(n.attrib),n.text,n.tail)
            for child in n: yield from flatten_xml(child)
        def flatten_record(n):
            yield (n['tag'],n['attributes'],n['text'],n['tail'])
            for child in n['children']: yield from flatten_record(child)
        for r in records:
            raw=blob(r['path']);self.assertEqual(r['xml'].encode(),raw);self.assertEqual(r['sha256'],hashlib.sha256(raw).hexdigest())
            nodes=ET.fromstring(raw).findall('l:changeSet',ns)
            expected_changes+=len(nodes)
            expected_declarations.extend((r['path'],list(flatten_xml(n))) for n in nodes)
        actual_declarations=[(r['path'],list(flatten_record(r['declaration']))) for r in rows('sirius-liquibase-changesets.jsonl')]
        self.assertEqual(actual_declarations,expected_declarations)
        self.assertEqual(len(actual_declarations),expected_changes);self.assertEqual(expected_changes,20)
    def test_source_candidates_remain_unproven_artifacts(self):
        r=rows('verified-source-candidates.jsonl');self.assertEqual(len(r),4)
        for c in r:
            self.assertEqual(c['artifact_correspondence'],'UNPROVEN')
            self.assertEqual(c['observed_refs'].get('refs/tags/'+c['tag']+'^{}',c['observed_refs']['refs/tags/'+c['tag']]),c['commit'])
        docs=rows('candidate-source-documents.jsonl')
        self.assertTrue(any('<version>2.5.5-SNAPSHOT</version>' in d['source'] for d in docs))
        self.assertTrue(any('Bundle-Version: 8.1.2' in d['source'] for d in docs))
    def test_syside_separate_notice_and_test_execution_gaps(self):
        d=next(d for d in rows('candidate-source-documents.jsonl') if d['path']=='scripts/build.mjs')
        self.assertIn('getLicenseText(dependencies)',d['source']);self.assertIn('"LICENSE"',d['source'])
        self.assertTrue(all(r['execution']=='NOT RUN' for r in rows('sirius-tests.jsonl')))
        self.assertTrue(all(r['execution']=='NOT RUN' for r in rows('sirius-liquibase-changesets.jsonl')))
if __name__=='__main__':unittest.main(verbosity=2)
