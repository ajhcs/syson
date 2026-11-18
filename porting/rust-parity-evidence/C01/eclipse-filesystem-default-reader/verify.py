"""Verify the default Maven descriptor reads and compare them to #391 offline."""
from copy import deepcopy
import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
SEALS = {
    "observation.json": "b43f35f3cb5831e85da4a6df94f470bf6cefb439384025d0a1bc32a468a41b65",
    "FilesystemDescriptorProbe.java": "59cc776af9a6d5394703a218b4d4918b66418b4b02b170e5035e1621fc6cb640",
    "../eclipse-filesystem-direct-model/observation.json":
        "79467ff3d54bceb01955245346240aae910f5d3c7c81cf8ad7d2c52c7e7d0f15",
    "../eclipse-filesystem-descriptor-passive-run/observation.json":
        "91f18d33d16b20aea6b1d7347a3154e3b18d2bfb23d9ca3d7ecb4d9f1e4bf25d",
}
HEADER = [["java", "21.0.12.1+1-LTS"], ["maven-model-builder", "3.9.16"]]
READER_TERMINAL = ["terminal", "direct-descriptor-only", "no-collection-or-binary-resolution"]
REFUSAL_TERMINAL = ["terminal", "outside-coordinate-refused", "before-descriptor-reader"]
NOTICE_LINES = [
    'SLF4J: Failed to load class "org.slf4j.impl.StaticLoggerBinder".',
    "SLF4J: Defaulting to no-operation (NOP) logger implementation",
    "SLF4J: See http://www.slf4j.org/codes.html#StaticLoggerBinder for further details.",
]


def require(condition, detail):
    if not condition:
        raise ValueError(detail)


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def read(name):
    raw = (HERE / name).read_bytes()
    require(digest(raw) == SEALS[name], f"byte seal changed: {name}")
    return raw


def trace_digest(records):
    return digest("".join("\t".join(row) + "\n" for row in records).encode("utf-8"))


def compare_edges(strict, capture):
    edges = {}
    for row in strict["cases"]["direct_model"]["records"]:
        if row[0] == "case":
            version = row[1]
            edges[version] = []
        if row[0] == "direct":
            edges[version].append(["edge", *row[1:7], *row[8:]])
    require(len(edges) == 25 and not edges["1.9.500"] and sum(map(len, edges.values())) == 90,
            "strict comparator acceptance changed")
    rejected = strict["rejected_raw_direct_tuples"]
    require(len(rejected) == 3, "rejected POM tuple count changed")
    edges["1.9.500"] = [["edge", *row[:6], *row[7:]] for row in rejected]

    declarations = capture["raw_model_declarations"]
    require(len(declarations) == 25 and [r[0] for r in declarations] == list(edges),
            "passive POM roster changed")
    trace = HEADER.copy()
    for index, (version, spelling, _) in enumerate(declarations):
        status, _, source_hash = capture["request_rows"][5 * index]
        require(status == 200 and spelling == ("4.0" if version == "1.9.500" else "4.0.0"),
                f"capture identity changed: {version}")
        trace.extend([
            ["case", version, source_hash, "3"], ["callbacks", "1", "1", "1", "1", "0"],
            *edges[version], ["native-built", str(len(edges[version]))],
        ])
    trace.append(READER_TERMINAL)
    require(sum(map(len, edges.values())) == 93, "descriptor edge count changed")
    return trace


def check_cases(cases, expected_reader):
    require(list(cases) == ["default_reader", "outside_coordinate"], "expected a reader census and refusal")
    reader, negative = cases["default_reader"], cases["outside_coordinate"]
    require(reader["records"] == expected_reader and reader["exit"] == 0, "default descriptor census changed")
    require(negative["records"] == HEADER + [REFUSAL_TERMINAL] and negative["exit"] == 42
            and negative["requested_release"] == "1.6.1", "outside cohort did not stop before reader")
    for case in cases.values():
        require(case["stdout_sha256"] == trace_digest(case["records"]), "trace digest changed")


def check_stderr_and_initial(obs):
    notice_bytes = ("\n".join(NOTICE_LINES) + "\n").encode("utf-8")
    require(len(notice_bytes) == 210 and obs["census_stderr"] == {
        "lines": NOTICE_LINES, "bytes": 210, "sha256": digest(notice_bytes),
        "allowed_bytes_and_sha256": [[0, digest(b"")], [210, digest(notice_bytes)]],
    }, "stderr or exact notice allowlist changed")
    initial = obs["initial_unqualified_attempt"]
    require(initial["status"] == "RETAINED_PARTIAL_OR_UNQUALIFIED"
            and initial["failure"] == "ValueError: runtime versions/stderr differ"
            and initial["controller_exit"] == 1
            and initial["completed_task_exits"] == {"kernel": 0, "compiler": 0, "default_reader": 0}
            and initial["default_reader_stdout_sha256"] == obs["cases"]["default_reader"]["stdout_sha256"]
            and initial["default_reader_stderr_sha256"] == obs["census_stderr"]["sha256"]
            and initial["unrun"] == ["outside_coordinate"]
            and initial["owned_scope_confirmed_empty_or_absent"] is True, "initial failed/unrun boundary changed")


def check_negatives(obs, expected_reader):
    def alter_trace(value, case, line, field, replacement):
        evidence = value["cases"][case]
        evidence["records"][line][field] = replacement
        evidence["stdout_sha256"] = trace_digest(evidence["records"])

    controls = (
        ("different input", lambda value: alter_trace(value, "default_reader", 2, 2, "0" * 64)),
        ("invalid descriptor event", lambda value: alter_trace(value, "default_reader", 3, 5, "1")),
        ("dependency scope changed", lambda value: alter_trace(value, "default_reader", 4, 6, "test")),
        ("graph terminal", lambda value: alter_trace(value, "default_reader", -1, 1, "graph-complete")),
        ("outside cohort reached reader", lambda value: alter_trace(value, "outside_coordinate", -1, 2, "inside-reader")),
        ("broader stderr allowlist", lambda value: value["census_stderr"]["allowed_bytes_and_sha256"].append([1, "0" * 64])),
        ("initial control reported run", lambda value: value["initial_unqualified_attempt"].update(unrun=[])),
    )
    for name, mutate in controls:
        changed = deepcopy(obs)
        mutate(changed)
        try:
            check_cases(changed["cases"], expected_reader)
            check_stderr_and_initial(changed)
        except ValueError:
            continue
        raise ValueError(f"accepted resealed negative: {name}")
    return len(controls)


def verify():
    read("FilesystemDescriptorProbe.java")
    strict = json.loads(read("../eclipse-filesystem-direct-model/observation.json"))
    capture = json.loads(read("../eclipse-filesystem-descriptor-passive-run/observation.json"))
    obs = json.loads(read("observation.json"))
    require(obs["schema"] == "SYSON-C01-ECLIPSE-FILESYSTEM-DEFAULT-READER/1"
            and obs["status"] == "OBSERVED_PAIRED_DATA_ONLY_REPLAY"
            and obs["direct_model_commit"] == "8549ae744107743ecbabdea134b433172ebd07c6"
            and strict["validation"]["level"] == 30 and obs["policy"] == {
                "reader": "org.apache.maven.repository.internal.DefaultArtifactDescriptorReader",
                "default_validation_level": 0, "strict_comparator_validation_level": 30,
                "descriptor_policy_flags": 3, "ignore_invalid": True, "ignore_missing": True,
            }, "policy, scope, or comparator identity changed")
    require(obs["callback_record_fields"] == ["fixed_jar_coordinate_echo", "fixed_pom_coordinate_echo",
            "pinned_pom_read", "repository_aggregation", "invalid_descriptor_event"] and obs["counts"] == {
                "descriptor_models": 25, "descriptor_edges": 93, "strict_models": 24, "strict_edges": 90,
                "edge_delta_version": "1.9.500", "edge_delta": 3,
            }, "callback semantics or edge counts changed")
    expected_reader = compare_edges(strict, capture)
    check_cases(obs["cases"], expected_reader)
    check_stderr_and_initial(obs)
    kernel = b"PASS socket(AF_INET, AF_INET6, AF_UNIX) and io_uring_setup denied by kernel\n"
    require(obs["controls"] == {
        "inputs_match_before_and_after": True, "controller_exit": 0,
        "owned_scope_confirmed_empty_or_absent": True, "kernel_witness_sha256": digest(kernel),
        "network": False, "offer_resolution": False, "graph_collection": False, "final_version_selected": False,
        "binary_resolution": False, "cache_installation": False, "project_execution": False,
    }, "offline scope or cleanup evidence changed")
    negatives = check_negatives(obs, expected_reader)
    print(f"PASS 25 descriptors, 93 edges and exact 1.9.500 delta, refused 1.6.1, controlled stderr, {negatives} negatives")


if __name__ == "__main__":
    verify()
