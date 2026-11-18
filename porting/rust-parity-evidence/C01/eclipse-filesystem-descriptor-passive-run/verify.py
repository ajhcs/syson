"""Verify the public Filesystem descriptor capture and its frozen offer roster."""
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import re

HERE = Path(__file__).resolve().parent
CORE = HERE.parents[2] / "rust-parity-plan-20261026" / "closure" / "core-compile"
EPOCH = "eclipse-filesystem-descriptors-01"
GROUP = "org.eclipse.platform"
ARTIFACT = "org.eclipse.core.filesystem"
ORIGIN = "https://repo.maven.apache.org/maven2/"
SEALS = {
    CORE / "acquire.py": "d41d8c7598befd3225552da6e278efb2fe4f06d72fda8103d14f1120e3198ef4",
    CORE / "requests" / f"{EPOCH}.json": "559988c53686025a527d483e8e884793e8d41e6cdbb73c33315741cbe381a079",
    HERE / "observation.json": "91f18d33d16b20aea6b1d7347a3154e3b18d2bfb23d9ca3d7ecb4d9f1e4bf25d",
    HERE.parent / "eclipse-filesystem-maven-offers" / "observation.json":
        "37166833e573ab008b3cbeb43af222860c57d2926335813b44d779332c7db78e",
}
ORDER = (
    ("", "pom", 200),
    (".sha512", "sha512", 404),
    (".sha256", "sha256", 404),
    (".sha1", "sha1", 200),
    (".asc", "signature", 200),
)
DIGEST = re.compile(r"[0-9a-f]{64}")


def require(condition, detail):
    if not condition:
        raise ValueError(detail)


def read(path):
    limit = 256 * 1024
    with path.open("rb") as stream:
        raw = stream.read(limit + 1)
    require(len(raw) <= limit and hashlib.sha256(raw).hexdigest() == SEALS[path],
            f"unexpected byte seal: {path.name}")
    return raw


def check_projection(manifest, obs, versions):
    require(obs["status"] == "PASS_PASSIVE_ACQUISITION_ONLY" and
            obs["epoch"] == manifest["epoch"] == EPOCH and manifest["status"] == "PROPOSED_ONLY",
            "capture epoch or status changed")
    require(len(versions) == len(set(versions)) == len(manifest["new_primaries"]) == 25
            and len(manifest["requests"]) == len(obs["request_rows"]) == 5 * len(versions),
            "capture cardinality changed")
    require(obs["response_row_fields"] == ["http_status", "body_bytes", "sha256"],
            "unexpected response columns")
    count = {"sha512_404": 0, "sha256_404": 0, "sha1_200": 0, "signature_200": 0}
    total = 0
    for index, (version, primary) in enumerate(zip(versions, manifest["new_primaries"], strict=True)):
        relative = f"{GROUP.replace('.', '/')}/{ARTIFACT}/{version}/{ARTIFACT}-{version}.pom"
        require(primary["coordinate"] == {"groupId": GROUP, "artifactId": ARTIFACT, "version": version}
                and primary["primary_index"] == index and primary["kind"] == "pom"
                and primary["relative_artifact"] == relative and primary["url"] == ORIGIN + relative,
                f"offered coordinate or primary changed: {version}")
        for offset, (suffix, role, status) in enumerate(ORDER):
            request = manifest["requests"][5 * index + offset]
            row = obs["request_rows"][5 * index + offset]
            require(request["primary_index"] == index and request.get("algorithm", request["role"]) == role
                    and request["url"] == ORIGIN + relative + suffix, f"request role or URL changed: {version} {role}")
            require(isinstance(row, list) and len(row) == 3, "response shape changed")
            code, size, digest = row
            require(type(code) is int and code == status and status in request["allowed_status"]
                    and type(size) is int and 0 < size <= request["maximum_bytes"]
                    and (role != "sha1" or size == 40) and isinstance(digest, str) and DIGEST.fullmatch(digest),
                    f"response differs for {version} {role}")
            total += size
            if suffix:
                count[f"{role}_{code}"] += 1
    require(total == obs["body_bytes"] == 108147
            and total <= manifest["limits"]["payload_and_sidecar_total_bytes"], "body accounting changed")
    require(count == obs["sidecar_http_counts"] == dict.fromkeys(count, 25)
            and obs["same_origin_matches"] == {"sha512": 0, "sha256": 0, "sha1": 25},
            "sidecar outcomes changed")

    declarations = obs["raw_model_declarations"]
    require(obs["model_declaration_fields"] == ["version", "model_version", "maven_namespace_present"]
            and [row[0] for row in declarations] == versions
            and all(len(row) == 3 and row[1] == ("4.0" if row[0] == "1.9.500" else "4.0.0")
                    and type(row[2]) is bool for row in declarations)
            and sum(row[2] for row in declarations) == 12
            and ["1.9.500", "4.0", True] in declarations, "raw model declarations changed")
    require(obs["outside_scope"] == dict.fromkeys((
        "publisher_signature_verified", "model_builder_acceptance", "graph_effects_observed",
        "final_version_selected", "cache_installation", "binary_execution", "offline_native_compile",
    ), False), "unsupported capture claim")


def check_negatives(manifest, obs, versions):
    def swap_primaries(candidate, _):
        primaries = candidate["new_primaries"]
        primaries[0], primaries[1] = primaries[1], primaries[0]

    controls = (
        ("reordered roster", swap_primaries),
        ("unadmitted URL", lambda candidate, _: candidate["requests"][0].update(url=ORIGIN + "other.xml")),
        ("SHA-512 404 became 200", lambda _, value: value["request_rows"][1].__setitem__(0, 200)),
        ("SHA-1 truncated", lambda _, value: value["request_rows"][3].__setitem__(1, 39)),
        ("non-digest POM", lambda _, value: value["request_rows"][0].__setitem__(2, "invalid")),
        ("anomalous version normalized", lambda _, value: next(
            row for row in value["raw_model_declarations"] if row[0] == "1.9.500").__setitem__(1, "4.0.0")),
        ("signature declared verified", lambda _, value: value["outside_scope"].update(publisher_signature_verified=True)),
    )
    for name, mutate in controls:
        changed_manifest, changed_obs = deepcopy(manifest), deepcopy(obs)
        mutate(changed_manifest, changed_obs)
        try:
            check_projection(changed_manifest, changed_obs, versions)
        except ValueError:
            continue
        raise ValueError(f"accepted mutated projection: {name}")
    return len(controls)


def verify():
    client = CORE / "acquire.py"
    manifest_path = CORE / "requests" / f"{EPOCH}.json"
    offers_path = HERE.parent / "eclipse-filesystem-maven-offers" / "observation.json"
    read(client)
    manifest = json.loads(read(manifest_path))
    obs = json.loads(read(HERE / "observation.json"))
    offers = json.loads(read(offers_path))
    versions = [row[1] for row in offers["cases"]["pinned"]["records"] if row[0] == "offer"]
    require(obs["schema"] == "SYSON-C01-ECLIPSE-FILESYSTEM-POM-PASSIVE/1"
            and obs["proposal_head"] == "a918815ff1a6f352c792fa297ef60db3a6ec3bd0"
            and obs["manifest_sha256"] == SEALS[manifest_path] and obs["client_sha256"] == SEALS[client]
            and manifest["source_bindings"]["filesystem_resolver_offers"]["sha256"] == SEALS[offers_path],
            "capture client or source bindings changed")
    check_projection(manifest, obs, versions)
    require(obs["frozen_input_count"] == 66 and obs["frozen_inputs_match_before_and_after"] is True
            and obs["all_final_urls_match_roster"] is True and obs["controller_exit"] == 0
            and obs["owned_scope_cleanup"] == "not-found"
            and obs["configured_limits"] == {
                "cpu.max": "100000 100000", "memory.high": "536870912", "memory.max": "1073741824",
                "memory.swap.max": "0", "pids.max": "32",
            }, "capture or cleanup facts changed")
    require(set(obs["custody_sha256"]) == {"frozen_scope", "getter_receipt", "controller_receipt"}
            and all(DIGEST.fullmatch(value) for value in obs["custody_sha256"].values()), "custody seals changed")
    negatives = check_negatives(manifest, obs, versions)
    print(f"PASS 125 responses, 25 POMs, 25 same-origin SHA-1 matches, and {negatives} rejected mutations")


if __name__ == "__main__":
    verify()
