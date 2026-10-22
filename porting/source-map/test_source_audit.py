#!/usr/bin/env python3
"""Source-preservation regression checks. No upstream product code is executed."""
import collections, hashlib, json, pathlib, subprocess, unittest, xml.etree.ElementTree as ET
ROOT=pathlib.Path(__file__).resolve().parents[2]; OUT=ROOT/'porting/source-map'
PIN='6ba2ac64e6b16ee2b05752ad55395eebb87d4910'
def raw(p): return subprocess.check_output(['git','-C',str(ROOT),'show',PIN+':'+p])
def rows(name): return [json.loads(line) for line in (OUT/name).read_text().splitlines()]
def flattened_raw(node):
    yield node.tag,dict(node.attrib),node.text,node.tail
    for child in node: yield from flattened_raw(child)
def flattened_record(node):
    yield node['tag'],node['attributes'],node['text'],node['tail']
    for child in node['children']: yield from flattened_record(child)
class SourcePreservation(unittest.TestCase):
    def test_ecore_exact_bytes_and_entire_package_tree(self):
        docs=rows('ecore-documents.jsonl'); self.assertEqual(len(docs),2)
        for doc in docs:
            blob=raw(doc['path'])
            self.assertEqual(doc['xml'].encode(),blob)
            self.assertEqual(doc['sha256'],hashlib.sha256(blob).hexdigest())
            self.assertEqual(list(flattened_raw(ET.fromstring(blob))),list(flattened_record(doc['parsed_root'])))
    def test_every_classifier_nested_tree_matches_raw(self):
        expected={}
        for doc in rows('ecore-documents.jsonl'):
            for ordinal,node in enumerate(ET.fromstring(raw(doc['path'])).findall('eClassifiers')):
                expected[(doc['path'],node.get('name'))]=(ordinal,node)
        classifiers=rows('metamodel.jsonl'); self.assertEqual(len(classifiers),len(expected))
        for record in classifiers:
            ordinal,node=expected[(record['path'],record['name'])]
            self.assertEqual(list(flattened_raw(node)),list(flattened_record(record['declaration'])))
            self.assertEqual(record['declaration']['source_location'],record['path']+'#//@eClassifiers.'+str(ordinal))
            def check_locations(r):
                seen=collections.Counter()
                for child in r['children']:
                    tag=child['tag']; self.assertEqual(child['source_location'],r['source_location']+'/@'+tag+'.'+str(seen[tag])); seen[tag]+=1
                    check_locations(child)
            check_locations(record['declaration'])
    def test_parameters_enum_literals_annotations_are_not_empty_projections(self):
        counts=collections.Counter()
        for record in rows('metamodel.jsonl'):
            counts.update(tag for tag,_,_,_ in flattened_record(record['declaration']))
        for tag in ['eParameters','eLiterals','eAnnotations']:
            self.assertGreater(counts[tag],0,tag)
        # This pinned XML has no annotation detail entries; do not invent them.
        self.assertEqual(counts['details'], 0)
        print('Preserved nested declaration counts:',dict((tag,counts[tag]) for tag in ['eParameters','eLiterals','eAnnotations','details']))
    def test_loss_of_nested_parameter_is_detectable(self):
        for record in rows('metamodel.jsonl'):
            original=list(flattened_record(record['declaration']))
            if any(tag=='eParameters' for tag,_,_,_ in original):
                changed=[row for row in original if row[0]!='eParameters']
                self.assertNotEqual(original,changed); return
        self.fail('No parameter-bearing declaration to test regression')
    def test_graphql_full_documents_preserve_raw_sources(self):
        for doc in rows('graphql-documents.jsonl'):
            self.assertEqual(doc['sdl'].encode(),raw(doc['path']))
        docs=rows('graphql-client-documents.jsonl'); self.assertEqual(len(docs),15)
        for doc in docs:
            blob=raw(doc['path']); self.assertEqual(doc['source'].encode(),blob)
            self.assertEqual(doc['sha256'],hashlib.sha256(blob).hexdigest())
            self.assertTrue(all(c['parser_status']=='not_parsed' for c in doc['candidates']))
    def test_lockfile_entries_and_integrity_preserved(self):
        records=rows('npm-locked-inputs.jsonl'); expected={}
        for lock in {r['lockfile'] for r in records}:
            for loc,value in json.loads(raw(lock))['packages'].items():
                if loc: expected[(lock,loc)]=value
        self.assertEqual(len(records),len(expected))
        for record in records:
            value=expected[(record['lockfile'],record['package_location'])]
            for key in ['version','resolved','integrity']:
                self.assertEqual(record[key],value.get(key))
            self.assertIsNone(record['source_git_sha'])
    def test_snapshot_and_unresolved_maven_source_not_claimed_pinned(self):
        records=rows('inherited-maven-inputs.jsonl')
        self.assertTrue(any(r['local_property_expansion']=='2.5.5-SNAPSHOT' for r in records))
        self.assertTrue(any(r['local_property_expansion']=='2026.9.3' for r in records))
        for record in records:
            self.assertIsNone(record['source_git_sha']); self.assertIsNone(record['artifact_checksum'])
    def test_syside_version_and_notice_gap_match_raw(self):
        record=json.loads((OUT/'syside-provenance.json').read_text()); blob=raw(record['path']); s=blob.decode()
        self.assertEqual(record['sha256'],hashlib.sha256(blob).hexdigest())
        self.assertIn('exports2.Version = "0.9.0";',s)
        self.assertIn('version: "0.9.0",',s)
        self.assertIn(record['embedded_repository_url'],s)
        self.assertEqual(record['inline_notice_marker_matches'],[])
        self.assertIsNone(record['source_git_sha'])
if __name__=='__main__': unittest.main(verbosity=2)
