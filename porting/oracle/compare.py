# SPDX-License-Identifier: EPL-2.0
"""Normalize two EMF JSON documents and diff them.

Normalization (every rule is reported, nothing is silently dropped):
  * generated `id` / `elementId` values become positional labels n0, n1, ...
    assigned in containment pre-order;
  * a string value that equals some node's `id` OR `elementId` is a reference
    and becomes ref(<label>). Which of the two it pointed at is kept as a
    separate "wire" finding, because the persisted format must use `id`;
  * the reference importer's root `sysml:Namespace` carrying the
    `org.eclipse.syson.sysml.imported` annotation is unwrapped for the
    structural diff, and its presence/absence is reported as a wire finding.
"""
IMPORTED = "org.eclipse.syson.sysml.imported"


def _walk(node, out):
    if isinstance(node, dict):
        if "eClass" in node and "id" in node:
            out.append(node)
        for v in node.values():
            _walk(v, out)
    elif isinstance(node, list):
        for v in node:
            _walk(v, out)


def _unwrap(content):
    """Return (roots, wrapped?) with the import Namespace removed."""
    if (len(content) == 1 and content[0].get("eClass") == "sysml:Namespace"
            and any(a.get("source") == IMPORTED for a in content[0]["data"].get("eAnnotations", []))):
        roots = []
        for rel in content[0]["data"].get("ownedRelationship", []):
            roots.extend(rel.get("data", {}).get("ownedRelatedElement", []))
        return roots, True
    return content, False


def normalize(document):
    roots, wrapped = _unwrap(document.get("content", []))
    nodes = []
    _walk(roots, nodes)
    by_id, by_element = {}, {}
    for i, n in enumerate(nodes):
        by_id[n["id"]] = f"n{i}"
        if "elementId" in n.get("data", {}):
            by_element[n["data"]["elementId"]] = f"n{i}"
    ref_kinds = set()

    def conv(value, key=None):
        if isinstance(value, dict):
            if "eClass" in value and "id" in value:
                data = {k: conv(v, k) for k, v in value.get("data", {}).items() if k != "elementId"}
                return {"#": by_id[value["id"]], "eClass": value["eClass"], "data": data}
            return {k: conv(v, k) for k, v in value.items()}
        if isinstance(value, list):
            return [conv(v, key) for v in value]
        if isinstance(value, str):
            if value in by_id:
                ref_kinds.add("id")
                return f"ref({by_id[value]})"
            if value in by_element:
                ref_kinds.add("elementId")
                return f"ref({by_element[value]})"
            # A reference to something outside this document (e.g. a library) stays opaque.
        return value

    return {"roots": [conv(r) for r in roots], "wrapped": wrapped,
            "reference_encoding": sorted(ref_kinds), "nodes": len(nodes)}


def diff(a, b, path="$", out=None, limit=40):
    out = [] if out is None else out
    if len(out) >= limit:
        return out
    if isinstance(a, dict) and isinstance(b, dict):
        if a.get("eClass") != b.get("eClass") and "eClass" in a and "eClass" in b:
            out.append({"path": path, "reference": a.get("eClass"), "rust": b.get("eClass")})
            return out
        for k in sorted(set(a) | set(b)):
            if k == "#":
                continue
            if k not in b:
                out.append({"path": f"{path}.{k}", "reference": _short(a[k]), "rust": "<absent>"})
            elif k not in a:
                out.append({"path": f"{path}.{k}", "reference": "<absent>", "rust": _short(b[k])})
            else:
                diff(a[k], b[k], f"{path}.{k}", out, limit)
    elif isinstance(a, list) and isinstance(b, list):
        for i in range(max(len(a), len(b))):
            if i >= len(a):
                out.append({"path": f"{path}[{i}]", "reference": "<absent>", "rust": _short(b[i])})
            elif i >= len(b):
                out.append({"path": f"{path}[{i}]", "reference": _short(a[i]), "rust": "<absent>"})
            else:
                diff(a[i], b[i], f"{path}[{i}]", out, limit)
    elif a != b:
        out.append({"path": path, "reference": _short(a), "rust": _short(b)})
    return out


def _short(v):
    if isinstance(v, dict) and "eClass" in v:
        name = v.get("data", {}).get("declaredName")
        return f"<{v['eClass']}{' ' + name if name else ''}>"
    if isinstance(v, (dict, list)):
        s = str(v)
        return s if len(s) <= 120 else s[:117] + "..."
    return v


def compare(reference_doc, rust_doc):
    ref, rust = normalize(reference_doc), normalize(rust_doc)
    structural = diff(ref["roots"], rust["roots"], "$")
    wire = []
    if ref["wrapped"] != rust["wrapped"]:
        wire.append(f"import root Namespace wrapper: reference={ref['wrapped']} rust={rust['wrapped']}")
    if ref["reference_encoding"] != rust["reference_encoding"]:
        wire.append(f"cross-references point at: reference={ref['reference_encoding']} "
                    f"rust={rust['reference_encoding']}")
    return {"structural_match": not structural, "structural_diffs": structural, "wire_diffs": wire,
            "reference_nodes": ref["nodes"], "rust_nodes": rust["nodes"]}
