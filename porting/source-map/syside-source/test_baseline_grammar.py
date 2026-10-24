#!/usr/bin/env python3
"""Static source comparison regressions; no upstream runtime execution."""
import copy,json,unittest
import check_baseline_grammar as checker
class BaselineGrammar(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.bundle,cls.candidates=checker.inputs()
    def test_exact_pinned_declarations_match_recorded_result(self):
        result=checker.compare(self.bundle,self.candidates)
        self.assertEqual(result,json.loads((checker.HERE/'baseline-grammar-correspondence.json').read_text()))
        self.assertEqual([r['bundle_source_line'] for r in result['grammars']],[45569,60250])
    def test_bundle_byte_drift_is_rejected_before_decode(self):
        with self.assertRaisesRegex(checker.CorrespondenceError,'bundle SHA256'):checker.compare(self.bundle+b'\n',self.candidates)
    def test_candidate_shape_drift_is_rejected(self):
        records=copy.deepcopy(self.candidates);records[0]['grammar']['rules']={}
        with self.assertRaisesRegex(checker.CorrespondenceError,'shape drift'):checker.compare(self.bundle,records)
    def test_candidate_declaration_drift_is_rejected(self):
        records=copy.deepcopy(self.candidates);records[0]['grammar']['rules'][0]['name']='ChangedEntry'
        with self.assertRaises(checker.CorrespondenceError):checker.compare(self.bundle,records)
    def test_consistent_reencoded_declaration_still_fails_pinned_hash(self):
        records=copy.deepcopy(self.candidates);grammar=records[0]['grammar'];grammar['rules'][0]['name']='ChangedEntry'
        decoded=json.dumps(grammar);records[0]['decoded_json']=decoded
        records[0]['encoded_template']=decoded.replace('\\','\\\\').replace('`','\\`').replace('${','\\${')
        with self.assertRaisesRegex(checker.CorrespondenceError,'canonical declaration drift'):checker.compare(self.bundle,records)
    def test_candidate_metadata_and_duplicate_drift_rejected(self):
        records=copy.deepcopy(self.candidates);records[0]['source_commit']='0'*40
        with self.assertRaisesRegex(checker.CorrespondenceError,'metadata drift'):checker.compare(self.bundle,records)
        with self.assertRaisesRegex(checker.CorrespondenceError,'duplicate drift'):checker.compare(self.bundle,[self.candidates[0],self.candidates[0]])
    def test_retained_template_drift_is_rejected(self):
        records=copy.deepcopy(self.candidates);records[0]['encoded_template']='${runCode()}'
        with self.assertRaisesRegex(checker.CorrespondenceError,'static declaration drift'):checker.compare(self.bundle,records)
    def test_shape_checker_rejects_unresolved_reference(self):
        grammar=copy.deepcopy(self.candidates[0]['grammar']);grammar['rules'][0]['definition']={'$ref':'#/rules@999999'}
        with self.assertRaisesRegex(checker.CorrespondenceError,'reference drift'):checker.validate_shape(grammar,'KerML')
if __name__=='__main__':unittest.main(verbosity=2)
