"""Verify a passive release repository and SQL join projection."""
import argparse
from copy import deepcopy
from hashlib import file_digest, sha256
from io import BytesIO
import json
from pathlib import Path, PurePosixPath
import re
from xml.etree import ElementTree
from zipfile import ZipFile

HERE = Path(__file__).resolve().parent
RELEASE = "85c031352695d6ac8f059f0215ca493c58a16805a111233998dd443d6990c9e1"
F26 = "caf7b4aedeba3a38dca32629620c11df94fd3fd31dda2bb195492d9f7c06548c"
B06 = "6de726b58d39c950830e3d2dbd4b101d5385832347e0446ff3ac80af3b7cc72f"
JOIN_SQL = "4123f03e36b9f14306b06a55ce207af5d812526234170d3c982a9397084bc2a7"
SEALED = "6a943006c8e10bfc597dc21487f1e51932cd3d1fb884219418d679b7c5ce1b3c"
APP = "sirius-web-application"
DOMAIN = "sirius-web-domain"
PROJECT = "sirius-web-projects-semanticdata-domain"
COLLAB = "sirius-components-collaborative"
INFRA = "sirius-web-infrastructure"
CLASSES = {
    APP: {
        "EditingContextPersistenceService": {"persist"},
        "RepresentationPersistenceService": {"save"},
        "RepresentationMetadataPersistenceService": {"save"},
        "RepresentationPersistenceStrategy": set(),
    },
    DOMAIN: {
        "Document": set(),
        "SemanticData": {"updateDocuments"},
        "RepresentationContent": {"updateContent", "updateMigrationData"},
        "RepresentationMetadata": set(),
        "RepresentationCompositeIdProvider": {"getId"},
    },
    PROJECT: {"ProjectSemanticData": set()},
    COLLAB: {"EditingContextSaver": set()},
}
FIELD_CLASSES = {"Document", "SemanticData", "RepresentationContent", "RepresentationMetadata", "ProjectSemanticData"}
ANNOTATIONS = {"Transactional", "Table", "Id", "Column", "MappedCollection", "Version"}
NAMED_CALLS = {"IEditingContextPersistenceService": "persist", "IRepresentationPersistenceService": "save"}
TABLES = {"document", "semantic_data", "project_semantic_data", "representation_content", "representation_metadata"}
UNKNOWN = ["native compare-and-swap token", "single application transaction across both lanes",
           "native storage mapping for six independent stamps", "exact target build and activation"]


def require(ok, reason):
    if not ok:
        raise ValueError(reason)


def sha(raw):
    return sha256(raw).hexdigest()


def compact(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def read_pinned(path, expected, maximum):
    with path.open("rb") as stream:
        raw = stream.read(maximum + 1)
    require(len(raw) <= maximum and sha(raw) == expected, "pinned input differs")
    return raw


class Reader:
    """The fields needed for exact classfile metadata, without executing a class."""

    def __init__(self, data):
        self.data = data
        self.offset = 0

    def take(self, size):
        start = self.offset
        self.offset += size
        require(self.offset <= len(self.data), "truncated classfile")
        return self.data[start:self.offset]

    def integer(self, size):
        return int.from_bytes(self.take(size), "big")


def constant_pool(reader):
    bound = reader.integer(2)
    require(1 <= bound <= 8192, "constant pool count")
    pool = {}
    index = 1
    while index < bound:
        tag = reader.integer(1)
        if tag == 1:
            value = reader.take(reader.integer(2)).decode(errors="replace")
        elif tag in {3, 4, 5, 6}:
            value = reader.take(8 if tag in {5, 6} else 4).hex()
        elif tag in {7, 8, 16, 19, 20}:
            value = reader.integer(2)
        elif tag in {9, 10, 11, 12, 17, 18}:
            value = (reader.integer(2), reader.integer(2))
        elif tag == 15:
            value = reader.take(3).hex()
        else:
            raise ValueError("unknown constant pool tag")
        pool[index] = (tag, value)
        index += 2 if tag in {5, 6} else 1
    return pool


def class_projection(raw, name, wanted_methods):
    reader = Reader(raw)
    require(reader.integer(4) == 0xCAFEBABE, "classfile signature differs")
    reader.take(4)
    pool = constant_pool(reader)

    def text(index):
        require(pool[index][0] == 1, "expected UTF-8 constant")
        return pool[index][1]

    def annotations(raw_annotations):
        cursor = Reader(raw_annotations)

        def one():
            descriptor = text(cursor.integer(2))
            pairs = {}
            for _ in range(cursor.integer(2)):
                element = text(cursor.integer(2))
                pairs[element] = value()
            return descriptor, pairs

        def value():
            tag = chr(cursor.integer(1))
            if tag in "BCDFIJSZsc":
                return pool[cursor.integer(2)][1]
            if tag == "e":
                return [text(cursor.integer(2)), text(cursor.integer(2))]
            if tag == "@":
                return one()
            if tag == "[":
                return [value() for _ in range(cursor.integer(2))]
            raise ValueError("unknown annotation value")

        rows = []
        for _ in range(cursor.integer(2)):
            descriptor, pairs = one()
            label = descriptor.rsplit("/", 1)[-1].removesuffix(";")
            if label in ANNOTATIONS:
                rows.append({"type": descriptor, "values": pairs})
        require(cursor.offset == len(raw_annotations), "trailing annotations")
        return rows

    def attributes():
        declared = []
        for _ in range(reader.integer(2)):
            tag = text(reader.integer(2))
            payload = reader.take(reader.integer(4))
            if tag in {"RuntimeVisibleAnnotations", "RuntimeInvisibleAnnotations"}:
                declared.extend(annotations(payload))
        return declared

    def members():
        rows = []
        for _ in range(reader.integer(2)):
            reader.integer(2)
            member = text(reader.integer(2))
            signature = text(reader.integer(2))
            rows.append({"name": member, "descriptor": signature, "annotations": attributes()})
        return rows

    reader.take(6)
    reader.take(2 * reader.integer(2))
    fields, methods, class_annotations = members(), members(), attributes()
    require(reader.offset == len(raw), "trailing classfile data")
    methods = [m for m in methods if m["name"] in wanted_methods]
    require({m["name"] for m in methods} == wanted_methods, "selected method is missing")
    named_references = set()
    for tag, indices in pool.values():
        if tag in {10, 11}:
            owner, signature = indices
            owner_name = text(pool[owner][1]).rsplit("/", 1)[-1]
            member_name = text(pool[signature][1][0])
            if NAMED_CALLS.get(owner_name) == member_name:
                named_references.add(owner_name + "." + member_name)
    markers = [value for tag, value in pool.values()
               if tag == 1 and value in {"SEMANTIC_CHANGE", "existsById", "\u0001#\u0001"}]
    return {"sha256": sha(raw), "annotations": class_annotations, "methods": methods,
            "fields": fields if name in FIELD_CLASSES else [],
            "named_method_references": sorted(named_references), "selected_constants": sorted(set(markers))}


def member(archive, name, limit):
    matches = [entry for entry in archive.infolist() if entry.filename == name and not entry.is_dir()]
    require(len(matches) == 1 and matches[0].file_size <= limit, "archive member missing or out of bounds")
    raw = archive.read(matches[0])
    require(len(raw) == matches[0].file_size, "archive member truncated")
    return raw


def local_name(tag):
    return tag.rsplit("}", 1)[-1]


def schema_projection(infra, history):
    paths, changes, columns, conversions = [], [], [], []

    def visit(path):
        require(path.startswith("db/") and path.endswith(".xml") and
                ".." not in PurePosixPath(path).parts and path not in paths and len(paths) < 40,
                "unadmitted changelog path")
        paths.append(path)
        root = ElementTree.fromstring(member(infra, path, 128 * 1024))
        for node in root:
            tag = local_name(node.tag)
            if tag == "include":
                value = node.attrib["file"]
                if node.attrib.get("relativeToChangelogFile") == "true":
                    value = str(PurePosixPath(path).parent / value)
                visit(value)
            elif tag == "changeSet":
                key = {"filename": path, "id": node.attrib["id"], "author": node.attrib["author"]}
                changes.append(key)
                for entry in node:
                    table = entry.attrib.get("tableName")
                    operation = local_name(entry.tag)
                    if table in TABLES and operation in {"createTable", "addColumn", "addNotNullConstraint"}:
                        if operation == "addNotNullConstraint":
                            values = [entry.attrib]
                        else:
                            values = [{"name": col.attrib.get("name"), "type": col.attrib.get("type"),
                                       "constraints": [c.attrib for c in col if local_name(c.tag) == "constraints"]}
                                      for col in entry if local_name(col.tag) == "column"]
                        columns.append({"change_id": key["id"], "table": table, "operation": operation, "columns": values})
                    if operation == "sql":
                        statement = " ".join((entry.text or "").split())
                        if re.match(r"ALTER TABLE representation_(?:metadata|content) ALTER COLUMN id TYPE TEXT", statement):
                            conversions.append(statement)

    visit("db/db.changelog-master.xml")
    applied = [{key: row[key] for key in ("filename", "id", "author")} for row in history]
    require(len(paths) == 29 and len(applied) == 20 and applied == changes, "release applied order differs")
    return {"resources": len(paths), "executed_changes": len(changes), "applied_identity_sha256": sha(compact(changes)),
            "selected_declarations": columns, "composite_id_conversions": conversions}


def artifact_projection(path, history):
    nested, classes = {}, {}
    with path.open("rb") as stream:
        require(file_digest(stream, "sha256").hexdigest() == RELEASE, "release JAR digest differs")
        stream.seek(0)
        with ZipFile(stream) as release:
            for module in (*CLASSES, INFRA):
                jar = member(release, f"BOOT-INF/lib/{module}-2026.9.3.jar", 24 * 1024 * 1024)
                nested[module] = sha(jar)
                with ZipFile(BytesIO(jar)) as inner:
                    if module == INFRA:
                        schema = schema_projection(inner, history)
                        continue
                    for cls, methods in CLASSES[module].items():
                        entries = [x for x in inner.namelist() if x.endswith("/" + cls + ".class")]
                        require(len(entries) == 1, "class entry is ambiguous")
                        classes[cls] = class_projection(member(inner, entries[0], 128 * 1024), cls, methods)
        stream.seek(0)
        require(file_digest(stream, "sha256").hexdigest() == RELEASE, "release JAR changed during read")
    return {"release_sha256": RELEASE, "nested_jars": nested, "classes": classes, "schema": schema}


def executed_join(path):
    raw = read_pinned(path, JOIN_SQL, 128 * 1024).decode()
    query = raw.split("CREATE TEMP VIEW selected_pair AS\n", 1)[1].split(";\nDO", 1)[0]
    columns, owners = query.removeprefix("SELECT ").split("\nFROM public.project AS p\n", 1)
    joins, predicates = owners.split("\nWHERE ", 1)
    fields = [part.strip() for part in columns.split(",")]
    edges = [" ".join(line.split()) for line in joins.splitlines()]
    matches = [" ".join(line.split()) for line in predicates.splitlines()]
    return {"source_sql_sha256": JOIN_SQL, "fields": fields, "root": "project AS p",
            "joins": edges, "predicates": [re.sub(r"'[^']*'", "'<fixture>'", item) for item in matches]}


def prior_data():
    f26 = json.loads(read_pinned(HERE.parent / "f26-release-changelog/observation.json", F26, 65536))
    rows = f26["observed"]
    require(all(r["exectype"] == "EXECUTED" and r["orderexecuted"] == i
                for i, r in enumerate(rows, start=1)), "prior executed order differs")
    b06 = json.loads(read_pinned(HERE.parent / "b06-paired-sql/observation.json", B06, 65536))
    require(b06["identity_check"]["compound_key_verified"] is True and
            b06["identity_check"]["metadata_uuid_matches_archive"] is True and
            b06["scope"]["application_transaction_exercised"] is False, "prior paired evidence differs")
    return rows


def verify(data):
    require(data["schema"] == "b06-release-repository-binding-v1" and
            data["release"]["release_sha256"] == RELEASE, "release or schema identity differs")
    require(set(data["release"]["nested_jars"]) == {*CLASSES, INFRA} and
            set(data["release"]["classes"]) == {cls for group in CLASSES.values() for cls in group},
            "module or class roster differs")
    classes = data["release"]["classes"]
    for cls, method in (("EditingContextPersistenceService", "persist"),
                        ("RepresentationPersistenceService", "save"),
                        ("RepresentationMetadataPersistenceService", "save")):
        exact = [row["annotations"] for row in classes[cls]["methods"] if row["name"] == method]
        require(exact == [[{"type": "Lorg/springframework/transaction/annotation/Transactional;", "values": {}}]],
                "transaction annotation differs")
    require(classes["EditingContextSaver"]["named_method_references"] ==
            ["IEditingContextPersistenceService.persist"]
            and "SEMANTIC_CHANGE" in classes["EditingContextSaver"]["selected_constants"]
            and classes["RepresentationPersistenceStrategy"]["named_method_references"] ==
            ["IRepresentationPersistenceService.save"]
            and "existsById" in classes["RepresentationPersistenceStrategy"]["selected_constants"]
            and "\u0001#\u0001" in classes["RepresentationCompositeIdProvider"]["selected_constants"],
            "caller references or compound identifier shape differs")
    for cls, table, fields in (
        ("Document", "document", {"id", "name", "content", "isReadOnly"}),
        ("SemanticData", "semantic_data", {"id", "documents"}),
        ("ProjectSemanticData", "project_semantic_data", {"id", "project", "semanticData"}),
        ("RepresentationContent", "representation_content",
         {"id", "representationMetadataId", "semanticData", "content", "lastMigrationPerformed", "migrationVersion"}),
        ("RepresentationMetadata", "representation_metadata",
         {"id", "representationMetadataId", "semanticData", "targetObjectId"}),
    ):
        record = classes[cls]
        require(fields.issubset({field["name"] for field in record["fields"]}) and
                any(a["type"].endswith("/Table;") and a["values"] == {"value": table}
                    for a in record["annotations"]), "repository table or field boundary differs")
    ddl = data["release"]["schema"]
    require(ddl["resources"] == 29 and ddl["executed_changes"] == 20 and
            len(ddl["composite_id_conversions"]) == 2 and
            all("CONCAT_WS('#', semantic_data_id::text, representation_metadata_id::text)" in sql
                for sql in ddl["composite_id_conversions"]), "release schema projection differs")
    join = data["executed_join"]
    require(join["source_sql_sha256"] == JOIN_SQL and join["root"] == "project AS p" and join["joins"] == [
        "JOIN public.project_semantic_data AS ps ON ps.project_id = p.id",
        "JOIN public.semantic_data AS s ON s.id = ps.semantic_data_id",
        "JOIN public.document AS d ON d.semantic_data_id = s.id",
        "JOIN public.representation_metadata AS rm ON rm.semantic_data_id = s.id",
        "JOIN public.representation_content AS rc ON rc.id = rm.id",
        "AND rc.semantic_data_id = rm.semantic_data_id",
        "AND rc.representation_metadata_id = rm.representation_metadata_id",
    ], "executed selected-row join differs")
    require({"d.id AS document_id", "d.semantic_data_id", "d.content AS document_content",
             "rm.representation_metadata_id", "rm.target_object_id", "rc.migration_version",
             "rc.last_migration_performed", "rc.content AS representation_content"}.issubset(join["fields"])
            and "AND d.name = '<fixture>' AND NOT d.is_read_only" in join["predicates"]
            and "AND rm.id = s.id::text || '<fixture>'" in join["predicates"], "row conditions differ")
    require(data["prior"] == {
        "changelog_head": "8d3df9e749592a2436a05661d1efb43b052f3bb7",
        "changelog_observation_sha256": F26,
        "sql_head": "8ce1ae50bc025214d0ad8e60691921c7fc61f6aa",
        "sql_observation_sha256": B06,
        "sql_result_sha256": "0ace28e511dbcbb91cd0365c34c6c06492be499f1876965fdaf16f30a1c25b4f",
        "paired_wire_head": "bad0efa31317ea24ec8ff155ef065d7679083d0d",
        "paired_wire_observation_sha256": "f16500c89ad4e80c97fb3fc57921975a49bf9d760e22cebc2b4c02977e1e03d7",
    }, "earlier source or activity evidence binding differs")
    require(data["unresolved"] == UNKNOWN and data["boundary"] == {
        "released_classfile_and_schema_observation": True,
        "new_sql_or_java_execution": False,
        "native_application_transaction_verified": False,
        "b06_implementation_admitted": False,
    }, "scope claims exceed the passive inspection")


def negative_controls(data):
    alterations = (
        (("release", "classes", "EditingContextPersistenceService", "methods", 0, "annotations"), []),
        (("release", "classes", "RepresentationPersistenceStrategy", "named_method_references"), []),
        (("release", "classes", "RepresentationCompositeIdProvider", "selected_constants"), []),
        (("release", "classes", "Document", "fields"), []),
        (("release", "schema", "executed_changes"), 19),
        (("release", "schema", "composite_id_conversions"), ["unrelated", "unrelated"]),
        (("executed_join", "joins", 6), "AND rc.representation_metadata_id = rm.semantic_data_id"),
        (("executed_join", "predicates"), []),
        (("prior", "paired_wire_head"), "0" * 40),
        (("unresolved", 0), "synthetic generation accepted as native"),
        (("unresolved", 2), "native stamp mapping accepted"),
        (("boundary", "native_application_transaction_verified"), True),
        (("boundary", "b06_implementation_admitted"), True),
    )
    for keys, changed in alterations:
        candidate = deepcopy(data)
        point = candidate
        for key in keys[:-1]:
            point = point[key]
        point[keys[-1]] = changed
        try:
            verify(candidate)
        except ValueError:
            continue
        raise ValueError(f"altered observation accepted at {keys}")
    return len(alterations)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", type=Path, help="exact official release archive")
    parser.add_argument("--observed-join", type=Path, help="exact retained SQL with read permission")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    data = json.loads(read_pinned(HERE / "observation.json", SEALED, 128 * 1024))
    verify(data)
    history = prior_data()
    if args.jar:
        require(artifact_projection(args.jar, history) == data["release"], "released artifact projection differs")
    if args.observed_join:
        require(executed_join(args.observed_join) == data["executed_join"], "retained SQL projection differs")
    suffix = f" and {negative_controls(data)} altered-record refusals" if args.self_test else ""
    print("PASS sealed B06 release repository binding" + suffix)


if __name__ == "__main__":
    main()
