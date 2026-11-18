"""Bind released Actuator handlers to packaged JVM declarations without running Java."""
import argparse
import hashlib
import io
import json
import re
import subprocess
import warnings
import zipfile
from collections import Counter, defaultdict
from pathlib import Path

FIXTURE = "rust/tools/native-http-contract/tests/fixtures/native-mappings.json"
FIXTURE_SHA = "6bc55ce5f0b0161309acc4e1752f7778372bf43befd83c28aaf9d6f6e5aed28f"
PARENT = "9cb99468ca2d59eeae8965d66e52878611626216"
JAR_SHA = "85c031352695d6ac8f059f0215ca493c58a16805a111233998dd443d6990c9e1"
TARGET = "6ba2ac64e6b16ee2b05752ad55395eebb87d4910"
RELEASE = "b380dada415ce05155150f23cb3b74ae21cc45e4"
SIRIUS = "3191e2b8990373afee3415f6b26f8c5039a28321"
VERSION = "2026.9.3"
FALLBACK = "org.eclipse.sirius.web.infrastructure.configuration.mvc.FrontendRouterConfiguration"
CONFIG = "backend/application/syson-application/src/main/resources/"
MIB = 1024 * 1024


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def git(repo, *args):
    return subprocess.check_output(["git", "-C", str(repo), *args])


def member(archive, path, ceiling):
    matches = [row for row in archive.infolist() if row.filename == path]
    require(len(matches) == 1, f"expected exactly one archive member {path}")
    info = matches[0]
    require(0 < info.file_size <= ceiling, f"out-of-bounds archive member {path}")
    with archive.open(info) as stream:
        data = stream.read(ceiling + 1)
    require(len(data) == info.file_size, f"incomplete archive member {path}")
    return data


class ClassReader:
    def __init__(self, raw):
        self.stream = io.BytesIO(raw)
        self.pool = {}

    def read(self, size):
        data = self.stream.read(size)
        require(len(data) == size, "truncated class")
        return data

    def uint(self, size):
        return int.from_bytes(self.read(size), "big")

    def attributes(self):
        for _ in range(self.uint(2)):
            self.uint(2)
            self.read(self.uint(4))

    def declaration(self):
        self.uint(2)
        name, descriptor = self.pool[self.uint(2)], self.pool[self.uint(2)]
        self.attributes()
        return name, descriptor

    def methods(self):
        require(self.read(4) == b"\xca\xfe\xba\xbe", "class magic mismatch")
        self.read(4)
        count = self.uint(2)
        index = 1
        while index < count:
            tag = self.uint(1)
            if tag == 1:
                self.pool[index] = self.read(self.uint(2))
            elif tag in (7, 8, 16, 19, 20):
                self.pool[index] = self.uint(2)
            elif tag in (3, 4, 9, 10, 11, 12, 17, 18):
                self.read(4)
            elif tag in (5, 6):
                self.read(8)
                index += 1
            elif tag == 15:
                self.read(3)
            else:
                raise ValueError(f"unexpected class constant tag {tag}")
            index += 1
        self.uint(2)
        name = self.pool[self.pool[self.uint(2)]]
        self.uint(2)
        self.read(self.uint(2) * 2)
        for _ in range(self.uint(2)):
            self.declaration()
        methods = {self.declaration() for _ in range(self.uint(2))}
        self.attributes()
        require(not self.stream.read(1), "trailing class bytes")
        return name, methods


def check_methods(class_name, methods, routes):
    for route in routes:
        signature = route["method"].encode("ascii"), route["descriptor"].encode("ascii")
        require(signature in methods, f"missing declared method descriptor for {class_name}.{route['method']}")


def capture(raw):
    require(sha(raw) == FIXTURE_SHA, "Actuator fixture digest mismatch")
    rows = json.loads(raw)["contexts"]["application"]["mappings"]["dispatcherServlets"]["dispatcherServlet"]
    routes = defaultdict(list)
    for row in rows:
        handler = row.get("details", {}).get("handlerMethod")
        if handler is not None and handler["className"].startswith("org.eclipse.sirius."):
            routes[handler["className"]].append({
                "predicate": row["predicate"], "method": handler["name"], "descriptor": handler["descriptor"],
            })
    fallback = sum(row["handler"].startswith(FALLBACK + "$$Lambda/") for row in rows)
    require((len(rows), sum(map(len, routes.values())), len(routes), fallback) == (54, 25, 10, 1),
            "unexpected dispatcher coverage")
    return routes


def packaged(outer, artifact_id):
    path = f"BOOT-INF/lib/{artifact_id}-{VERSION}.jar"
    raw = member(outer, path, 32 * MIB)
    jar = zipfile.ZipFile(io.BytesIO(raw))
    coordinates = f"META-INF/maven/org.eclipse.sirius/{artifact_id}/pom.properties"
    props = dict(line.split("=", 1) for line in member(jar, coordinates, 2048).decode().splitlines()
                 if "=" in line and not line.startswith("#"))
    require(props == {"groupId": "org.eclipse.sirius", "artifactId": artifact_id, "version": VERSION},
            f"unexpected Maven coordinates for {artifact_id}")
    return ({"group_id": "org.eclipse.sirius", "artifact_id": artifact_id, "version": VERSION,
             "path": path, "sha256": sha(raw)}, jar)


def class_record(repo, names, class_name, module, raw, routes):
    stem = class_name.replace(".", "/")
    actual, methods = ClassReader(raw).methods()
    require(actual == stem.encode("ascii"), f"class entry/name mismatch for {class_name}")
    check_methods(class_name, methods, routes)
    paths = [name for name in names if name.endswith(f"/src/main/java/{stem}.java".encode())]
    require(len(paths) == 1, f"missing or ambiguous Sirius source for {class_name}")
    path = paths[0].decode()
    source = git(repo, "show", f"{SIRIUS}:{path}")
    for route in routes:
        pattern = rb"(?m)^\s*public[^\n]*\b" + re.escape(route["method"].encode("ascii")) + rb"\s*\("
        require(re.search(pattern, source) is not None, f"missing source declaration name in {path}")
    return {"name": class_name, "class_path": stem + ".class", "class_sha256": sha(raw),
            "jar_artifact_id": module, "source": {"path": path, "sha256": sha(source)}, "routes": routes}


def sys_on_source(repo, jar):
    paths = (CONFIG + "application.properties", CONFIG + "application-dev.properties",
             "backend/application/syson-sysml-import/src/main/java/org/eclipse/syson/sysml/upload/"
             "SysMLExternalResourceLoaderService.java", "frontend/syson/src/index.tsx", "pom.xml")
    result = []
    for path in paths:
        data = git(repo, "show", f"{TARGET}:{path}")
        require(data == git(repo, "show", f"{RELEASE}:{path}"), f"release and target sources differ: {path}")
        row = {"path": path, "source_sha256": sha(data)}
        if path.startswith(CONFIG):
            entry = "BOOT-INF/classes/" + path.rsplit("/", 1)[1]
            require(data == member(jar, entry, 64 * 1024), f"release resource differs: {path}")
            row["release_jar_member"] = entry
        result.append(row)
    delta = git(repo, "diff", "--name-only", "-z", TARGET, RELEASE).decode().split("\0")[:-1]
    groups = Counter("/".join(path.split("/")[:2]) for path in delta)
    require(groups == {"backend/metamodel": 104, "backend/views": 17, "doc/content": 3, "CHANGELOG.adoc": 1},
            "unexpected distribution of the target/release delta")
    return result, dict(groups)


def collect(args):
    require(args.jar.is_file() and args.jar.stat().st_size == 139_691_999, "release JAR size mismatch")
    with args.jar.open("rb") as stream:
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    require(digest == JAR_SHA, "release JAR digest mismatch")
    fixture = (Path(__file__).resolve().parents[4] / FIXTURE).read_bytes()
    routes = capture(fixture)
    names = git(args.sirius, "ls-tree", "-r", "--name-only", "-z", SIRIUS).split(b"\0")
    with zipfile.ZipFile(args.jar) as outer:
        modules = ("sirius-components-graphql", "sirius-web-application", "sirius-web-infrastructure")
        jars = {name: packaged(outer, name) for name in modules}
        records = []
        for name in sorted(routes):
            path = name.replace(".", "/") + ".class"
            matches = [module for module, (_, jar) in jars.items() if path in jar.namelist()]
            require(len(matches) == 1, f"expected unique controller in the inspected JARs: {name}")
            module = matches[0]
            raw = member(jars[module][1], path, MIB)
            records.append(class_record(args.sirius, names, name, module, raw, routes[name]))
        module = "sirius-web-infrastructure"
        fallback_raw = member(jars[module][1], FALLBACK.replace(".", "/") + ".class", MIB)
        fallback = class_record(args.sirius, names, FALLBACK, module, fallback_raw, [])
        configuration, groups = sys_on_source(args.syson, outer)
        for _, jar in jars.values():
            jar.close()
    observation = {
        "artifact_sha256": JAR_SHA, "target_sys_on_revision": TARGET,
        "released_sys_on_source_label": RELEASE, "sirius_source_tag_revision": SIRIUS,
        "actuator_fixture": {"sha256": FIXTURE_SHA, "parent_commit": PARENT,
                             "all_dispatcher_rows": 54, "bound_sirius_handler_method_rows": 25},
        "nested_release_jars": [entry for entry, _ in jars.values()],
        "declared_sirius_controller_methods": records,
        "frontend_fallback_enclosing_class": fallback,
        "sys_on_source_anchors": configuration,
        "target_release_changed_paths_by_directory": groups,
    }
    return observation, fixture, fallback_raw


def expect_refusal(action, message):
    try:
        action()
    except ValueError as error:
        require(message in str(error), f"unexpected refusal: {error}")
        return
    raise ValueError("negative control was accepted")


def controls(observation, fixture, binary):
    expect_refusal(lambda: capture(fixture + b"\n"), "fixture digest mismatch")
    expect_refusal(lambda: ClassReader(binary[:-1]).methods(), "truncated class")
    route = observation["declared_sirius_controller_methods"][0]["routes"][0]
    declared = {(route["method"].encode("ascii"), route["descriptor"].encode("ascii"))}
    expect_refusal(lambda: check_methods("control", declared, [{**route, "method": "absent"}]), "missing declared")
    expect_refusal(lambda: check_methods("control", declared, [{**route, "descriptor": "()V"}]), "missing declared")
    data = io.BytesIO()
    with warnings.catch_warnings(), zipfile.ZipFile(data, mode="w") as archive:
        warnings.simplefilter("ignore", UserWarning)
        archive.writestr("same", b"first")
        archive.writestr("same", b"second")
    with zipfile.ZipFile(data) as archive:
        for name in ("missing", "same"):
            expect_refusal(lambda: member(archive, name, 64), "expected exactly one")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", type=Path, required=True, help="official SysON release JAR")
    parser.add_argument("--syson", type=Path, required=True, help="Git checkout with both pinned SysON revisions")
    parser.add_argument("--sirius", type=Path, required=True, help="Git checkout with the pinned Sirius Web revision")
    parser.add_argument("--emit-observation", action="store_true", help="emit independently derived JSON")
    args = parser.parse_args()
    observation, fixture, binary = collect(args)
    controls(observation, fixture, binary)
    projection = (json.dumps(observation, indent=2, sort_keys=True) + "\n").encode()
    if args.emit_observation:
        print(projection.decode(), end="")
    else:
        reference = Path(__file__).with_name("observation.json").read_bytes()
        require(projection == reference, "committed observation differs from replay")
        print("PASS: 25 exact Sirius method descriptors, 10 classes, five source anchors, six negative controls")


if __name__ == "__main__":
    main()
