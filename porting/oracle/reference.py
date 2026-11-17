# SPDX-License-Identifier: EPL-2.0
"""Client for a running reference SysON started by oracle.py.

import_text(url, name, source) creates a blank project, uploads one .sysml file
through the same uploadDocument path the UI uses, and returns the uploaded
document as the server's own EMF JSON plus the import report.
"""
import io
import json
import urllib.request
import uuid
import zipfile

CREATE_PROJECT = """mutation($i: CreateProjectInput!) { createProject(input: $i) {
  __typename ... on CreateProjectSuccessPayload { project { id } }
  ... on ErrorPayload { messages { body level } } } }"""
EDITING_CONTEXT = """query($p: ID!) { viewer { project(projectId: $p) {
  currentEditingContext { id } } } }"""
UPLOAD = """mutation($input: UploadDocumentInput!) { uploadDocument(input: $input) {
  __typename ... on UploadDocumentSuccessPayload { report }
  ... on ErrorPayload { messages { body level } } } }"""
DELETE_PROJECT = """mutation($i: DeleteProjectInput!) { deleteProject(input: $i) { __typename } }"""


class ReferenceError(Exception):
    pass


def _post(url, body, content_type):
    req = urllib.request.Request(url, data=body, method="POST", headers={"Content-Type": content_type})
    with urllib.request.urlopen(req, timeout=120) as r:
        return json.loads(r.read())


def gql(base, query, variables):
    out = _post(base + "/api/graphql", json.dumps({"query": query, "variables": variables}).encode(),
                "application/json")
    if out.get("errors"):
        raise ReferenceError(json.dumps(out["errors"])[:800])
    return out["data"]


def _payload(data, field):
    payload = data[field]
    if payload["__typename"] == "ErrorPayload":
        raise ReferenceError(f"{field}: " + "; ".join(m["body"] for m in payload["messages"]))
    return payload


def _upload(base, editing_context, filename, content):
    """GraphQL multipart request (operations + map + one file part)."""
    boundary = uuid.uuid4().hex
    operations = {"query": UPLOAD, "variables": {"input": {
        "id": str(uuid.uuid4()), "editingContextId": editing_context, "file": None, "readOnly": False}}}
    parts = [("operations", None, json.dumps(operations).encode()),
             # Sirius Web reads each map entry as one string naming a field of $input.
             ("map", None, json.dumps({"0": "variables.file"}).encode()),
             ("0", filename, content)]
    body = io.BytesIO()
    for name, fname, data in parts:
        body.write(f"--{boundary}\r\n".encode())
        disposition = f'form-data; name="{name}"' + (f'; filename="{fname}"' if fname else "")
        body.write(f"Content-Disposition: {disposition}\r\n".encode())
        if fname:
            body.write(b"Content-Type: application/octet-stream\r\n")
        body.write(b"\r\n" + data + b"\r\n")
    body.write(f"--{boundary}--\r\n".encode())
    out = _post(base + "/api/graphql/upload", body.getvalue(), f"multipart/form-data; boundary={boundary}")
    if out.get("errors"):
        raise ReferenceError(json.dumps(out["errors"])[:800])
    return _payload(out["data"], "uploadDocument")


def _project_documents(base, project_id):
    req = urllib.request.Request(f"{base}/api/projects/{project_id}")
    with urllib.request.urlopen(req, timeout=120) as r:
        archive = zipfile.ZipFile(io.BytesIO(r.read()))
    docs = {}
    for name in archive.namelist():
        if "/documents/" in name and name.endswith(".json"):
            docs[name] = json.loads(archive.read(name))
    return docs


def import_text(base, name, source, keep=False):
    project_id = str(uuid.uuid4())
    created = _payload(gql(base, CREATE_PROJECT, {"i": {
        "id": str(uuid.uuid4()), "name": f"oracle-{name}"[:60], "templateId": "blank-project",
        "libraryIds": []}}), "createProject")
    project_id = created["project"]["id"]
    try:
        ec = gql(base, EDITING_CONTEXT, {"p": project_id})["viewer"]["project"]["currentEditingContext"]["id"]
        report = _upload(base, ec, f"{name}.sysml", source.encode())
        docs = _project_documents(base, project_id)
        return {"project_id": project_id, "report": report.get("report"), "documents": docs}
    finally:
        if not keep:
            try:
                gql(base, DELETE_PROJECT, {"i": {"id": str(uuid.uuid4()), "projectId": project_id}})
            except Exception:
                pass
