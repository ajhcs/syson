"""Check the archived outcomes of four visibility cases."""
from hashlib import sha256
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
RECEIPT_SHA256 = "068663011d10ba8548f1071e7a274198fa81bd6b64a34035407dffdb98b00c9a"

EXPECTED = {
    "peer-private": ("A::T", None, ["A", None], "B", "private", 10, "A::T"),
    "peer-public": ("A::T", "A::T", ["A", "A::T"], "B", None, 10, "A::T"),
    "local-private": ("T", "A::T", ["A::T"], "A", "private", 8, "A::T"),
    "missing": ("A::Missing", None, ["A", None], "B", None, 10, None),
}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(data):
    return sha256(data).hexdigest()


def check(data):
    cases = data["cases"]
    require([case["name"] for case in cases] == list(EXPECTED), "exact case order")
    for row in cases:
        name = row["name"]
        text, target, parts, owner, visibility, count, rust_target = EXPECTED[name]
        require(digest(row["source"].encode()) == row["source_sha256"], "exact source")
        ast = row["release"]
        require((ast["text"], ast["target"], ast["parts"]) == (text, target, parts), "release AST link")
        saved = row["converter"]
        require(saved["edge_saved"] is True and saved["typed_feature"] == "x"
                and saved["type"] == target and saved["usage_scope"] == owner
                and type(saved["resource_nodes"]) is int and saved["resource_nodes"] == count
                and saved["membership_visibility"] == visibility, "saved SysON edge")
        warnings = [] if target is not None else [[
            "WARNING",
            f"Unable to resolve name '{text}' for reference 'type' on element '[FeatureTyping] B::x::<FeatureTyping>'",
        ]]
        require(saved["messages"] == warnings, "converter warning")
        rust = row["rust"]
        if rust_target is None:
            require(rust["status"] == "error" and rust["error"] == 'Link(Unresolved("Missing"))'
                    and rust["identities_requested"] == 6 and "target" not in rust, "Rust refusal")
        else:
            require(rust["status"] == "ok" and rust["target"] == rust_target
                    and rust["owner"] == owner and rust["membership_visibility"] == visibility
                    and rust["identities_requested"] == (6 if owner == "A" else 7), "Rust saved edge")


def main():
    raw = (ROOT / "observation.json").read_bytes()
    require(digest(raw) == RECEIPT_SHA256, "observation checksum")
    check(json.loads(raw))
    print("PASS: two resolved native controls, two retained unresolved edges and exact Rust outcomes")


if __name__ == "__main__":
    main()
