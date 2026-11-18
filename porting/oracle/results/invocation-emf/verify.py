"""Verify the retained released SysON converter evidence."""
from base64 import b64decode
from copy import deepcopy
from hashlib import sha256
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
OBSERVATION_SHA = "5cc9cb070cc32a45ed4a5a3bc89e83576106013f4e43a412a396a975df7430a6"
PARSER_OBSERVATION_SHA = "52ddd54c8e14dbc6ff11fb4bf9c9921682b8beb1113faf03faa692728b1cfad4"
SCOPE_SHA = "e42359a84a07e85b1437feb56c4f26a3c1f9f9352e167c7412ce8bc998c548ba"
TERMINAL_SHA = "3ec4ed1695144e238f78dfab0ba7e696e9aab2557a867948ccbc0b87f27b955e"
MANIFEST_SHA = "f52d000613d33d3dd20a1c406fa24a0b94bd72afd1059d6eeaae712bf53db77c"
EMPTY_SHA = sha256(b"").hexdigest()
CASES = {
    "part-call": ("initial", "PartDefinition", "A", "part-definition", False),
    "abstract-part-call": ("followup", "PartDefinition", "A", "part-definition", True),
    "typed-calculation-call": ("followup", "CalculationDefinition", "F", "calculation-definition", None),
}
ARTIFACTS = {"converted.tsv", "reloaded.tsv", "messages.tsv", "model.emf.json"}


def require(condition, detail):
    if not condition:
        raise ValueError(detail)


def hash_text(text):
    return sha256(text.encode("utf-8")).hexdigest()


def read_pinned(path, expected_sha):
    raw = path.read_bytes()
    require(sha256(raw).hexdigest() == expected_sha, f"changed capture: {path.name}")
    return json.loads(raw)


def value(rows, label):
    entries = [row[1:] for row in rows if row[0] == label]
    require(len(entries) == 1, f"expected one {label}")
    return entries[0]


def field(raw):
    return None if raw == "-" else b64decode(raw, validate=True).decode("utf-8")


def match_in_model(row, element):
    kind, declared_name, proxy, resource, position, fragment, element_id, proxy_uri = row
    require((f"sysml:{kind}", field(declared_name)) == (element["eClass"], element["data"].get("declaredName")),
            "projection metaclass or name differs from serialized JSON")
    require((proxy, resource, proxy_uri) == ("false", "model", "-") and int(position) >= 0, "object identity role")
    require((field(fragment), field(element_id)) == (element["id"], element["data"]["elementId"]),
            "projected URI fragment or semantic ID differs from serialized JSON")


def only_child(element, feature, kind):
    rows = [child for child in element["data"].get(feature, []) if child["eClass"] == f"sysml:{kind}"]
    require(len(rows) == 1, f"expected one owned {kind}")
    return rows[0]


def owned_definition(package, kind, name):
    owned = [child for membership in package["data"]["ownedRelationship"]
             for child in membership["data"].get("ownedRelatedElement", [])]
    found = [child for child in owned
             if (child["eClass"], child["data"].get("declaredName")) == (f"sysml:{kind}", name)]
    require(len(found) == 1, f"expected unique package element {kind} {name}")
    return found[0]


def check_typing(typing, target, feature):
    require((typing["data"].get("type"), typing["data"].get("typedFeature"))
            == (target["id"], feature["id"]), "wrong saved FeatureTyping endpoints")


def check_saved_and_projection(files, case):
    _, kind, name, target_label, is_abstract = CASES[case]
    raw = json.loads(files["model.emf.json"]["data"])
    (root,) = raw["content"]
    require(root["eClass"] == "sysml:Namespace", "expected converter namespace")
    member = only_child(root, "ownedRelationship", "OwningMembership")
    package = only_child(member, "ownedRelatedElement", "Package")
    require(package["data"]["declaredName"] == "P", "fixture package")
    target = owned_definition(package, kind, name)
    usage_kind = "AttributeUsage" if is_abstract is None else "PartUsage"
    usage = owned_definition(package, usage_kind, "x")
    owner = only_child(usage, "ownedRelationship", "FeatureValue")
    invocation = only_child(owner, "ownedRelatedElement", "InvocationExpression")
    typing = only_child(invocation, "ownedRelationship", "FeatureTyping")
    require(invocation["data"]["ownedRelationship"] == [typing], "unexpected invocation relationship")
    check_typing(typing, target, invocation)

    require(files["converted.tsv"] == files["reloaded.tsv"], "readback projection differs")
    rows = [line.split("\t") for line in files["converted.tsv"]["data"].splitlines()]
    for label, count in (("roots", "1"), ("errors", "0"), ("warnings", "0"), ("invocations", "1")):
        require(value(rows, label) == [count], f"unexpected {label}")
    require(not {row[0] for row in rows} & {"membership-member", "result-type", "calculation-result-type"},
            "unexpected invocation membership or derived result type")
    for label in ("instantiated-operation", "instantiated-getter", "function-getter", "result-getter"):
        require(value(rows, label) == ["null"], f"{label} changed")
    for label, element in (("root", root), ("owner", usage), ("container", owner), ("invocation", invocation),
                           ("raw-typing", typing), (target_label, target)):
        match_in_model(value(rows, label), element)
    raw_type = value(rows, "raw-type")
    for label in ("typing-type", "feature-type"):
        require(value(rows, label) == raw_type, f"{label} does not identify the declared target")
    require(raw_type == value(rows, target_label), "raw type has different phase-local identity")
    for label in ("raw-typed-feature", "typed-feature"):
        require(value(rows, label) == value(rows, "invocation"), f"{label} does not identify the invocation")
    require(value(rows, "relationship") == value(rows, "raw-typing"), "relationship identity changed")

    if is_abstract is None:
        require(value(rows, "calculation-result") == ["null"], "calculation result getter changed")
        out = only_child(target, "ownedRelationship", "ReturnParameterMembership")
        parameter = only_child(out, "ownedRelatedElement", "ReferenceUsage")
        require((parameter["data"]["declaredName"], parameter["data"]["direction"]) == ("r", "out"),
                "calculation return parameter changed")
        parameter_type = owned_definition(package, "AttributeDefinition", "N")
        match_in_model(value(rows, "attribute-definition"), parameter_type)
        check_typing(only_child(parameter, "ownedRelationship", "FeatureTyping"), parameter_type, parameter)
    else:
        require(target["data"].get("isAbstract", False) is is_abstract
                and value(rows, "is-abstract") == [str(is_abstract).lower()], "abstract definition control changed")


def check(record, parser):
    provenance = record["provenance"]
    require((provenance["guarded_scope_sha256"], provenance["terminal_receipt_sha256"],
             provenance["post_run_manifest_sha256"]) == (SCOPE_SHA, TERMINAL_SHA, MANIFEST_SHA), "custody pins")
    require(provenance["parser_pr_head"] == "6194295e8c2edd34c1d9f90954b6a10d49db4c55", "parser PR head")
    require(provenance["release"] == "v2026.9.3"
            and provenance["release_commit"] == parser["context"]["release_source"]
            and provenance["classpath_member_count"] == 312, "release provenance")
    run = provenance["process"]
    require(run["blocked"] is False and run["frozen_inputs_unchanged"] is True and run["failure"] is None,
            "run failed or input drift")
    require([step["step"] for step in run["records"]] == ["kernel", "compile", "converter"], "step order")
    for step in run["records"]:
        require(step["exit"] == 0 and step["guard"] is None and step["stderr_sha256"] == EMPTY_SHA, "run step failed")
    kernel_stdout = provenance["kernel_stdout"]
    require(kernel_stdout == "PASS socket(AF_INET, AF_INET6, AF_UNIX) denied by kernel\n"
            and run["records"][0]["stdout_sha256"] == hash_text(kernel_stdout), "socket deny proof")

    require(set(record["cases"]) == set(CASES), "unexpected converter fixture set")
    for name, case in record["cases"].items():
        prior = parser["captures"][CASES[name][0]]
        row = prior["observations"][name]
        require(row["bare"]["exit"] == row["validate"]["exit"] == 0 and row["validate"]["stderr"] == "",
                "independent CLI validation must be clean")
        require(case["inputs"] == {"source_sha256": row["source_sha256"], "parser_ast_sha256": row["bare"]["ast"]["sha256"],
                                   "parser_terminal_sha256": prior["receipt_sha256"]}, "parser custody mismatch")
        files = case["files"]
        require(set(files) == ARTIFACTS and files["messages.tsv"]["data"] == "", "missing output or converter message")
        for file in files.values():
            require(hash_text(file["data"]) == file["sha256"], "raw artifact hash mismatch")
        check_saved_and_projection(files, name)


def reject_mutations(record, parser):
    edits = (
        ("part-call", "model.emf.json", '"type":', '"differentType":'),
        ("abstract-part-call", "model.emf.json", '"isAbstract":true', '"isAbstract":false'),
        ("typed-calculation-call", "model.emf.json", '"direction":"out"', '"direction":"in"'),
        ("part-call", "reloaded.tsv", "invocations\t1", "invocations\t0"),
        ("part-call", "both projections", "result-getter\tnull", "result-getter\tInstance"),
        ("typed-calculation-call", "both projections", "calculation-result\tnull", "calculation-result\tInstance"),
    )
    for case, filename, old, new in edits:
        altered = deepcopy(record)
        files = altered["cases"][case]["files"]
        filenames = ("converted.tsv", "reloaded.tsv") if filename == "both projections" else (filename,)
        for name in filenames:
            text = files[name]["data"]
            require(old in text, f"mutation input missing: {case}/{name}")
            mutated = text.replace(old, new, 1)
            files[name] = {"data": mutated, "sha256": hash_text(mutated)}
        try:
            check(altered, parser)
        except ValueError:
            continue
        raise ValueError(f"accepted mutation: {case}/{filename}/{old}")
    return len(edits)


def main():
    record = read_pinned(HERE / "observation.json", OBSERVATION_SHA)
    parser = read_pinned(HERE.parent / "constructor-invocation/observation.json", PARSER_OBSERVATION_SHA)
    check(record, parser)
    print(f"Verified 3 released converter fixtures and rejected {reject_mutations(record, parser)} mutations.")


if __name__ == "__main__":
    main()
