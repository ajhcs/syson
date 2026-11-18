"""Verify the released abstract PartUsage typing evidence offline."""
from hashlib import sha256
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
SEAL = "1e81e8ab942e4d228ae57cfa861309de026b8f8cdfd230bdbffa54bd092ed3f0"
WRONG_REPORT = ("There are validation errors:\n"
                "line 1: PartUsage must be typed by Structures only [part x : A;]\n")
FIXTURES = {
    "abstract": ("abstract part def", "PartDefinition", "abstract"),
    "concrete": ("part def", "PartDefinition", None),
    "wrong-metaclass": ("attribute def", "AttributeDefinition", None),
}
EVIDENCE_PINS = {
    "converter_harness_sha256": "2ee830dacc380ca06e7e4b374dbf2e0a8e5ff9b7395c8c7eccb38f0450b46268",
    "converter_receipt_sha256": "c2f0aca7f5ad161c8809ae5c88f8348d27f6d510dd2d9b6ee2eebb1b6de7408a",
    "node_projection_sha256": "23e49ccfd5cebeb44d5b86ee384ef5d5d2808d3e53b885d1b1276a8ce420335c",
    "node_receipt_sha256": "7b4fc26d4faf1c24e23b2886a3ff7c0946b5fc6dc30f85bb457e51077dd873f7",
    "rust_probe_sha256": "27982845e1f07c5a346ad4960b532f2495bf322a1ca940fdc89c1587914c9ba7",
}


def require(condition, detail):
    if not condition:
        raise ValueError(detail)


def node(kind, **data):
    return {"eClass": "sysml:" + kind, "data": data}


def package_shape(target_class, marker):
    definition = node(target_class, declaredName="A")
    if marker is not None:
        definition["data"]["isAbstract"] = True
    link = node("FeatureTyping", type="P::A", typedFeature="P::x")
    usage = node("PartUsage", declaredName="x", isComposite=True, ownedRelationship=[link])
    return node("Package", declaredName="P", ownedRelationship=[
        node("OwningMembership", ownedRelatedElement=[definition]),
        node("OwningMembership", ownedRelatedElement=[usage]),
    ])


def check(data):
    require(data["schema"] == "SYSON-RELEASED-ABSTRACT-PART-TYPING/1", "wrong schema")
    require(data["scope"] == dict.fromkeys(
        ("aql", "http", "database", "constructor", "default_importer_changed", "network"), False
    ) and all(value is False for value in data["scope"].values()), "scope changed")
    require(data["release"] == {
        "jar_sha256": "85c031352695d6ac8f059f0215ca493c58a16805a111233998dd443d6990c9e1",
        "source_label": "b380dada415ce05155150f23cb3b74ae21cc45e4",
    }, "release changed")
    require(data["evidence"] == EVIDENCE_PINS, "runtime evidence binding changed")
    require(set(data["cases"]) == set(FIXTURES), "case inventory changed")
    for name, (keyword, target_class, marker) in FIXTURES.items():
        row = data["cases"][name]
        expected_source = "package P { " + keyword + " A; part x : A; }"
        require(row["fixture"] == expected_source
                and row["fixture_sha256"] == sha256(expected_source.encode()).hexdigest(), "fixture changed")
        wrong = name == "wrong-metaclass"
        ast = row["released_bare_ast"]
        require((ast["typed_feature"], ast["target"], ast["target_class"], ast["raw_marker"]) ==
                ("P::x", "P::A", target_class, marker), "released bare AST link changed")
        cli = row["released_stdlib_none_validation"]
        require(type(cli["exit"]) is int and cli ==
                ({"exit": 1, "stderr": WRONG_REPORT} if wrong else {"exit": 0, "stderr": ""}),
                "independent CLI outcome changed")
        converter = row["native_converter"]
        require(converter["package_projection"] == package_shape(target_class, marker)
                and converter["resource_nodes"] == 8 and converter["package_nodes"] == 6
                and converter["transformation_messages"] == 0, "converter ownership or reference changed")
        require(converter["derived_part_definitions"] == ([] if wrong else ["P::A"]),
                "derived definitions changed")
        rust = row["rust_stage"]
        require(rust["identities_requested"] == 6, "Rust allocation changed")
        if wrong:
            require(rust["status"] == "error" and rust["expected_type"] == "PartDefinition"
                    and rust["document"] is False, "Rust wrong-type refusal changed")
        else:
            require(rust["status"] == "ok" and rust["document"] is True
                    and (rust["typed_feature"], rust["target"]) == ("P::x", "P::A")
                    and rust["is_abstract_field"] is (True if marker is not None else None),
                    "Rust positive link changed")


def main():
    try:
        content = (HERE / "observation.json").read_bytes()
        require(sha256(content).hexdigest() == SEAL, "observation bytes changed")
        check(json.loads(content))
    except (OSError, ValueError, KeyError, TypeError) as failure:
        print(f"FAIL: {failure}", file=sys.stderr)
        return 1
    print("PASS: abstract and concrete typing, wrong-metaclass conversion, CLI and Rust refusals")
    return 0


if __name__ == "__main__":
    sys.exit(main())
