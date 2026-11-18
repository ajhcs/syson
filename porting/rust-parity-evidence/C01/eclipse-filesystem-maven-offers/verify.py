"""Verify the paired Maven offers against the captured declarations without a network."""
from copy import deepcopy
import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
SEALS = {
    "observation.json": "37166833e573ab008b3cbeb43af222860c57d2926335813b44d779332c7db78e",
    "FilesystemOffers.java": "e9adef5d31178e27cd0d388784293b15d14869b4012b4658b46be8203c811ceb",
    "../eclipse-filesystem-metadata-passive-run/observation.json":
        "3269cbf40e827dc12f0a0ec79e02e8c5fbbcc47759509a87d2dd0adc54a2e906",
}
GA = "org.eclipse.platform:org.eclipse.core.filesystem"
RANGE = "[1.7.0,2.0.0)"
TOOLS = ["21.0.12.1+1-LTS", "3.9.16"]
TERMINAL = ["terminal", "offers-only", "no-descriptor-or-binary-resolution"]


def require(condition, detail):
    if not condition:
        raise ValueError(detail)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def read(name):
    data = (HERE / name).read_bytes()
    require(digest(data) == SEALS[name], f"byte seal changed: {name}")
    return data


def numeric(version):
    parts = version.split(".")
    require(len(parts) == 3 and all(part.isascii() and part.isdecimal() for part in parts),
            "unexpected captured version spelling")
    return tuple(int(part) for part in parts)


def trace_digest(records):
    return digest("".join("\t".join(row) + "\n" for row in records).encode("utf-8"))


def check_cases(cases, declarations):
    require(list(cases) == ["pinned", "withheld"], "paired case roster changed")
    expected = [v for v in declarations if (1, 7, 0) <= numeric(v) < (2, 0, 0)]
    require(len(declarations) == len(set(declarations)) == 26 and len(expected) == 25,
            "capture or numeric boundary changed")
    for name, case in cases.items():
        rows = case["records"]
        state = "pinned" if name == "pinned" else "absent"
        offers = expected if name == "pinned" else []
        require(rows == [
            ["case", name, GA, RANGE], ["tools", *TOOLS],
            ["metadata", "local", "RELEASE_OR_SNAPSHOT", "absent"],
            ["metadata", "central", "RELEASE_OR_SNAPSHOT", state],
            *[["offer", version, "central"] for version in offers],
            ["offers", str(len(offers))], TERMINAL,
        ], f"unexpected offers, order, context or terminal: {name}")
        require(case["stdout_sha256"] == trace_digest(rows), f"trace seal changed: {name}")

    offered = [row[1] for row in cases["pinned"]["records"] if row[0] == "offer"]
    retained = [v for v in declarations if v in offered]
    return {
        "declared": len(declarations), "offered": len(offered),
        "omitted": [v for v in declarations if v not in offered],
        "added": [v for v in offered if v not in declarations],
        "retained_xml_order": retained == offered,
    }


def check_negatives(cases, declarations):
    def swap_offers(rows):
        rows[4], rows[5] = rows[5], rows[4]

    controls = (
        ("reordered", "pinned", swap_offers),
        ("below lower bound", "pinned", lambda rows: rows[4].__setitem__(1, "1.6.1")),
        ("unexpected origin", "pinned", lambda rows: rows[4].__setitem__(2, "local")),
        ("central present in withheld", "withheld", lambda rows: rows[3].__setitem__(3, "pinned")),
        ("offer in withheld", "withheld", lambda rows: rows.insert(4, ["offer", "1.7.0", "central"])),
        ("selection terminal", "pinned", lambda rows: rows[-1].__setitem__(1, "selected")),
    )
    for name, arm, mutate in controls:
        candidate = deepcopy(cases)
        rows = candidate[arm]["records"]
        mutate(rows)
        candidate[arm]["stdout_sha256"] = trace_digest(rows)
        try:
            check_cases(candidate, declarations)
        except ValueError:
            continue
        raise ValueError(f"accepted resealed negative: {name}")
    return len(controls)


def verify():
    read("FilesystemOffers.java")
    capture = json.loads(read("../eclipse-filesystem-metadata-passive-run/observation.json"))
    observation = json.loads(read("observation.json"))
    declarations = capture["metadata_declarations"]
    require(observation["schema"] == "SYSON-C01-ECLIPSE-FILESYSTEM-MAVEN-OFFERS/1"
            and observation["status"] == "OBSERVED_PAIRED_DATA_ONLY_REPLAY"
            and observation["capture_commit"] == "b165d3b386dad5cad0af338413c2a3272f3a14f2"
            and observation["probe_sha256"] == SEALS["FilesystemOffers.java"]
            and observation["range"] == RANGE
            and observation["metadata_sha256"] == declarations["metadata_sha256"] == capture["request_rows"][0][2]
            and declarations["group_id"] + ":" + declarations["artifact_id"] == GA,
            "captured metadata or replay input identity changed")
    cases = observation["cases"]
    compare = check_cases(cases, declarations["versions"])
    require(compare == observation["declaration_comparison"] ==
            {"declared": 26, "offered": 25, "omitted": ["1.6.1"], "added": [], "retained_xml_order": True},
            "capture comparison changed")
    require(observation["process_exits"] == dict.fromkeys(("kernel", "compiler", "pinned", "withheld"), 0),
            "process exits changed")
    kernel = b"PASS socket(AF_INET, AF_INET6, AF_UNIX) and io_uring_setup denied by kernel\n"
    require(observation["controls"] == {
        "inputs_match_before_and_after": True, "controller_exit": 0,
        "owned_scope_confirmed_empty_or_absent": True,
        "kernel_witness_sha256": digest(kernel), "network": False, "pom_resolution": False,
        "binary_resolution": False, "cache_installation": False, "project_execution": False,
    }, "offline or process cleanup evidence changed")
    negatives = check_negatives(cases, declarations["versions"])
    print(f"PASS 25 pinned offers in XML order, 0 withheld offers, and {negatives} rejected mutations")


if __name__ == "__main__":
    verify()
