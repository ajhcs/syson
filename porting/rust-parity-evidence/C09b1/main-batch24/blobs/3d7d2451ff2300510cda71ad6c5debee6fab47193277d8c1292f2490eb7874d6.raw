"""Compare captured public fields; preserve every difference and named OPEN gate."""
import argparse
import copy
import hashlib
import json
import struct
from pathlib import Path

ABSENT = {"state": "ABSENT"}
META = {"$type", "$cstNode", "$container", "$containerProperty", "$containerIndex", "$childIndex", "$meta", "$document"}


CORPUS_SHA256 = "a4fb319e8b30b3c50504bdcc42f38b35acee7ae179550874e66bf113c35d05f4"
REQUIRED_CASES = [tuple(line.split()) for line in """
package-definition-typed-usage finite
body-membership-context finite
qualified-typing-raw-segments finite
quoted-unicode-byte-utf16-spans finite
anonymous-package-semicolon finite
anonymous-package-body finite
anonymous-definition-semicolon finite
anonymous-definition-body finite
empty-whitespace finite
empty-source finite
defaults-and-absent-optionals finite
repeated-declaration-order finite
quoted-keyword-name finite
nested-packages finite
unicode-qualified-typing finite
crlf-token-spans finite
old-arena-source finite
new-arena-source finite
limit-fixture-native-parse finite
incomplete-package-body malformed
incomplete-definition malformed
missing-typing-name malformed
incomplete-qualified-reference malformed
missing-usage-semicolon malformed
unmatched-closing-brace malformed
unclosed-name malformed
unadmitted-name-escape malformed
outside-private-package outside-whitelist
outside-untyped-usage outside-whitelist
outside-usage-body outside-whitelist
outside-short-name outside-whitelist
outside-comment outside-whitelist
outside-port-definition outside-whitelist
outside-specialization outside-whitelist
outside-bom-whitespace outside-whitelist
keyword-def outside-whitelist
keyword-private outside-whitelist
keyword-in outside-whitelist
keyword-package-reference outside-whitelist
outer-whitespace-package finite
outer-whitespace-definition finite
trailing-whitespace-package finite
""".splitlines() if line]


def require(condition, message):
    if not condition:
        raise ValueError(message)


def strict_loads(text):
    def pairs(items):
        result = {}
        for key, value in items:
            require(key not in result, "duplicate JSON key: " + key)
            result[key] = value
        return result
    def constant(value):
        raise ValueError("nonfinite JSON constant: " + value)
    return json.loads(text, object_pairs_hook=pairs, parse_constant=constant)


def validate_corpus(corpus, assigned=False):
    require(isinstance(corpus, dict) and type(corpus.get("schema")) is int
            and corpus["schema"] == 1, "corpus schema")
    cases = corpus.get("cases")
    require(isinstance(cases, list) and bool(cases), "nonempty corpus required")
    ids = []
    for spec in cases:
        require(isinstance(spec, dict), "corpus case object")
        ident, category = spec.get("id"), spec.get("category")
        require(isinstance(ident, str) and bool(ident), "nonempty case ID")
        require(isinstance(spec.get("source"), str), "case source string")
        require(category in {"finite", "malformed", "outside-whitelist"}, "case category")
        expected = {"finite": None, "malformed": "Malformed", "outside-whitelist": "Unsupported"}[category]
        require(spec.get("expected_restriction") == expected, "category/restriction mismatch")
        ids.append(ident)
    require(len(ids) == len(set(ids)), "duplicate corpus IDs")
    if assigned:
        require([(c["id"], c["category"]) for c in cases] == REQUIRED_CASES,
                "required 42 case IDs/categories/order changed")
    return cases


def validate_rust_capture(capture):
    def keys(value, required, label):
        require(type(value) is dict and set(value) == set(required), label + " keys")
    def unsigned(value):
        return type(value) is int and value >= 0
    def lexeme(value):
        keys(value, {"bytes", "utf16", "text"}, "Rust lexeme")
        require(type(value["text"]) is str, "Rust lexeme text")
        for axis in ["bytes", "utf16"]:
            pair = value[axis]
            require(type(pair) is list and len(pair) == 2 and all(unsigned(v) for v in pair)
                    and pair[0] <= pair[1], "Rust lexeme offsets")
    def datum(value):
        keys(value, {"value", "lexeme"}, "Rust Datum")
        if value["lexeme"] is not None:
            lexeme(value["lexeme"])
        item = value["value"]
        require(type(item) is dict and type(item.get("kind")) is str, "Rust value kind")
        kind = item["kind"]
        shapes = {"null": {"kind"}, "boolean": {"kind", "value"}, "number": {"kind", "bits"},
                  "string": {"kind", "units"}, "node": {"kind", "id"}, "array": {"kind", "items"},
                  "unresolved_reference": {"kind", "text", "target_type"}}
        require(kind in shapes, "unknown Rust value kind")
        keys(item, shapes[kind], "Rust " + kind)
        if kind == "boolean":
            require(type(item["value"]) is bool, "Rust Boolean payload")
        elif kind == "number":
            bits = item["bits"]
            require(type(bits) is str and len(bits) == 16 and all(c in "0123456789abcdef" for c in bits), "Rust Number bits")
        elif kind == "string":
            require(type(item["units"]) is list and all(unsigned(v) and v <= 65535 for v in item["units"]), "Rust UTF-16 units")
        elif kind == "node":
            require(unsigned(item["id"]) and item["id"] in ids, "Rust child handle")
        elif kind == "array":
            require(type(item["items"]) is list, "Rust Array items")
            for child in item["items"]:
                datum(child)
        elif kind == "unresolved_reference":
            require(type(item["text"]) is str and type(item["target_type"]) is str, "Rust unresolved-reference strings")
    keys(capture, {"id", "source", "outcome"}, "Rust case")
    require(type(capture["id"]) is str and type(capture["source"]) is str, "Rust case identity/source")
    outcome = capture["outcome"]
    require(type(outcome) is dict and type(outcome.get("kind")) is str, "Rust outcome kind")
    if outcome["kind"] == "rust_error":
        keys(outcome, {"kind", "error"}, "Rust error")
        require(type(outcome["error"]) is str, "Rust error text")
        return
    require(outcome["kind"] == "arena", "unknown Rust outcome")
    keys(outcome, {"kind", "root", "nodes"}, "Rust arena")
    require(type(outcome["nodes"]) is list and bool(outcome["nodes"]), "Rust arena nodes")
    for node in outcome["nodes"]:
        keys(node, {"id", "node_type", "fields", "origin", "container"}, "Rust node")
        require(unsigned(node["id"]), "Rust node ID")
    ids = {node["id"] for node in outcome["nodes"]}
    require(len(ids) == len(outcome["nodes"]) and unsigned(outcome["root"]) and outcome["root"] in ids, "Rust arena root/IDs")
    for node in outcome["nodes"]:
        require(type(node["node_type"]) is str and type(node["fields"]) is dict, "Rust node type/fields")
        if node["origin"] is not None:
            lexeme(node["origin"])
        container = node["container"]
        if container is not None:
            keys(container, {"node", "property", "index"}, "Rust container")
            require(unsigned(container["node"]) and container["node"] in ids, "Rust container handle")
            require(type(container["property"]) is str, "Rust container property")
            require(container["index"] is None or unsigned(container["index"]), "Rust container index")
        for name, field in node["fields"].items():
            require(type(name) is str, "Rust field name")
            datum(field)


def validate_reference_indices(expected):
    require(type(expected["references"]) is list, "original references array")
    for reference in expected["references"]:
        require(type(reference) is dict and "index" in reference, "original reference index presence")
        require(type(reference["owner_path"]) is str and type(reference["property"]) is str, "original reference identity")
        index = reference["index"]
        require(index is None or (type(index) is int and index >= 0), "original reference index type")


def same(a, b):
    if type(a) is not type(b):
        return False
    if isinstance(a, dict):
        return a.keys() == b.keys() and all(same(a[k], b[k]) for k in a)
    if isinstance(a, list):
        return len(a) == len(b) and all(same(x, y) for x, y in zip(a, b))
    return a == b


def units(text):
    raw = text.encode("utf-16-le", errors="surrogatepass")
    return [int.from_bytes(raw[i:i + 2], "little") for i in range(0, len(raw), 2)]


def pointer(key):
    return key.replace("~", "~0").replace("/", "~1")


class Case:
    def __init__(self, spec, native, rust, expected):
        self.spec, self.native, self.rust, self.expected = spec, native, rust, expected
        self.differences, self.open_items = [], []
        self.npaths, self.rpaths, self.nby, self.rby = {}, {}, {}, {}
        self.naliases, self.raliases, self.used_references = [], [], set()

    def diff(self, path, kind, native=ABSENT, rust=ABSENT):
        self.differences.append(dict(path=path, kind=kind, native=native, rust=rust))

    def opened(self, path, kind, detail):
        self.open_items.append(dict(path=path, kind=kind, detail=detail))

    def props(self, obj):
        result = {}
        for field in obj["properties"]:
            key = field["key"]
            if not isinstance(key, str):
                self.opened(obj["id"], "symbol_property", field)
                continue
            if key in result:
                raise ValueError("duplicate native property: " + key)
            result[key] = field.get("value", {"kind": "unevaluated_accessor"})
        if not same([p["key"] for p in obj["properties"]], obj["property_keys"]):
            self.opened(obj["id"], "property_inventory_inconsistent", obj)
        return result

    def obj(self, value):
        if isinstance(value, dict) and set(value) == {"$object"}:
            return self.objects[value["$object"]]
        return None

    def array(self, obj):
        fields = self.props(obj)
        length = fields.get("length")
        if type(length) is not int or length < 0:
            raise ValueError("invalid native array length")
        extra = set(fields) - {"length"} - {str(i) for i in range(length)}
        if extra:
            self.opened(obj["id"], "array_extra_properties", sorted(extra))
        return [fields.get(str(i), {"kind": "array_hole"}) for i in range(length)]

    def walk_native(self, value, path):
        obj = self.obj(value)
        if not obj:
            return
        if obj["kind"] == "array":
            for i, child in enumerate(self.array(obj)):
                self.walk_native(child, path + "/" + str(i))
        elif obj["kind"] == "ast":
            if obj["id"] in self.npaths:
                self.naliases.append(dict(path=path, canonical=self.npaths[obj["id"]]))
                return
            self.npaths[obj["id"]], self.nby[path] = path, obj
            for key, child in sorted(self.props(obj).items()):
                if not key.startswith("$"):
                    self.walk_native(child, path + "/" + pointer(key))

    def walk_rust(self, value, path):
        if value["kind"] == "array":
            for i, child in enumerate(value["items"]):
                self.walk_rust(child["value"], path + "/" + str(i))
        elif value["kind"] == "node":
            ident = value["id"]
            if ident in self.rpaths:
                self.raliases.append(dict(path=path, canonical=self.rpaths[ident]))
                return
            node = self.rnodes[ident]
            self.rpaths[ident], self.rby[path] = path, node
            for key, child in sorted(node["fields"].items()):
                if not key.startswith("$"):
                    self.walk_rust(child["value"], path + "/" + pointer(key))

    def native_value(self, value, owner, key, index=None):
        obj = self.obj(value)
        if obj:
            if obj["kind"] == "ast":
                if obj["id"] not in self.npaths:
                    self.opened(owner, "unmapped_native_alias", obj["id"])
                return dict(kind="node", path=self.npaths.get(obj["id"], "UNMAPPED:" + obj["id"]))
            if obj["kind"] == "array":
                return dict(kind="array", items=[self.native_value(v, owner, key, i) for i, v in enumerate(self.array(obj))])
            fields = self.props(obj)
            if "$refText" in fields:
                identity = (owner, key, index)
                ref = self.references.get(identity)
                self.used_references.add(identity)
                if not ref or "target_type" not in ref:
                    self.opened(owner + "/" + pointer(key), "reference_type_unavailable", ref or identity)
                elif not same(ref["ref_text"], fields["$refText"]):
                    self.diff(owner, "reference_oracle_text", ref["ref_text"], fields["$refText"])
                raw_cst = self.csts.get(fields.get("$refNode", {}).get("$cst"))
                if ref and ref.get("cst") and raw_cst:
                    old = {k: raw_cst["span"].get(k, ABSENT) for k in ["offset", "end", "text"]}
                    new = {k: ref["cst"].get(k, ABSENT) for k in ["offset", "end", "text"]}
                    if not same(old, new):
                        self.diff(owner, "reference_cst_recapture", old, new)
                else:
                    self.opened(owner, "reference_cst_capture_missing", dict(raw=raw_cst, expected=ref))
                if any(k in fields for k in ["ref", "error", "$nodeDescription"]):
                    self.opened(owner, "reference_resolution_accessors_not_invoked", "Only unresolved text, target type and source/CST are compared; getter/link state remains unevaluated.")
                for prop in fields:
                    if prop not in {"$refText", "$refNode", "ref", "error", "$nodeDescription"}:
                        self.opened(owner, "reference_internal_field", dict(key=prop, value=fields[prop]))
                return dict(kind="unresolved_reference", text=fields["$refText"],
                            target_type=ref.get("target_type", ABSENT) if ref else ABSENT)
            self.opened(owner, "unhandled_native_object", obj)
            return dict(kind="unhandled_native_object", id=obj["id"])
        if type(value) is bool:
            return dict(kind="boolean", value=value)
        if isinstance(value, str):
            return dict(kind="string", units=units(value))
        if type(value) in (int, float):
            return dict(kind="number", bits=struct.pack(">d", float(value)).hex())
        if value is None:
            return dict(kind="null")
        return value  # Explicit undefined/nonfinite/accessor tags remain distinct.

    def rust_value(self, value):
        if value["kind"] == "node":
            if value["id"] not in self.rpaths:
                self.opened("root", "unmapped_rust_alias", value["id"])
            return dict(kind="node", path=self.rpaths.get(value["id"], "UNMAPPED:" + str(value["id"])))
        if value["kind"] == "array":
            return dict(kind="array", items=[self.rust_value(v["value"]) for v in value["items"]])
        return value

    def validate_lexeme(self, path, lexeme):
        try:
            start, end = lexeme["bytes"]
            a, b = lexeme["utf16"]
            if any(type(x) is not int for x in [start, end, a, b]) or not 0 <= start <= end:
                raise ValueError("invalid offsets")
            raw = self.spec["source"].encode("utf-8")
            if end > len(raw) or raw[start:end].decode("utf-8") != lexeme["text"]:
                raise ValueError("byte slice/text")
            if a != len(units(raw[:start].decode("utf-8"))) or b != len(units(raw[:end].decode("utf-8"))):
                raise ValueError("UTF-16 offsets")
            return True
        except (KeyError, TypeError, ValueError, UnicodeError) as error:
            self.diff(path, "invalid_rust_lexeme", str(error), lexeme)
            return False

    def match_span(self, path, span, lexeme):
        if not span or "offset" not in span or "end" not in span or "text" not in span:
            self.opened(path, "native_cst_span_missing", span)
            return
        actual = dict(utf16=lexeme["utf16"], text=lexeme["text"])
        expected = dict(utf16=[span["offset"], span["end"]], text=span["text"])
        if not same(expected, actual):
            self.diff(path, "cst_lexeme", expected, actual)

    def datum_lexeme(self, owner, key, datum, native, index=None):
        path = owner + "/" + pointer(key) + ("/" + str(index) if index is not None else "")
        obj = self.obj(native)
        if datum["value"]["kind"] == "array" and obj and obj["kind"] == "array":
            values = self.array(obj)
            for i, child in enumerate(datum["value"]["items"]):
                if i < len(values):
                    self.datum_lexeme(owner, key, child, values[i], i)
        literal = datum["value"]["kind"] in {"string", "number", "boolean", "unresolved_reference"}
        lexeme = datum.get("lexeme")
        ref = self.references.get((owner, key, index))
        spans = self.spans.get((owner, key))
        candidates = []
        if ref:
            if ref.get("cst_error"):
                self.opened(path, "reference_cst_error", ref["cst_error"])
            elif ref.get("cst"):
                candidates = [ref["cst"]]
        elif spans and "nodes" in spans:
            candidates = spans["nodes"] if index is None else spans["nodes"][index:index + 1]
        elif literal:
            self.opened(path, "property_cst_unavailable", spans or ABSENT)
        if lexeme is not None:
            self.validate_lexeme(path, lexeme)
            if len(candidates) == 1:
                self.match_span(path, candidates[0], lexeme)
            else:
                self.opened(path, "datum_cst_ambiguous_or_absent", candidates)
        elif literal and candidates:
            self.diff(path, "datum_lexeme_missing", candidates, None)

    def containers(self, path, fields, node):
        expected = None
        if "$container" in fields:
            parent = self.obj(fields["$container"])
            if not parent or parent["id"] not in self.npaths:
                self.opened(path, "native_container_unmapped", fields["$container"])
            expected = dict(parent=self.npaths.get(parent["id"]) if parent else ABSENT,
                            property=fields.get("$containerProperty", ABSENT),
                            index=dict(present="$containerIndex" in fields, value=fields.get("$containerIndex")) if "$containerIndex" in fields else dict(present=False))
        elif "$containerProperty" in fields or "$containerIndex" in fields:
            self.opened(path, "orphan_container_metadata", fields)
        actual = node.get("container")
        if actual is not None:
            actual = dict(parent=self.rpaths.get(actual["node"], "UNMAPPED:" + str(actual["node"])),
                          property=actual["property"], index=dict(present=actual["index"] is not None, value=actual["index"]) if actual["index"] is not None else dict(present=False))
        if not same(expected, actual):
            self.diff(path, "container", expected, actual)
        if "$childIndex" in fields:
            if "$childIndex" not in node["fields"]:
                self.opened(path + "/$childIndex", "native_child_index_not_exposed", fields["$childIndex"])
        for key in fields:
            if key.startswith("$") and key not in META and key != "$children":
                self.opened(path + "/" + pointer(key), "unclassified_metadata", fields[key])
        for key in ["$meta", "$document"]:
            if key in fields:
                self.opened(path + "/" + key, "external_metadata_not_qualified", fields[key])

    def run(self):
        for label, capture in [("native", self.native), ("rust", self.rust), ("references", self.expected)]:
            if not same(self.spec["source"], capture.get("source", ABSENT)):
                self.diff("root", label + "_source", self.spec["source"], capture.get("source", ABSENT))
        validate_rust_capture(self.rust)
        validate_reference_indices(self.expected)
        outcome = self.rust["outcome"]
        if self.spec.get("category") != "finite":
            error_class = outcome.get("error", "").split(" ")[0].split("(")[0]
            if outcome["kind"] != "rust_error" or error_class != self.spec.get("expected_restriction"):
                self.diff("root", "declared_rust_restriction", self.spec.get("expected_restriction"), outcome)
            self.opened("root", "diagnostic_and_domain_disposition_open", dict(native_lexer_errors=self.native.get("lexer_errors"),
                native_parser_errors=self.native.get("parser_errors"), rust=outcome))
            return self.result()
        if self.native.get("parse_exception") or self.native.get("lexer_errors") or self.native.get("parser_errors"):
            self.diff("root", "native_finite_parse_failure", self.native.get("parse_exception") or self.native.get("lexer_errors") or self.native.get("parser_errors"))
        if outcome["kind"] != "arena":
            self.diff("root", "finite_rust_refusal", "arena", outcome)
            return self.result()
        if self.expected.get("capture_error"):
            self.opened("root", "original_expectation_capture_error", self.expected["capture_error"])
        if self.expected.get("lexer_error_count") != len(self.native.get("lexer_errors") or []) or self.expected.get("parser_error_count") != len(self.native.get("parser_errors") or []):
            self.opened("root", "native_recapture_diagnostics_changed", self.expected)
        self.objects = {o["id"]: o for o in self.native["tree"]["objects"]}
        self.csts = {o["id"]: o for o in self.native["tree"]["cst"]}
        self.rnodes = {n["id"]: n for n in outcome["nodes"]}
        if len(self.objects) != len(self.native["tree"]["objects"]) or any(type(i) is not str for i in self.objects):
            raise ValueError("duplicate or invalid native object ids")
        if len(self.csts) != len(self.native["tree"]["cst"]) or any(type(i) is not str for i in self.csts):
            raise ValueError("duplicate or invalid native CST ids")
        if len(self.rnodes) != len(outcome["nodes"]) or any(type(i) is not int or i < 0 for i in self.rnodes) or type(outcome["root"]) is not int:
            raise ValueError("duplicate or invalid public Rust node ids")
        self.references = {(r["owner_path"], r["property"], r.get("index")): r for r in self.expected["references"]}
        self.spans = {(r["owner_path"], r["property"]): r for r in self.expected["property_spans"]}
        self.metadata = {r["path"]: r for r in self.expected["metadata"]}
        if len(self.references) != len(self.expected["references"]) or len(self.spans) != len(self.expected["property_spans"]) or len(self.metadata) != len(self.expected["metadata"]):
            raise ValueError("duplicate original expectation identity")
        self.walk_native(self.native["tree"]["root"], "root")
        self.walk_rust(dict(kind="node", id=outcome["root"]), "root")
        if not same(self.expected.get("aliases", ABSENT), self.naliases):
            self.opened("root", "native_recapture_aliases_changed", self.expected.get("aliases", ABSENT))
        if set(self.metadata) != set(self.nby):
            self.opened("root", "native_metadata_path_coverage", dict(
                missing=sorted(set(self.nby) - set(self.metadata)), extra=sorted(set(self.metadata) - set(self.nby))))
        span_keys = {(path, key) for path, obj in self.nby.items() for key in self.props(obj)
                     if not key.startswith("$") or key == "$children"}
        if set(self.spans) != span_keys:
            self.opened("root", "native_property_span_path_coverage", dict(
                missing=sorted(span_keys - set(self.spans)), extra=sorted(set(self.spans) - span_keys)))
        for (owner, key), span in self.spans.items():
            if "error" in span:
                self.opened(owner + "/" + pointer(key), "property_cst_helper_error", span["error"])
        if not same(self.naliases, self.raliases):
            self.diff("root", "structural_aliases", self.naliases, self.raliases)
        if set(self.rnodes) != set(self.rpaths):
            self.opened("root", "unmapped_public_rust_nodes", sorted(set(self.rnodes) - set(self.rpaths)))
        for path in sorted(set(self.nby) | set(self.rby)):
            native, rust = self.nby.get(path), self.rby.get(path)
            if native is None or rust is None:
                self.diff(path, "node_presence", native or ABSENT, rust or ABSENT)
                continue
            fields = self.props(native)
            if not same(native["node_type"], rust["node_type"]):
                self.diff(path, "node_type", native["node_type"], rust["node_type"])
            for key in sorted((set(fields) - META) | set(rust["fields"])):
                if key not in fields or key not in rust["fields"]:
                    self.diff(path + "/" + pointer(key), "field_presence", fields.get(key, ABSENT), rust["fields"].get(key, ABSENT))
                    continue
                nv = self.native_value(fields[key], path, key)
                rv = self.rust_value(rust["fields"][key]["value"])
                if not same(nv, rv):
                    self.diff(path + "/" + pointer(key), "field_value", nv, rv)
                self.datum_lexeme(path, key, rust["fields"][key], fields[key])
            self.containers(path, fields, rust)
            cst = self.csts.get(fields.get("$cstNode", {}).get("$cst"))
            if rust.get("origin") is None:
                self.diff(path, "node_origin_missing", cst, None)
            else:
                self.validate_lexeme(path, rust["origin"])
                self.match_span(path, cst.get("span") if cst else None, rust["origin"])
            meta = self.metadata.get(path)
            if not meta:
                self.opened(path, "native_metadata_expectations_missing", native["node_type"])
                continue
            if not same(native["property_keys"], meta["own_keys"]) or not same(native["node_type"], meta["node_type"]):
                self.diff(path, "native_recapture_inventory", native["property_keys"], meta)
            child_index = meta.get("child_index", ABSENT)
            previous = dict(present=True, descriptor_kind="data", value=fields["$childIndex"]) if "$childIndex" in fields else dict(present=False)
            if not same(previous, child_index):
                self.opened(path, "native_child_index_recapture", dict(previous=previous, current=child_index))
            for mandatory in meta["mandatory"]:
                key, kind = mandatory["name"], mandatory["type"]
                nv = self.native_value(fields.get(key, ABSENT), path, key)
                rv = rust["fields"].get(key, {}).get("value", ABSENT)
                if kind not in {"array", "boolean"}:
                    self.opened(path, "unknown_mandatory_kind", mandatory)
                elif nv.get("kind") != kind or rv.get("kind") != kind:
                    self.diff(path + "/" + pointer(key), "mandatory_kind", dict(required=kind, value=nv), rv)
        for key in set(self.references) - self.used_references:
            self.opened(key[0], "reference_not_compared", self.references[key])
        self.opened("root", "cst_graph_and_grammar_context_open", "Origins/property/reference spans checked; complete CST topology, range/feature/retarget identities and nonexposed metadata have no equivalent public Rust capture.")
        self.opened("root", "javascript_descriptor_and_owner_lifecycle_open", "Own-key inventories are retained and recaptured; JS descriptor flags/property insertion order and private arena-owner lifecycle are not exposed by equivalent public Rust fields.")
        return self.result()

    def result(self):
        return dict(id=self.spec["id"], category=self.spec.get("category"), differences=self.differences,
                    open_items=self.open_items, native_node_paths=sorted(self.nby), rust_node_paths=sorted(self.rby))


def compare(corpus, native, rust, references):
    ids = [c["id"] for c in validate_corpus(corpus)]
    for capture in [native, rust, references]:
        require(isinstance(capture, dict) and isinstance(capture.get("cases"), list), "capture cases array")
        require(all(isinstance(c, dict) for c in capture["cases"]), "capture case object")
        require(same([c.get("id", ABSENT) for c in capture["cases"]], ids),
                "missing, extra, duplicate or reordered capture case")
    rows = []
    for values in zip(corpus["cases"], native["cases"], rust["cases"], references["cases"]):
        case = Case(*values)
        try:
            rows.append(case.run())
        except Exception as error:
            case.opened("root", "comparison_capture_schema_failure", dict(type=type(error).__name__, message=str(error)))
            rows.append(case.result())
    return rows


def controls():
    # Entire case comparison, not just a scalar equality helper.
    def obj(ident, kind, fields, node_type=None):
        row = dict(id=ident, kind=kind, property_keys=list(fields), properties=[dict(key=k, value=v) for k, v in fields.items()])
        if node_type:
            row["node_type"] = node_type
        return row
    ref = lambda ident: {"$object": ident}
    datum = lambda value, lexeme=None: dict(value=value, lexeme=lexeme)
    span = dict(bytes=[0, 1], utf16=[0, 1], text="x")
    objects = [obj("r", "ast", {"$type":"Namespace", "children":ref("a"), "$children":ref("a"), "flag":True, "$cstNode":{"$cst":"c"}}, "Namespace"),
        obj("a", "array", {"0":ref("n"), "1":ref("m"), "length":2}),
        obj("n", "ast", {"$type":"TypeReference", "parts":ref("p"), "$children":ref("z"), "$container":ref("r"), "$containerProperty":"children", "$containerIndex":0, "$cstNode":{"$cst":"c"}}, "TypeReference"),
        obj("m", "ast", {"$type":"Package", "$children":ref("z"), "$container":ref("r"), "$containerProperty":"children", "$containerIndex":1, "$cstNode":{"$cst":"c"}}, "Package"),
        obj("p", "array", {"0":ref("q"), "length":1}), obj("q", "reference", {"$refText":"x"}), obj("z", "array", {"length":0})]
    nodes = [dict(id=0, node_type="Namespace", origin=span, container=None, fields={"flag":datum(dict(kind="boolean",value=True)),
        "children":datum(dict(kind="array",items=[datum(dict(kind="node",id=1)),datum(dict(kind="node",id=2))])),
        "$children":datum(dict(kind="array",items=[datum(dict(kind="node",id=1)),datum(dict(kind="node",id=2))]))}),
        dict(id=1,node_type="TypeReference",origin=span,container=dict(node=0,property="children",index=0),fields={"parts":datum(dict(kind="array",items=[datum(dict(kind="unresolved_reference",text="x",target_type="Type"),span)])),"$children":datum(dict(kind="array",items=[]))}),
        dict(id=2,node_type="Package",origin=span,container=dict(node=0,property="children",index=1),fields={"$children":datum(dict(kind="array",items=[]))})]
    spec=dict(id="control",source="x",category="finite")
    native=dict(id="control",source="x",lexer_errors=[],parser_errors=[],tree=dict(root=ref("r"),objects=objects,cst=[dict(id="c",span=dict(offset=0,end=1,text="x"))]))
    rust=dict(id="control",source="x",outcome=dict(kind="arena",root=0,nodes=nodes))
    expected=dict(id="control",source="x",aliases=[],lexer_error_count=0,parser_error_count=0,references=[dict(owner_path="root/children/0",property="parts",index=0,ref_text="x",target_type="Type",cst=dict(offset=0,end=1,text="x"))],property_spans=[],metadata=[dict(path=p,node_type=o["node_type"],own_keys=o["property_keys"],mandatory=[]) for p,o in [("root",objects[0]),("root/children/0",objects[2]),("root/children/1",objects[3])]])
    def pipeline(n, r, e):
        return compare(dict(schema=1,cases=[spec]),dict(cases=[n]),dict(cases=[r]),dict(cases=[e]))[0]
    baseline=pipeline(native,rust,expected)
    require(not baseline["differences"], "divergence-control baseline differs: " + str(baseline))
    mutations = [
        ("bool-type", lambda n,r,e: r["outcome"]["nodes"][0]["fields"]["flag"]["value"].update(value=1)),
        ("cache-alias", lambda n,r,e: r["outcome"]["nodes"][0]["fields"]["$children"]["value"]["items"].reverse()),
        ("array-order", lambda n,r,e: r["outcome"]["nodes"][0]["fields"]["children"]["value"]["items"].reverse()),
        ("container-index", lambda n,r,e: r["outcome"]["nodes"][1]["container"].update(index=99)),
        ("origin", lambda n,r,e: r["outcome"]["nodes"][1]["origin"].update(utf16=[0,0])),
        ("reference-type", lambda n,r,e: r["outcome"]["nodes"][1]["fields"]["parts"]["value"]["items"][0]["value"].update(target_type="Element")),
        ("datum-lexeme", lambda n,r,e: r["outcome"]["nodes"][1]["fields"]["parts"]["value"]["items"][0]["lexeme"].update(bytes=[0,0])),
        ("absent-field", lambda n,r,e: r["outcome"]["nodes"][0]["fields"].pop("flag")),
    ]
    passed=[]
    for name, mutate in mutations:
        n,r,e=copy.deepcopy((native,rust,expected)); mutate(n,r,e)
        result=pipeline(n,r,e)
        require(bool(result["differences"]) or any(o["kind"] == "comparison_capture_schema_failure" for o in result["open_items"]),
                "divergence control was accepted: " + name)
        passed.append(name)
    integrity = []
    def reject(name, action):
        try:
            action()
        except (ValueError, TypeError, KeyError):
            integrity.append(name)
            return
        raise ValueError("integrity control was accepted: " + name)
    reject("empty-corpus", lambda: compare(dict(schema=1,cases=[]),dict(cases=[]),dict(cases=[]),dict(cases=[])))
    reject("truncated-native-cases", lambda: compare(dict(schema=1,cases=[spec]),dict(cases=[]),dict(cases=[rust]),dict(cases=[expected])))
    bad=copy.deepcopy(spec);bad["category"]="unknown"
    reject("invalid-category", lambda: validate_corpus(dict(schema=1,cases=[bad])))
    reject("duplicate-case-ID", lambda: validate_corpus(dict(schema=1,cases=[spec,spec])))
    reject("missing-assigned-cases", lambda: validate_corpus(dict(schema=1,cases=[spec]), assigned=True))
    assigned=[dict(id=i,category=c,source="",**({"expected_restriction":"Malformed"} if c=="malformed" else
              {"expected_restriction":"Unsupported"} if c=="outside-whitelist" else {})) for i,c in REQUIRED_CASES]
    validate_corpus(dict(schema=1,cases=assigned), assigned=True)
    reject("reordered-assigned-cases", lambda: validate_corpus(dict(schema=1,cases=list(reversed(assigned))), assigned=True))
    reject("duplicate-JSON-key", lambda: strict_loads('{"cases":[],"cases":[1]}'))
    reject("truncated-JSON", lambda: strict_loads('{"cases":['))
    reject("nonfinite-JSON", lambda: strict_loads('{"number":NaN}'))
    broken=copy.deepcopy(rust);broken["outcome"]["nodes"][0].pop("fields")
    result=pipeline(native,broken,expected)
    require(any(o["kind"]=="comparison_capture_schema_failure" for o in result["open_items"]), "truncated Rust schema was accepted")
    integrity.append("truncated-Rust-fields")
    recapture=copy.deepcopy(expected);recapture["aliases"].append(dict(path="extra",canonical="root"))
    result=pipeline(native,rust,recapture)
    require(any(o["kind"]=="native_recapture_aliases_changed" for o in result["open_items"]), "changed recapture aliases were accepted")
    integrity.append("recapture-aliases")
    schema_mutations = [
        ("bool-child-handle", lambda r,e: r["outcome"]["nodes"][0]["fields"]["children"]["value"]["items"][0]["value"].update(id=True)),
        ("bool-container-handle", lambda r,e: r["outcome"]["nodes"][1]["container"].update(node=False)),
        ("bool-container-index", lambda r,e: r["outcome"]["nodes"][1]["container"].update(index=False)),
        ("missing-root-container", lambda r,e: r["outcome"]["nodes"][0].pop("container")),
        ("missing-default-Datum-lexeme", lambda r,e: r["outcome"]["nodes"][0]["fields"]["flag"].pop("lexeme")),
        ("bool-reference-index", lambda r,e: e["references"][0].update(index=False)),
    ]
    for name, mutate in schema_mutations:
        r,e=copy.deepcopy((rust,expected));mutate(r,e)
        result=pipeline(native,r,e)
        require(any(o["kind"] == "comparison_capture_schema_failure" for o in result["open_items"]),
                "typed schema control was accepted: " + name)
        integrity.append(name)
    return dict(status="PASS_PIPELINE_DIVERGENCE_CONTROLS", cases=passed, integrity_cases=integrity)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ["corpus","native","rust","references","out"]:
        parser.add_argument("--"+name,type=Path,required=True)
    parser.add_argument("--self-test",action="store_true")
    args=parser.parse_args()
    if not args.out.is_absolute() or not args.out.resolve().is_relative_to(Path("/workspace")):
        parser.error("new output must be an absolute /workspace path")
    report=dict(status="OPEN_REQUIRED_QUALIFICATION",case_results=[],differences=[],open_items=[])
    try:
        paths=[args.corpus,args.native,args.rust,args.references]
        documents=[strict_loads(p.read_text(encoding="utf-8")) for p in paths]
        corpus,native,rust,references=documents
        validate_corpus(corpus, assigned=True)
        require(hashlib.sha256(args.corpus.read_bytes()).hexdigest() == CORPUS_SHA256,
                "assigned corpus bytes changed; review a new corpus before qualification")
        report["case_results"]=compare(corpus,native,rust,references)
        for key in ["bundle_sha256","corpus_sha256","node","v8","executable_sha256"]:
            if key not in native.get("provenance",{}) or key not in references.get("provenance",{}):
                report["open_items"].append(dict(kind="required_oracle_provenance_missing",key=key))
            elif not same(native["provenance"][key],references["provenance"][key]):
                report["open_items"].append(dict(kind="oracle_provenance_mismatch",key=key))
        if native.get("provenance",{}).get("bundle_sha256") != "47d8fe11fda1b58b6257463087f663be953c2ab6c2b5e1e2ccc44161ed5cd0bd":
            report["open_items"].append(dict(kind="pinned_bundle_mismatch"))
        digest=hashlib.sha256(args.corpus.read_bytes()).hexdigest()
        if native.get("provenance",{}).get("corpus_sha256") != digest:
            report["open_items"].append(dict(kind="corpus_hash_mismatch",actual=digest))
        report["controls"]=controls() if args.self_test else dict(status="NOT_RUN")
        if not args.self_test:
            report["open_items"].append(dict(kind="pipeline_controls_not_run"))
        report["inputs"]={str(p.resolve()):hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
    except Exception as error:
        report["open_items"].append(dict(kind="qualification_input_or_control_failure",type=type(error).__name__,message=str(error)))
    for case in report["case_results"]:
        for kind in ["differences","open_items"]:
            report[kind].extend(dict(case=case["id"],**row) for row in case[kind])
    report["scope"]="All captured finite structural fields, aliases, containers, origins and source/CST-backed datum lexemes; original reference target methods. Named OPEN metadata/diagnostic/workflow boundaries remain required. No accepted feature or deployed-runtime claim."
    if not report["differences"] and not report["open_items"]:
        report["status"]="PASS_ASSIGNED_CAPTURE_QUALIFICATION_ONLY"
    with args.out.open("x") as output:
        json.dump(report,output,indent=2,ensure_ascii=True);output.write("\n")
    print(json.dumps(dict(status=report["status"],differences=len(report["differences"]),open_items=len(report["open_items"]))))
    raise SystemExit(0 if report["status"].startswith("PASS_") else 1)


if __name__ == "__main__":
    main()
