# SPDX-License-Identifier: EPL-2.0
"""Exercise SysON's project ZIP upload and collect a single fixture re-export."""

import argparse
from hashlib import sha256
from io import BytesIO
import json
from pathlib import Path
from urllib.parse import urlparse
from urllib.request import Request, urlopen
from uuid import UUID, uuid4
from zipfile import ZipFile

FIXTURE_HASH = "96846d20a44dee2cf87e3643a7c6804ce0db0a13731aee013a1abaf40ab28012"
RELEASE_HASH = "85c031352695d6ac8f059f0215ca493c58a16805a111233998dd443d6990c9e1"
RELEASE_REVISION = "b380dada415ce05155150f23cb3b74ae21cc45e4"
UPLOAD = """mutation($input: UploadProjectInput!) { uploadProject(input: $input) {
  __typename ... on UploadProjectSuccessPayload { project { id name } }
  ... on ErrorPayload { messages { body level } } } }"""
PROJECTS = """{ viewer { projects(first: 10) {
  edges { node { id name } } pageInfo { hasNextPage } } } }"""


def require(condition, message):
    if not condition:
        raise ValueError(message)


def dump(path, data):
    path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def request(url, body=None, content_type=None, status=200):
    headers = {"Content-Type": content_type} if content_type else {}
    with urlopen(Request(url, data=body, headers=headers), timeout=120) as response:
        require(response.status == status, f"Unexpected HTTP status {response.status}")
        data = response.read(256 * 1024 + 1)
        require(len(data) <= 256 * 1024, "Response exceeded the fixture limit")
        return data, response.headers


def graphql(base, output, label, query=None, archive=None):
    if archive is None:
        body = json.dumps({"query": query}).encode()
        data, _ = request(base + "/api/graphql", body, "application/json")
    else:
        boundary = uuid4().hex
        operations = {"query": UPLOAD, "variables": {"input": {"id": str(uuid4()), "file": None}}}
        parts = [
            ('name="operations"', json.dumps(operations).encode()),
            ('name="map"', b'{"0":"variables.file"}'),
            ('name="0"; filename="ArchiveDemo.zip"\r\nContent-Type: application/zip', archive),
        ]
        body = b"".join(
            f"--{boundary}\r\nContent-Disposition: form-data; {header}\r\n\r\n".encode()
            + data + b"\r\n"
            for header, data in parts
        ) + f"--{boundary}--\r\n".encode()
        data, _ = request(
            base + "/api/graphql/upload", body, f"multipart/form-data; boundary={boundary}", status=201
        )
    (output / f"{label}.json").write_bytes(data)
    return json.loads(data)


def graphql_data(response):
    require(not response.get("errors"), f"Unexpected GraphQL execution error: {response}")
    return response["data"]


def projects(base, output, label):
    page = graphql_data(graphql(base, output, label, query=PROJECTS))["viewer"]["projects"]
    require(not page["pageInfo"]["hasNextPage"], f"{label}: project list is incomplete")
    return [edge["node"] for edge in page["edges"]]


def archive_without_manifest(blob):
    result = BytesIO()
    with ZipFile(BytesIO(blob)) as source, ZipFile(result, "w") as destination:
        for info in source.infolist():
            if info.filename != "ArchiveDemo/manifest.json":
                destination.writestr(info, source.read(info))
    return result.getvalue()


def capture(state, fixture, output):
    blob = fixture.read_bytes()
    require(sha256(blob).hexdigest() == FIXTURE_HASH, "Fixture bytes differ from the reviewed ZIP")
    require(
        (state["jar_source_revision"], state["jar_sha256"]) == (RELEASE_REVISION, RELEASE_HASH),
        "Expected the pinned SysON v2026.9.3 runtime",
    )
    base = state["url"]
    parsed = urlparse(base)
    require(
        parsed.scheme == "http" and parsed.netloc == f"127.0.0.1:{state['app_port']}"
        and not parsed.path and not parsed.query and not parsed.fragment,
        "Expected the loopback URL from oracle.py",
    )

    before = projects(base, output, "projects-before")
    require(not before, "The archive control needs a fresh oracle database")
    negative = archive_without_manifest(blob)
    (output / "archive-without-manifest.zip").write_bytes(negative)
    rejection = graphql(base, output, "upload-missing-manifest", archive=negative)
    errors = rejection.get("errors", [])
    require(
        rejection["data"] is None and len(errors) == 1
        and errors[0]["path"] == ["uploadProject"]
        and errors[0]["extensions"]["classification"] == "DataFetchingException",
        f"Unexpected response to the missing-manifest control: {rejection}",
    )
    require(projects(base, output, "projects-after-rejection") == before, "Failed upload added a project")

    payload = graphql_data(graphql(base, output, "upload-reviewed-fixture", archive=blob))["uploadProject"]
    require(payload["__typename"] == "UploadProjectSuccessPayload", f"Fixture upload failed: {payload}")
    project = payload["project"]
    project_id = str(UUID(project["id"]))
    require(project_id == project["id"], "The uploaded project ID is not a canonical UUID")
    require(project["name"] == "ArchiveDemo", "The uploaded project name changed")
    require(projects(base, output, "projects-after-fixture") == [project], "Expected one new project")

    archive, headers = request(f"{base}/api/projects/{project_id}")
    require(headers.get_content_type() == "application/zip", "Download must return application/zip")
    filename = headers.get_filename()
    require(filename == "ArchiveDemo.zip", f"Unexpected download filename: {filename}")
    (output / "archive-demo-reexport.syson.zip").write_bytes(archive)
    dump(output / "capture.json", {
        "release": "v2026.9.3",
        "release_source_revision": RELEASE_REVISION,
        "release_jar_sha256": RELEASE_HASH,
        "input": {"bytes": len(blob), "sha256": FIXTURE_HASH},
        "missing_manifest_control": {
            "bytes": len(negative),
            "sha256": sha256(negative).hexdigest(),
            "graphql_response": rejection,
            "projects_after": 0,
        },
        "output": {
            "project": project,
            "download_filename": filename,
            "bytes": len(archive),
            "sha256": sha256(archive).hexdigest(),
        },
    })


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("state", type=Path)
    parser.add_argument("fixture", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    args.output.mkdir()
    capture(json.loads(args.state.read_text(encoding="utf-8")), args.fixture, args.output)


if __name__ == "__main__":
    main()
