#!/usr/bin/env python3
"""Read pinned Java/Ecore as text; write only this preparation directory.

This performs static evidence extraction, not Java/AQL/SysON execution. The
initializer/menu projections are labeled source interpretations, not an oracle.
SPDX-License-Identifier: EPL-2.0
"""
from pathlib import Path
import hashlib
import json
import re
import xml.etree.ElementTree as ET

OUT = Path(__file__).resolve().parent
REPO = OUT.parents[2]
BASELINE = "6ba2ac64e6b16ee2b05752ad55395eebb87d4910"
META = "backend/metamodel/syson-sysml-metamodel/src/main"
JAVA = META + "/java/org/eclipse/syson/sysml"
SERVICE = "backend/services/syson-sysml-metamodel-services/src/main/java/org/eclipse/syson/sysml/metamodel/services"
APP = "backend/application/syson-application-configuration/src/main/java/org/eclipse/syson/application/services"
COMMON = "backend/views/syson-diagram-common-view/src/main/java/org/eclipse/syson/diagram/common/view"
read_paths = set()

def read(path):
    read_paths.add(path)
    return (REPO / path).read_text()

def line(text, offset):
    return text.count("\n", 0, offset) + 1

def digest(text):
    return hashlib.sha256(text.encode()).hexdigest()

def save(name, data):
    (OUT / name).write_text(json.dumps({"baseline": BASELINE, "interpretation": "static_source_not_runtime", "data": data}, indent=2) + "\n")

def body_end(text, start):
    depth, state, index = 0, "code", start
    while index < len(text):
        ch = text[index]
        nxt = text[index:index + 2]
        if state in ('"', "'"):
            if ch == "\\": index += 2; continue
            if ch == state: state = "code"
        elif state == "line":
            if ch == "\n": state = "code"
        elif state == "comment":
            if nxt == "*/": state = "code"; index += 2; continue
        elif nxt == "//": state = "line"; index += 2; continue
        elif nxt == "/*": state = "comment"; index += 2; continue
        elif ch in ('"', "'"): state = ch
        elif ch == "{": depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0: return index + 1
        index += 1
    raise ValueError("unclosed source body")

METHOD = re.compile(r"(?m)^    (?:public|protected|private)\s+(?:static\s+)?(?:<[^>]+>\s+)?[\w.<>?, \[\]]+?\s+(\w+)\(([^)]*)\)\s*(?:throws\s+[^\{]+)?\{")
def methods(path):
    text = read(path)
    out = []
    for match in METHOD.finditer(text):
        start = match.end() - 1
        end = body_end(text, start)
        out.append({"name": match.group(1), "line": line(text, match.start()),
                    "signature": " ".join(text[match.start():start].split()),
                    "body": text[start + 1:end - 1], "body_sha256": digest(text[start:end])})
    return text, out

root = ET.fromstring(read(META + "/resources/model/sysml.ecore"))
xsi = "{http://www.w3.org/2001/XMLSchema-instance}type"
classes = {n.attrib["name"]: n for n in root.findall("eClassifiers") if n.attrib[xsi] == "ecore:EClass"}
enums = {n.attrib["name"]: n for n in root.findall("eClassifiers") if n.attrib[xsi] == "ecore:EEnum"}
assert len(classes) == 175 and len(enums) == 7

def local_supers(name):
    return [v[3:] for v in classes[name].attrib.get("eSuperTypes", "").split() if v.startswith("#//")]

def subtype(name, expected, seen=None):
    seen = set() if seen is None else seen
    if name == expected: return True
    if name in seen: return False
    seen.add(name)
    return any(subtype(parent, expected, seen) for parent in local_supers(name))

factory_path = JAVA + "/impl/SysmlFactoryImpl.java"
factory, factory_methods = methods(factory_path)
factory_cases = dict((kind, label) for label, kind in re.findall(r"case SysmlPackage\.([A-Z_]+):\s*return this.create(\w+)\(\);", factory))
raw_methods = {m["name"][6:]: m for m in factory_methods if m["name"].startswith("create") and m["signature"].endswith("()") and m["name"][6:] in classes}
abstract = [name for name, node in classes.items() if node.attrib.get("abstract") == "true"]
assert len(abstract) == 8 and len(factory_cases) == len(raw_methods) == 167
assert set(factory_cases) == set(classes) - set(abstract)
assert all("setElementId(ElementUtil.generateUUID(" in m["body"] for m in raw_methods.values())
assert all(m["body"].count(";") == 3 for m in raw_methods.values())

init_path = SERVICE + "/ElementInitializerSwitch.java"
init_text, init_methods = methods(init_path)
overrides = {m["name"][4:]: m for m in init_methods if m["name"].startswith("case")}
for name, method in overrides.items():
    method["return_mode"] = "null_fallthrough" if re.search(r"return (?:null|super\.case\w+\(object\));", method["body"]) else "nonnull_object"
    method["setters"] = re.findall(r"(?:object|\w+)\.set\w+\([^;]*\);", method["body"])
    method["explicit_case_calls"] = re.findall(r"this\.case(\w+)\(", method["body"])
    method["raw_created_types"] = re.findall(r"SysmlFactory\.eINSTANCE\.create(\w+)\(", method["body"])
    method["helper_calls"] = re.findall(r"this\.(?!case)(\w+)\(", method["body"])

switch_path = JAVA + "/util/SysmlSwitch.java"
switch = read(switch_path)
dispatch = {}
for match in re.finditer(r"case SysmlPackage\.([A-Z_]+):\s*\{", switch):
    start = match.end() - 1; end = body_end(switch, start)
    calls = re.findall(r"this\.case(\w+)\(", switch[start:end])
    dispatch[calls[0]] = {"switch_line": line(switch, match.start()), "case_order": calls}
assert set(dispatch) == set(classes)

class_rows, field_rows, attribute_rows, impl_index = [], [], [], []
for name, node in classes.items():
    path = JAVA + "/impl/" + name + "Impl.java"
    text, entries = methods(path)
    constructor = re.search(r"protected " + name + r"Impl\(\)\s*\{", text)
    assert constructor is not None
    end = body_end(text, constructor.end() - 1)
    ctor_body = text[constructor.end():end - 1].strip()
    assert ctor_body == "super();", (name, ctor_body)
    parent = re.search(r"class " + name + r"Impl\s+extends\s+(\w+)", text)
    for field in re.finditer(r"(?m)^    protected (?:(static final) )?([^;\n]+?) (\w+)(?:\s*=\s*([^;]+))?;", text):
        if "(" in field.group(2): continue
        field_rows.append({"class": name, "source": path, "line": line(text, field.start()), "constant": bool(field.group(1)), "java_type": field.group(2), "field": field.group(3), "initializer": field.group(4), "no_initializer_means": None if field.group(4) else "Java_zero_or_null_not_JSON_presence"})
    for feature in node.findall("eStructuralFeatures"):
        if feature.attrib.get(xsi) == "ecore:EAttribute":
            attribute_rows.append({"declared_by": name, **feature.attrib})
    impl_index.append({"class": name, "source": path, "java_superclass": parent.group(1) if parent else None, "constructor_line": line(text, constructor.start()), "constructor_body": ctor_body, "state_methods": [{k: m[k] for k in ("name", "line", "body_sha256")} for m in entries if m["name"] in ("eIsSet", "eUnset", "eSet", "eGet")]})
    observed = []
    for called in dispatch[name]["case_order"]:
        if called in overrides:
            observed.append(called)
            if overrides[called]["return_mode"] == "nonnull_object": break
    class_rows.append({"class": name, "abstract": node.attrib.get("abstract") == "true", "interface": node.attrib.get("interface") == "true", "raw_constructible": name in factory_cases, "factory_case": factory_cases.get(name), "factory_method_line": raw_methods.get(name, {}).get("line"), "factory_body_sha256": raw_methods.get(name, {}).get("body_sha256"), "local_supertypes": local_supers(name), "external_supertypes": [s for s in node.attrib.get("eSuperTypes", "").split() if not s.startswith("#//")], **dispatch[name], "initializer_override_projection": observed, "terminal_initializer": observed[-1] if observed else "defaultCase"})

save("class-factory-and-dispatch.json", class_rows)
save("initializer-overrides.json", [{k:v for k,v in m.items() if k != "body"} for m in overrides.values()])
save("initializer-helper-methods.json", [{k:m[k] for k in ("name", "line", "signature", "body_sha256")} for m in init_methods if not m["name"].startswith("case")])
save("java-fields-and-defaults.json", field_rows)
save("declared-ecore-attributes.json", attribute_rows)
save("implementation-state-methods.json", impl_index)

menu_path = APP + "/GetChildCreationSwitch.java"
menu_text, menu_methods_list = methods(menu_path)
menu_methods = {m["name"]: m for m in menu_methods_list}
concrete = [name for name in classes if name not in abstract]
def candidates(method):
    if method in ("caseNamespace", "casePackage"): return candidates("getPackageChildren")
    dynamic = []
    if method == "caseDefinition": dynamic = [c for c in concrete if subtype(c, "Usage") or subtype(c, "Import")]
    elif method == "caseFeatureMembership": dynamic = [c for c in concrete if subtype(c, "Feature")]
    elif method == "caseOwningMembership": dynamic = [c for c in concrete if not subtype(c, "Membership")]
    elif method == "getPackageChildren": dynamic = [c for c in concrete if any(subtype(c, t) for t in ("Definition", "Usage", "Import", "Package"))]
    result = list(dynamic)
    for match in re.finditer(r"childrenCandidates\.add\(SysmlPackage\.eINSTANCE\.get(\w+)\(\)\)|childrenCandidates\.addAll\(this\.(case\w+)\(object\)\)", menu_methods[method]["body"]):
        if match.group(1): result.append(match.group(1))
        else: result.extend(candidates(match.group(2)))
    return result
menu_rows = []
for name in classes:
    method = next(("case" + c for c in dispatch[name]["case_order"] if "case" + c in menu_methods), "defaultCase")
    menu_rows.append({"container_class": name, "menu_method": method, "method_line": menu_methods[method]["line"], "candidate_membership_set": sorted(set(candidates(method))), "candidate_count_including_duplicates": len(candidates(method)), "abstract_dummy_factory_rejects_entry": name in abstract, "ordering": "membership_set_only_service_sorts_display_labels"})
save("explorer-menu-candidates.json", menu_rows)
save("intermediate-container-switch.json", [{k:m[k] for k in ("name", "line", "signature", "body_sha256")} for m in methods("backend/services/syson-services/src/main/java/org/eclipse/syson/util/GetIntermediateContainerCreationSwitch.java")[1]])

# Inventory all main-source view tool providers and palette-section switches.
tool_paths = sorted(str(p.relative_to(REPO)) for p in (REPO / "backend/views").rglob("*.java") if "/src/main/java/" in str(p) and ("/tools/" in str(p) or "ToolSectionSwitch" in p.name or "ToolSectionNameSwitch" in p.name or "ToolDescriptionService" in p.name or "Palette" in p.name))
tools = []
for path in tool_paths:
    text, entries = methods(path)
    tools.append({"source": path, "methods": [{k:m[k] for k in ("name", "line", "signature", "body_sha256")} for m in entries], "service_method_refs": sorted(set(re.findall(r"(\w+)::(\w+)", text))), "class_refs": sorted(set(re.findall(r"SysmlPackage\.eINSTANCE\.get(\w+)\(", text))), "provider_refs": sorted(set(re.findall(r"new (\w+ToolProvider|\w+ToolNodeProvider)\(", text))), "string_literals": [{"line": line(text, m.start()), "java_literal": m.group()} for m in re.finditer(r'"(?:\\.|[^"\\])*"', text)]})
save("diagram-tool-source-index.json", tools)

# Supplemental application palette contributors and frontend command/menu
# adapters are outside backend/views but still affect creation availability.
application_tools = []
for path_obj in sorted((REPO / "backend/application").rglob("*PaletteToolsProvider.java")):
    if "/src/main/java/" not in str(path_obj): continue
    path = str(path_obj.relative_to(REPO)); text, entries = methods(path)
    application_tools.append({"source": path, "methods": [{k:m[k] for k in ("name", "line", "signature", "body_sha256")} for m in entries], "literal_tokens": [{"line": line(text, m.start()), "java_literal": m.group()} for m in re.finditer(r'"(?:\\.|[^"\\])*"', text)]})
save("application-palette-source-index.json", application_tools)
frontend_commands = []
for path_obj in sorted((REPO / "frontend").rglob("*")):
    if not path_obj.is_file() or path_obj.suffix not in (".ts", ".tsx") or "/src/" not in str(path_obj): continue
    if not re.search(r"Menu|Tool|Modal|useCreate|useInsert|ExtensionRegistry", path_obj.name): continue
    path = str(path_obj.relative_to(REPO)); text = read(path)
    frontend_commands.append({"source": path, "exported_symbols": re.findall(r"export (?:const|function|class) (\w+)", text), "graphql_mutation_names": sorted(set(re.findall(r"mutation (\w+)", text))), "literal_tokens": [{"line": line(text,m.start()), "token": m.group()} for m in re.finditer(r'"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\'', text)]})
save("frontend-command-source-index.json", frontend_commands)

# Inventory every non-generated main-source backend create method, including AQL
# adapters. Names/signatures and bodies are evidence references, not authorization.
commands = []
for path_obj in sorted((REPO / "backend").rglob("*.java")):
    path = str(path_obj.relative_to(REPO))
    if "/src/main/java/" not in path or path.startswith("backend/metamodel/"): continue
    text = path_obj.read_text()
    if not re.search(r"\bcreate\w*\s*\(", text): continue
    text, entries = methods(path)
    for method in entries:
        if method["name"].startswith("create"):
            commands.append({"source": path, **{k:method[k] for k in ("name", "line", "signature", "body_sha256")}, "raw_created_types": sorted(set(re.findall(r"SysmlFactory\.eINSTANCE\.create(\w+)\(", method["body"]))), "initializer_call_present": "initialize(" in method["body"] or "ElementInitializerSwitch" in method["body"]})
save("backend-create-method-index.json", commands)

extra_sources = [JAVA + "/util/ElementUtil.java", JAVA + "/impl/SysmlPackageImpl.java", APP + "/SysMLv2EditService.java", APP + "/SysONJsonResourceProcessor.java", APP + "/StandardLibrariesLoader.java", "backend/services/syson-services/src/main/java/org/eclipse/syson/util/SysmlEClassSwitch.java", SERVICE + "/MetamodelMutationElementService.java", SERVICE.replace("/main/", "/test/") + "/ElementInitializerSwitchTest.java", "backend/application/syson-application/src/test/java/org/eclipse/syson/metamodel/helper/ElementInitializerSwitchIntegrationTest.java", "backend/application/syson-application-configuration/src/test/java/org/eclipse/syson/application/services/GetChildCreationSwitchTest.java", JAVA + "/SysmlPackage.java", "backend/application/syson-application/src/test/java/org/eclipse/syson/application/controller/objects/ObjectCreationTests.java", "backend/application/syson-application/src/test/java/org/eclipse/syson/application/data/GeneralViewWithTopNodesTestProjectData.java", "backend/application/syson-application/src/test/resources/scripts/database-content/GeneralView-WithTopNodes.sql", SERVICE.replace("/main/", "/test/") + "/MetamodelMutationElementServiceTest.java"]
for path in extra_sources: read(path)
(OUT / "source-sha256.txt").write_text("".join(f"{hashlib.sha256((REPO/p).read_bytes()).hexdigest()}  {p}\n" for p in sorted(read_paths)))
summary = {"domain_classes": len(classes), "abstract_classes": abstract, "interfaces": [], "raw_factory_cases": len(factory_cases), "enum_types": len(enums), "constructor_bodies_checked": len(impl_index), "field_declarations": len(field_rows), "declared_ecore_attributes": len(attribute_rows), "initializer_override_count": len(overrides), "diagram_tool_source_files": len(tools), "application_palette_source_files": len(application_tools), "frontend_command_source_files": len(frontend_commands), "backend_create_methods": len(commands), "fingerprinted_source_files": len(read_paths)}
save("summary.json", summary)
print(json.dumps(summary, indent=2))
