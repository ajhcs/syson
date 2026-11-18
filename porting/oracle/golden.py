# SPDX-License-Identifier: EPL-2.0
"""Record and verify passive reference results."""

import hashlib
import json
import re

import compare

REFERENCE = {
    "jar_sha256": "85c031352695d6ac8f059f0215ca493c58a16805a111233998dd443d6990c9e1",
    "jar_source_revision": "b380dada415ce05155150f23cb3b74ae21cc45e4",
}
MAX_BYTES = 32 * 1024 * 1024
FIELDS = {"schema", "id", "source_sha256", "reference", "result", "normalized", "record_sha256"}


class GoldenError(ValueError):
    pass


def case_name(case_id):
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_.-]{0,199}", case_id):
        raise GoldenError(f"invalid golden case ID: {case_id!r}")
    return f"{case_id}.json"


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def record_digest(record):
    body = {k: v for k, v in record.items() if k != "record_sha256"}
    return digest(json.dumps(body, separators=(",", ":"), allow_nan=False).encode())


def normalized_result(result):
    status, document = result.get("status"), result.get("document")
    if status not in ("ok", "empty", "error"):
        raise GoldenError(f"reference result is not recordable: {status!r}")
    if status == "error":
        if document is not None or not isinstance(result.get("error"), str):
            raise GoldenError("reference refusal lacks an error or contains a document")
        return None
    if not isinstance(document, dict) or not isinstance(document.get("content"), list):
        raise GoldenError("reference result lacks a document content list")
    if bool(document["content"]) != (status == "ok"):
        raise GoldenError("reference status and document content disagree")
    return compare.normalize(document)


def capture(case_id, source, state, result):
    case_name(case_id)
    reference = {key: state.get(key) for key in REFERENCE}
    if reference != REFERENCE:
        raise GoldenError("live reference does not match the pinned release")
    record = {
        "schema": 1,
        "id": case_id,
        "source_sha256": digest(source.encode("utf-8")),
        "reference": reference,
        "result": result,
        "normalized": normalized_result(result),
    }
    record["record_sha256"] = record_digest(record)
    return record


def unique_object(pairs):
    values = {}
    for key, value in pairs:
        if key in values:
            raise GoldenError(f"duplicate key: {key}")
        values[key] = value
    return values


def write(directory, records):
    encoded = {}
    for record in records:
        name = case_name(record["id"])
        body = (json.dumps(record, indent=1, ensure_ascii=False, allow_nan=False) + "\n").encode("utf-8")
        if name in encoded or len(body) > MAX_BYTES:
            raise GoldenError(f"duplicate or oversized record: {name}")
        encoded[name] = body
    directory.mkdir()
    for name, body in encoded.items():
        with (directory / name).open("xb") as stream:
            stream.write(body)


def load(directory, case_id, source):
    path = directory / case_name(case_id)
    try:
        with path.open("rb") as stream:
            data = stream.read(MAX_BYTES + 1)
        if len(data) > MAX_BYTES:
            raise GoldenError("record exceeds size limit")
        record = json.loads(data, object_pairs_hook=unique_object)
        if type(record) is not dict or set(record) != FIELDS:
            raise GoldenError("unexpected golden record shape")
        if type(record["schema"]) is not int or record["schema"] != 1 or record["id"] != case_id:
            raise GoldenError("golden record schema or ID mismatch")
        if record["source_sha256"] != digest(source.encode("utf-8")):
            raise GoldenError("stale source bytes; re-record the reference result")
        if record["reference"] != REFERENCE:
            raise GoldenError("record uses a different reference release")
        if record["record_sha256"] != record_digest(record):
            raise GoldenError("record digest mismatch")
        result = record["result"]
        if type(result) is not dict or record["normalized"] != normalized_result(result):
            raise GoldenError("raw and normalized reference results differ")
        return result
    except FileNotFoundError as error:
        raise GoldenError(f"{case_id}: golden record missing at {path}") from error
    except (OSError, ValueError, TypeError, KeyError, AttributeError, RecursionError) as error:
        raise GoldenError(f"{case_id}: {error}") from error
