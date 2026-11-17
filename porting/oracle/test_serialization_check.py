# SPDX-License-Identifier: EPL-2.0

import copy
import unittest
from unittest.mock import patch

import serialization_check as check


def node(kind, object_id, **data):
    return {"eClass": f"sysml:{kind}", "id": object_id,
            "data": {"elementId": f"semantic-{object_id}", **data}}


def model(use_aliases=False):
    definition = node("PartDefinition", "definition", declaredName="Wheel")
    typing = node("FeatureTyping", "typing",
                  type="semantic-definition" if use_aliases else "definition",
                  typedFeature="semantic-usage" if use_aliases else "usage")
    usage = node("PartUsage", "usage", declaredName="wheel", isComposite=True,
                 ownedRelationship=[typing])
    return {"content": [node("Package", "package", declaredName="Vehicle", ownedRelationship=[
        node("OwningMembership", "member-a", ownedRelatedElement=[definition]),
        node("OwningMembership", "member-b", ownedRelatedElement=[usage]),
    ])]}


def typing_data(document):
    return next(n["data"] for _, n in check.objects(document["content"]) if n["eClass"] == check.TYPING)


class SerializationChecks(unittest.TestCase):
    def test_control_rewrites_only_the_named_reference_roles(self):
        source = model(use_aliases=True)
        expected = copy.deepcopy(source)
        typing_data(expected).update(type="definition", typedFeature="usage")
        fixed = check.id_control(source)
        self.assertEqual(fixed, expected)
        self.assertEqual(typing_data(source)["type"], "semantic-definition")
        check.check_readback(source, fixed)

    def test_lost_wrong_and_semantic_only_links_are_refused(self):
        before = model()
        for value, reason in ((None, "no resolved"), ("usage", "different object"),
                              ("semantic-definition", "no resolved"), ("unresolved#id", "no resolved")):
            with self.subTest(value=value):
                after = copy.deepcopy(before)
                typing_data(after)["type"] = value
                with self.assertRaisesRegex(check.ModelMismatch, reason):
                    check.check_readback(before, after)

    def test_content_shape_loss_is_a_separate_failure(self):
        before = model()
        after = copy.deepcopy(before)
        after["content"][0]["data"]["declaredName"] = "Changed"
        with self.assertRaisesRegex(check.ModelMismatch, "declaredName: value changed"):
            check.check_readback(before, after)
        after["content"] = []
        with self.assertRaisesRegex(check.ModelMismatch, "expected sysml:Package"):
            check.check_readback(before, after)

    def test_changed_semantic_id_is_refused_even_when_shape_matches(self):
        before = model()
        after = copy.deepcopy(before)
        after["content"][0]["data"]["elementId"] = "changed-semantic-id"
        with self.assertRaisesRegex(check.ModelMismatch, "semantic identity changed"):
            check.check_readback(before, after)

    def test_skipped_unsupported_and_infrastructure_are_inconclusive(self):
        with patch.object(check, "upload_and_download", side_effect=OSError("unavailable")) as upload:
            self.assertEqual(check.run_case("unused", "refused", None)["opens_in_syson"], "n/a")
            foreign = {"content": [node("UnknownDefinition", "unknown")]}
            self.assertEqual(check.run_case("unused", "unsupported", foreign)["opens_in_syson"], "n/a")
            upload.assert_not_called()
            result = check.run_case("unused", "admitted", model())
        self.assertEqual(result["opens_in_syson"], "n/a")
        self.assertIn("reference harness error", result["reason"])

    def test_new_reference_protocol_exception_remains_inconclusive(self):
        class ProtocolFailure(Exception):
            pass

        with patch.object(check.reference, "ReferenceProtocolError", ProtocolFailure, create=True):
            with patch.object(check, "upload_and_download", side_effect=ProtocolFailure("server failed")):
                result = check.run_case("unused", "admitted", model())
        self.assertEqual(result["opens_in_syson"], "n/a")
        self.assertIn("reference harness error", result["reason"])
