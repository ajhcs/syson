"""Verify stored outcomes for the SysON constructor and invocation fixtures."""

import copy
from hashlib import sha256
import json
from pathlib import Path
import re


OBSERVATION = Path(__file__).with_name("observation.json")
OBSERVATION_SHA = "52ddd54c8e14dbc6ff11fb4bf9c9921682b8beb1113faf03faa692728b1cfad4"
CLI = "47d8fe11fda1b58b6257463087f663be953c2ab6c2b5e1e2ccc44161ed5cd0bd"
BASELINE = "6ba2ac64e6b16ee2b05752ad55395eebb87d4910"
RELEASE = "b380dada415ce05155150f23cb3b74ae21cc45e4"
SHA_PATTERN = re.compile(r"[0-9a-f]{64}")
HEADER = "There are validation errors:"
TYPING_DIAGNOSTIC = "A Feature must be typed by at least one type."
CASES = {
    "initial": {"abstract-new", "concrete-new", "part-call", "calculation-call"},
    "followup": {"abstract-part-call", "typed-calculation-call"},
}


def require(condition, detail):
    if not condition:
        raise ValueError(detail)


def digest(text):
    return sha256(text.encode()).hexdigest()


def declaration(ast, metaclass, name):
    matching = [entry for entry in ast["declarations"] if (entry["class"], entry["name"]) == (metaclass, name)]
    require(len(matching) == 1, f"expected one {metaclass} {name}")
    return matching[0]


def check_fixture(name, row):
    require(digest(row["source"]) == row["source_sha256"], "fixture hash")
    bare, validated = row["bare"], row["validate"]
    for capture in (bare, validated):
        require(SHA_PATTERN.fullmatch(capture["stdout_sha256"]) and
                digest(capture["stderr"]) == capture["stderr_sha256"], "channel hashes")
    require(bare["exit"] == 0 and bare["stderr"] == "" and SHA_PATTERN.fullmatch(bare["ast"]["sha256"]), "bare AST")
    ast = bare["ast"]
    recovery = name.endswith("-new")
    untyped = name == "calculation-call"
    if name.endswith("calculation-call"):
        declaration(ast, "CalculationDefinition", "F")
        usage = declaration(ast, "AttributeUsage", "x")
        kind, cst, reference, text, target = "InvocationExpression", "F()", "TypeReference", "F", "P::F"
        if not untyped:
            declaration(ast, "AttributeDefinition", "N")
            require("return r : N;" in row["source"] and usage["cst"] == "attribute x : N = F();", "local type control")
    else:
        marker = "abstract" if name.startswith("abstract-") else None
        require(declaration(ast, "PartDefinition", "A")["abstract_marker"] == marker, "abstract definition marker")
        value = "new" if recovery else "A();"
        require(declaration(ast, "PartUsage", "x")["cst"] == "part x : A = " + value, "part value extent")
        if recovery:
            kind, cst, reference, text, target = "FeatureReferenceExpression", "new", "FeatureReference", "new", None
        else:
            kind, cst, reference, text, target = "InvocationExpression", "A()", "TypeReference", "A", "P::A"
    expected = [{"class": kind, "cst": cst, "references": [{"class": reference, "text": text, "target": target}]}]
    if untyped:
        expected.insert(0, {"class": "LiteralNumber", "cst": "1", "references": []})
    require(ast["expressions"] == expected, "expression projection")
    report = validated["stderr"]
    if recovery or untyped:
        require(validated["exit"] == 1 and validated["ast"] is None and report.startswith(HEADER + "\n"),
                "failure channel")
        first = report.splitlines()[1]
        if recovery:
            require(first.startswith("line 1: Expecting:") and "but found: 'A' [A]" in report and
                    "Could not resolve reference to Feature named 'new'." in report, "syntax diagnostic")
        else:
            require(first.startswith("line 1: " + TYPING_DIAGNOSTIC) and report.count(TYPING_DIAGNOSTIC) == 3,
                    "typing diagnostics")
    else:
        require(validated["exit"] == 0 and report == "" and validated["ast"] == ast, "clean validation AST")


def check(record):
    context = record["context"]
    require((context["baseline"], context["release_source"], context["same_cli_sha256_at_both_sources"])
            == (BASELINE, RELEASE, CLI), "source pins")
    require(set(context["runtime_exclusions"]) == {"EMF", "editing-context AQL", "evaluation", "application", "Rust"},
            "runtime scope")
    runs = record["captures"]
    require(set(runs) == set(CASES), "run scopes")
    for label, run in runs.items():
        require(SHA_PATTERN.fullmatch(run["receipt_sha256"]) and run["kernel_socket_deny_self_test"] == "passed",
                "captured run admission")
        require(set(run["observations"]) == CASES[label], "fixture set")
        for name, row in run["observations"].items():
            check_fixture(name, row)
    initial = runs["initial"]["observations"]
    require(initial["abstract-new"]["validate"]["stderr"] == initial["concrete-new"]["validate"]["stderr"],
            "paired syntax diagnostics")


def reject_mutations(record):
    edits = (
        (("initial", "abstract-new", "bare", "exit"), 1),
        (("initial", "concrete-new", "bare", "ast", "expressions", 0, "class"), "ConstructorExpression"),
        (("initial", "abstract-new", "bare", "ast", "expressions", 0, "references", 0, "target"), "P::A"),
        (("initial", "part-call", "validate", "exit"), 1),
        (("initial", "calculation-call", "validate", "exit"), 0),
        (("followup", "abstract-part-call", "bare", "ast", "declarations", 0, "abstract_marker"), None),
        (("followup", "typed-calculation-call", "bare", "ast", "expressions", 0, "references", 0, "target"), None),
    )
    for (run, name, *keys), value in edits:
        mutation = copy.deepcopy(record)
        parent = mutation["captures"][run]["observations"][name]
        for key in keys[:-1]:
            parent = parent[key]
        parent[keys[-1]] = value
        try:
            check(mutation)
        except ValueError:
            continue
        raise ValueError(f"accepted mutation {run} {name} {keys}")
    return len(edits)


def main():
    raw = OBSERVATION.read_bytes()
    require(sha256(raw).hexdigest() == OBSERVATION_SHA, "observation changed")
    record = json.loads(raw)
    check(record)
    print(f"Verified 6 fixtures in both CLI contexts and rejected {reject_mutations(record)} mutations.")


if __name__ == "__main__":
    main()
