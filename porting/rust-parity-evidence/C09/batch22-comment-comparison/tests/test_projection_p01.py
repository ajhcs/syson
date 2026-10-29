"""Exercise the public raw-input kernel against real pinned capture admission.

SYSON_C09B1_FIXTURES points to the C09a fixture schema plus projection_sha256.
Optional capture_admission overrides the sibling capture_p01.py path. All input
paths and the existing temporary_root are supplied and admitted by the root job.
No native/compiler/browser processes are launched by these tests.
"""
import copy
import hashlib
import json
import os
from pathlib import Path
import tempfile
from types import ModuleType
import unittest


class ProjectionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.config = json.loads(Path(os.environ['SYSON_C09B1_FIXTURES']).read_bytes())
        directory = Path(__file__).resolve().parents[1]
        source = directory / 'projection_p01.py'
        raw = source.read_bytes()
        if hashlib.sha256(raw).hexdigest() != cls.config['projection_sha256']:
            raise ValueError('unreviewed projection test subject')
        cls.subject = ModuleType('reviewed_p01_projection')
        cls.subject.__file__ = str(source)
        exec(compile(raw, str(source), 'exec'), cls.subject.__dict__)
        cls.capture = Path(cls.config.get('capture_admission', directory / 'capture_p01.py'))
        cls.constructor = Path(cls.config['constructor_comparator'])
        cls.temporary_root = Path(cls.config['temporary_root']).resolve()
        if not cls.temporary_root.is_relative_to('/workspace') or not cls.temporary_root.is_dir():
            raise ValueError('tests require an existing admitted Samsung temporary directory')
        cls.raw = {epoch: {name: Path(path).read_bytes() for name, path in paths.items()}
                   for epoch, paths in cls.config['epochs'].items()}
        if set(cls.raw) != {'15', '25'} or any(
            set(blobs) != {'corpus', 'native', 'rust', 'receipt'} for blobs in cls.raw.values()
        ):
            raise ValueError('both complete fixture epochs are required')

    def documents(self, epoch='25'):
        return {name: json.loads(raw) for name, raw in self.raw[epoch].items()}

    def compare(self, docs=None, epoch='25', capture=None, constructor=None):
        blobs = dict(self.raw[epoch])
        if docs is not None:
            for name in blobs:
                if docs[name] != json.loads(blobs[name]):
                    blobs[name] = json.dumps(docs[name]).encode()
        return self.subject.compare(*(blobs[name] for name in ['corpus', 'native', 'rust', 'receipt']),
                                    constructor or self.constructor, capture or self.capture)

    def cases(self, docs, cid='05-quoted-slashes'):
        return [next(case for case in docs[name]['cases'] if case['id'] == cid)
                for name in ['native', 'rust']]

    def values(self, record):
        return {row['key']: row['value'] for row in record['properties'] if 'value' in row}

    def property(self, record, key):
        return next(row for row in record['properties'] if row['key'] == key)

    def drop(self, record, key):
        self.assertIn(key, record['property_keys'])
        record['properties'] = [row for row in record['properties'] if row['key'] != key]
        record['property_keys'].remove(key)

    def references(self, native):
        objects = {obj['id']: obj for obj in native['tree']['objects']}
        owner = next(obj for obj in objects.values() if obj.get('node_type') == 'TypeReference')
        array = objects[self.values(owner)['parts']['$object']]
        values = self.values(array)
        parts = [objects[values[str(index)]['$object']] for index in range(values['length'])]
        return owner, array, parts

    def rust_parts(self, rust):
        node = next(node for node in rust['outcome']['nodes'] if node['node_type'] == 'TypeReference')
        return node['fields']['parts']['value']['items']

    def result_row(self, report, cid='05-quoted-slashes'):
        return next(row for row in report['cases'] if row['id'] == cid)

    def assert_reference_difference(self, row, suffix):
        self.assertEqual(row['status'], 'DIVERGED')
        self.assertIn('references', row['differences'])
        self.assertTrue(any(path.startswith('references/') and suffix in path
                            for path in row['difference_paths']))
        self.assertTrue(row['open_items'])

    def test_complete_epochs_preserve_old_red_and_current_selected_outcomes(self):
        old, current = self.compare(epoch='15'), self.compare()
        self.assertEqual((old['difference_count'], old['divergent_cases']), (2, 2))
        changed = [row for row in old['cases'] if row['differences']]
        self.assertEqual({row['id'] for row in changed}, {'04-comment-only-EOF', 'comment-only-root'})
        self.assertTrue(all(row['differences'] == ['root_origin'] for row in changed))
        self.assertEqual(len(current['cases']), 25)
        self.assertEqual(current['difference_count'], 0)
        self.assertFalse(current['full_parity_claimed'])
        json.dumps(current, allow_nan=False)

    def test_source_paths_hashes_receipt_limit_and_diagnostics_are_retained(self):
        result = self.compare()
        self.assertEqual(result['capture_module'], {
            'path': str(self.capture),
            'sha256': '57c141ae620d103b1fe6104286cb887a62ec8dcccaf45ed34fc6baf64985a88c'})
        self.assertEqual(result['constructor_comparator']['path'], str(self.constructor))
        self.assertEqual(result['constructor_comparator']['sha256'],
                         '3d7d2451ff2300510cda71ad6c5debee6fab47193277d8c1292f2490eb7874d6')
        self.assertIn('no output digest', result['qualification_binding']['limit'])
        for name, raw in self.raw['25'].items():
            self.assertEqual(result['input_sha256'][name], hashlib.sha256(raw).hexdigest())
        docs = self.documents()
        for native in docs['native']['cases']:
            row = self.result_row(result, native['id'])
            self.assertEqual(row['native_lexer_errors'], native['lexer_errors'])
            self.assertEqual(row['native_parser_errors'], native['parser_errors'])

    def test_six_exact_restrictions_and_coordinate_fault_remain_open(self):
        result = self.compare()
        cases = ['07-semantic-block-comment', '08-overlap-multiline-note', 'late-overlap',
                 '09-commented-closing-brace', '10-no-token-merging', 'quoted-error-offset']
        for cid in cases:
            row = self.result_row(result, cid)
            self.assertEqual(row['status'], 'OPEN')
            self.assertIn('open_reason', row)
            self.assertFalse(row['differences'])
        fault = self.result_row(result, 'quoted-error-offset')
        self.assertTrue(any('17, 18, 32' in item for item in fault['open_items']))

    def test_missing_origin_does_not_hide_known_absent_namespace_name(self):
        docs = self.documents()
        native, rust = self.cases(docs, '01-LF')
        self.drop(self.references(native)[2][0], '$refNode')
        before = self.result_row(self.compare(docs), '01-LF')
        self.assertEqual(before['status'], 'OPEN')
        self.assertIsNone(before['native_projection']['selected'][0]['name_units'])
        root = next(node for node in rust['outcome']['nodes'] if node['id'] == rust['outcome']['root'])
        root['fields']['declaredName'] = {'lexeme': None, 'value': {'kind': 'string', 'units': [88]}}
        row = self.result_row(self.compare(docs), '01-LF')
        self.assertEqual(row['status'], 'DIVERGED')
        self.assertIn('selected/0/name_units', row['difference_paths'])
        self.assertTrue(row['open_items'])

    def test_missing_first_origin_retains_all_available_parts(self):
        docs = self.documents()
        native, _ = self.cases(docs)
        self.drop(self.references(native)[2][0], '$refNode')
        row = self.result_row(self.compare(docs))
        self.assertEqual(row['status'], 'OPEN')
        self.assertFalse(row['differences'])
        parts = row['native_projection']['references'][0]['parts']
        self.assertEqual(len(parts), 2)
        self.assertIn('unavailable', parts[0]['origin'])
        self.assertIsInstance(parts[0]['text_units'], list)
        self.assertIsInstance(parts[1]['text_units'], list)
        self.assertIn('utf16', parts[1]['origin'])

    def test_missing_first_origin_still_compares_same_and_later_text(self):
        for index in [0, 1]:
            with self.subTest(part=index):
                docs = self.documents()
                native, rust = self.cases(docs)
                self.drop(self.references(native)[2][0], '$refNode')
                self.rust_parts(rust)[index]['value']['text'] += '_changed'
                self.assert_reference_difference(self.result_row(self.compare(docs)),
                                                 '/parts/' + str(index) + '/text_units')

    def test_missing_first_text_preserves_its_origin_and_later_origin(self):
        docs = self.documents()
        native, rust = self.cases(docs)
        self.drop(self.references(native)[2][0], '$refText')
        row = self.result_row(self.compare(docs))
        self.assertEqual(row['status'], 'OPEN')
        self.assertFalse(row['differences'])
        part = row['native_projection']['references'][0]['parts'][0]
        self.assertIn('unavailable', part['text_units'])
        self.assertIn('utf16', part['origin'])
        self.rust_parts(rust)[1]['lexeme'] = {'bytes': [0, 0], 'utf16': [0, 0], 'text': ''}
        self.assert_reference_difference(self.result_row(self.compare(docs)), '/parts/1/origin')

    def test_missing_typeref_origin_preserves_parts_and_available_contradiction(self):
        docs = self.documents()
        native, rust = self.cases(docs)
        self.drop(self.references(native)[0], '$cstNode')
        row = self.result_row(self.compare(docs))
        self.assertEqual(row['status'], 'OPEN')
        self.assertFalse(row['differences'])
        reference = row['native_projection']['references'][0]
        self.assertIn('unavailable', reference['origin'])
        self.assertEqual(len(reference['parts']), 2)
        self.rust_parts(rust)[1]['value']['text'] += '_changed'
        self.assert_reference_difference(self.result_row(self.compare(docs)), '/parts/1/text_units')

    def test_unavailable_null_and_empty_parts_remain_distinct(self):
        for kind in ['missing', 'null', 'empty']:
            with self.subTest(kind=kind):
                docs = self.documents()
                native, _ = self.cases(docs)
                owner, array, _ = self.references(native)
                if kind == 'missing':
                    self.drop(owner, 'parts')
                elif kind == 'null':
                    self.property(owner, 'parts')['value'] = None
                else:
                    array['properties'] = [self.property(array, 'length')]
                    array['properties'][0]['value'] = 0
                    array['property_keys'] = ['length']
                row = self.result_row(self.compare(docs))
                parts = row['native_projection']['references'][0]['parts']
                if kind == 'missing':
                    self.assertEqual(row['status'], 'OPEN')
                    self.assertFalse(row['differences'])
                    self.assertIn('unavailable', parts)
                else:
                    self.assertEqual(row['status'], 'DIVERGED')
                    self.assertIn('references', row['differences'])
                    self.assertEqual(parts, None if kind == 'null' else [])

    def test_unevaluated_reference_accessor_does_not_hide_available_text(self):
        docs = self.documents()
        native, rust = self.cases(docs)
        prop = self.property(self.references(native)[2][0], '$refNode')
        prop.pop('value')
        prop['descriptor'] = {'enumerable': True, 'configurable': True,
                              'accessor': True, 'getter': True, 'setter': False}
        self.rust_parts(rust)[0]['value']['text'] += '_changed'
        self.assert_reference_difference(self.result_row(self.compare(docs)), '/parts/0/text_units')

    def test_known_null_reference_text_is_compared_instead_of_skipped(self):
        docs = self.documents()
        native, _ = self.cases(docs)
        self.property(self.references(native)[2][0], '$refText')['value'] = None
        row = self.result_row(self.compare(docs))
        self.assertEqual(row['status'], 'DIVERGED')
        self.assertIn('references/0/parts/0/text_units', row['difference_paths'])

    def test_dropped_nonselected_type_is_still_visible_in_full_census(self):
        docs = self.documents()
        _, rust = self.cases(docs)
        reference = next(node for node in rust['outcome']['nodes'] if node['node_type'] == 'TypeReference')
        owner = next(node for node in rust['outcome']['nodes'] if node['id'] == reference['container']['node'])
        owner['fields'].pop('targetRef')
        rust['outcome']['nodes'].remove(reference)
        row = self.result_row(self.compare(docs))
        self.assertEqual(row['status'], 'DIVERGED')
        self.assertIn('ast_type_census', row['differences'])

    def test_finite_ownership_is_checked_after_real_schema_admission(self):
        docs = self.documents()
        _, rust = self.cases(docs)
        node = next(node for node in rust['outcome']['nodes'] if node['node_type'] == 'Package')
        node['container']['property'] = 'wrong'
        with self.assertRaisesRegex(ValueError, 'ownership/containment'):
            self.compare(docs)

    def test_new_root_range_does_not_corrupt_shared_native_positions(self):
        docs = self.documents()
        native, _ = self.cases(docs, '01-LF')
        objects = {obj['id']: obj for obj in native['tree']['objects']}
        root = objects[native['tree']['root']['$object']]
        cst_id = self.values(root)['$cstNode']['$cst']
        cst = next(node for node in native['tree']['cst'] if node['id'] == cst_id)
        old_objects = copy.deepcopy(native['tree']['objects'])
        old_cst = copy.deepcopy(native['tree']['cst'])
        span = cst['span']
        copied_range = copy.deepcopy(objects[span['range']['$object']])
        copied_range['id'] = 'test-root-range'
        additions = [copied_range]
        for endpoint in ['start', 'end']:
            position = copy.deepcopy(objects[self.values(copied_range)[endpoint]['$object']])
            position['id'] = 'test-root-' + endpoint
            for field in position['properties']:
                field['value'] = 0
            additions.append(position)
            self.property(copied_range, endpoint)['value'] = {'$object': position['id']}
        self.assertFalse(set(objects) & {obj['id'] for obj in additions})
        native['tree']['objects'].extend(additions)
        span.update(offset=0, end=0, length=0, text='', range={'$object': copied_range['id']})
        self.assertEqual(native['tree']['objects'][:len(old_objects)], old_objects)
        self.assertEqual([node for node in native['tree']['cst'] if node['id'] != cst_id],
                         [node for node in old_cst if node['id'] != cst_id])
        row = self.result_row(self.compare(docs), '01-LF')
        self.assertEqual(row['status'], 'DIVERGED')
        self.assertIn('root_origin', row['differences'])

    def test_unexpected_comment_interior_refusal_is_a_difference(self):
        docs = self.documents()
        _, rust = self.cases(docs, '04-comment-only-EOF')
        offset = rust['source'].encode().index(b'/*')
        rust['outcome'] = {'kind': 'rust_error', 'error': 'Unsupported { offset: ' + str(offset) + ' }'}
        row = self.result_row(self.compare(docs), '04-comment-only-EOF')
        self.assertEqual(row['status'], 'DIVERGED')
        self.assertIn('unexpected_Rust_refusal', row['differences'])

    def test_recovered_input_requires_exact_complete_error_payload(self):
        for error in ['Malformed { offset: 21', 'Malformed { offset: 999999, expected: "}" }',
                      'Malformed { offset: 4, expected: "}" }']:
            with self.subTest(error=error):
                docs = self.documents()
                _, rust = self.cases(docs, '09-commented-closing-brace')
                rust['outcome']['error'] = error
                if 'offset: 4,' in error:
                    row = self.result_row(self.compare(docs), '09-commented-closing-brace')
                    self.assertEqual(row['status'], 'DIVERGED')
                else:
                    with self.assertRaises(ValueError):
                        self.compare(docs)

    def test_capture_source_is_rechecked_each_call_before_code_execution(self):
        with tempfile.TemporaryDirectory(dir=self.temporary_root) as directory:
            path = Path(directory) / 'capture.py'
            sentinel = Path(directory) / 'executed'
            path.write_bytes(self.capture.read_bytes())
            self.assertEqual(self.compare(capture=path)['difference_count'], 0)
            path.write_text('from pathlib import Path\nPath(' + repr(str(sentinel)) + ').touch()\n')
            with self.assertRaisesRegex(ValueError, 'capture admission source hash mismatch'):
                self.compare(capture=path)
            self.assertFalse(sentinel.exists())

    def test_constructor_source_is_rechecked_through_actual_admission(self):
        with tempfile.TemporaryDirectory(dir=self.temporary_root) as directory:
            path = Path(directory) / 'constructor.py'
            sentinel = Path(directory) / 'executed'
            path.write_bytes(self.constructor.read_bytes())
            self.assertEqual(self.compare(constructor=path)['difference_count'], 0)
            path.write_text('from pathlib import Path\nPath(' + repr(str(sentinel)) + ').touch()\n')
            with self.assertRaisesRegex(ValueError, 'constructor comparator source hash mismatch'):
                self.compare(constructor=path)
            self.assertFalse(sentinel.exists())

    def test_each_call_readmits_schema_receipt_and_exact_corpus_bytes(self):
        self.assertEqual(self.compare()['difference_count'], 0)
        for change in ['origin', 'receipt', 'corpus']:
            with self.subTest(change=change):
                docs = self.documents()
                if change == 'origin':
                    self.cases(docs)[1]['outcome']['nodes'][0].pop('origin')
                elif change == 'receipt':
                    docs['receipt']['source_unchanged'] = False
                else:
                    docs['corpus']['cases'].reverse()
                with self.assertRaises(ValueError):
                    self.compare(docs)

    def test_returned_report_cannot_replace_fresh_native_expectations(self):
        report = self.compare()
        report['cases'].clear()
        report['native_provenance'].clear()
        current = self.compare()
        self.assertEqual(len(current['cases']), 25)
        self.assertEqual(current['difference_count'], 0)
        self.assertTrue(current['native_provenance'])


if __name__ == '__main__':
    unittest.main()
