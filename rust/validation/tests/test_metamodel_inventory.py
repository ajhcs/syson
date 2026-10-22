# SPDX-License-Identifier: EPL-2.0
import importlib.util
from pathlib import Path
import unittest
import xml.etree.ElementTree as ET

SCRIPT = Path(__file__).resolve().parents[1] / 'scripts/check_metamodel_inventory.py'
spec = importlib.util.spec_from_file_location('metamodel_gate', SCRIPT)
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)


class NestedDeclarationTests(unittest.TestCase):
    def setUp(self):
        self.xml = ET.fromstring('<eClassifiers name="Example"><eOperations name="operation"><eParameters name="p"><eAnnotations source="a"/></eParameters></eOperations><eLiterals name="literal"/></eClassifiers>')
        def node(tag, attributes, location, children):
            return {'tag': tag, 'attributes': attributes, 'source_location': location,
                    'text': None, 'tail': None, 'children': children}
        root = 'model.ecore#//@eClassifiers.0'
        self.record = node('eClassifiers', {'name': 'Example'}, root, [
            node('eOperations', {'name': 'operation'}, root+'/@eOperations.0', [
                node('eParameters', {'name': 'p'}, root+'/@eOperations.0/@eParameters.0', [
                    node('eAnnotations', {'source': 'a'}, root+'/@eOperations.0/@eParameters.0/@eAnnotations.0', [])])]),
            node('eLiterals', {'name': 'literal'}, root+'/@eLiterals.0', [])])

    def check(self):
        gate.compare(self.xml, self.record, 'model.ecore#//@eClassifiers.0')

    def test_emf_fragment_containment_locations(self):
        self.check()

    def test_missing_nested_parameter_rejected(self):
        self.record['children'][0]['children'] = []
        with self.assertRaises(AssertionError): self.check()

    def test_missing_nested_annotation_rejected(self):
        self.record['children'][0]['children'][0]['children'] = []
        with self.assertRaises(AssertionError): self.check()

    def test_missing_enum_literal_rejected(self):
        self.record['children'].pop()
        with self.assertRaises(AssertionError): self.check()

    def test_non_emf_fragment_rejected(self):
        self.record['children'][0]['source_location'] = 'model.ecore#//@eClassifiers.0/eOperations.0'
        with self.assertRaises(AssertionError): self.check()

    def test_attribute_change_rejected(self):
        self.record['children'][0]['children'][0]['attributes']['name'] = 'changed'
        with self.assertRaises(AssertionError): self.check()


if __name__ == '__main__':
    unittest.main()
