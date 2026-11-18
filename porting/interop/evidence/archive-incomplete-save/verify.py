from argparse import ArgumentParser
from hashlib import sha256
from io import BytesIO
from pathlib import Path
from zipfile import ZipFile
import json


HERE = Path(__file__).resolve().parent
OBSERVATION_SHA256 = "ee9a3e903c8dd2f185be9b72c4bd285d8738b091673ea7ae0be220ddeb32befb"
BASIS_COMMIT = "f7775b030340589eee2bd0086d0e4c1603ea9d29"
ARCHIVE_SHA256 = "8283fdf1759eb841b519000828b58b4888762f96d4678066a4571b7a30c3774d"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(data):
    return sha256(data).hexdigest()


def unique(pairs):
    out = {}
    for key, value in pairs:
        require(key not in out, f"duplicate key {key}")
        out[key] = value
    return out


def bounded(path, maximum):
    with path.open("rb") as stream:
        data = stream.read(maximum + 1)
    require(len(data) <= maximum, "input exceeds size limit")
    return data


def verify_observation(data):
    require(data["case"] == "incomplete-save" and data["basis_commit"] == BASIS_COMMIT,
            "case or basis changed")
    require(data["request"] == {"declared_bytes": 7, "sent_bytes": 6, "termination": "write-side EOF"},
            "incomplete request shape changed")
    require(data["statuses"] == {
        "pending_before_close": 404, "close_response": 500, "pending_after_close": 404,
        "pending_after_restart": 404, "complete_retry": 303, "archive_download": 200,
        "stale_retry": 409,
    }, "visibility or retry outcome changed")
    require(data["short_read_sha256"] == digest(b"fixture request did not complete: failed to fill whole buffer\n")
            and data["close_response_sha256"] == digest(b"Unable to read fixture state."),
            "short-read witness changed")
    require(data["published_roles"] == ["ZIP", "confirmation record"]
            and all(data[field] is True for field in (
                "staged_files_preserved_until_valid_save", "prior_files_preserved_after_valid_save",
                "retry_redirects_to_saved_revision", "stale_retry_links_to_saved_revision",
                "marker_binds_archive_and_checkpoint", "scope_removed",
            )), "publication, preservation, or cleanup changed")
    require(data["archive_sha256"] == ARCHIVE_SHA256 and data["archive_size"] == 27382
            and data["diagram_nodes"] == 9 and data["objects"] == 431
            and data["selected_size"] == [207, 101] and data["reaped_services"] == 4,
            "archive or service projection changed")


def verify_archive(raw):
    require(digest(raw) == ARCHIVE_SHA256, "archive seal changed")
    with ZipFile(BytesIO(raw)) as archive:
        names = archive.namelist()
        require(len(set(names)) == len(names) == 3 and
                sum(info.file_size for info in archive.infolist()) <= 4 * 1024 * 1024,
                "archive inventory or size changed")
        require(archive.testzip() is None, "invalid ZIP member CRC")
        representation, = (name for name in names if "/representations/" in name)
        document, = (name for name in names if "/documents/" in name)
        manifest, = (name for name in names if name.endswith("/manifest.json"))
        require(len({representation, document, manifest}) == 3, "archive roles changed")
        items = [json.loads(archive.read(document), object_pairs_hook=unique)]
        object_count = 0
        while items:
            item = items.pop()
            if isinstance(item, dict):
                object_count += "eClass" in item
                items.extend(item.values())
            elif isinstance(item, list):
                items.extend(item)
        require(object_count == 431, "model object count changed")
        view = json.loads(archive.read(representation), object_pairs_hook=unique)["representation"]
        root, = view["nodes"]
        drivetrain, = (node for node in root["childNodes"] if node["targetObjectLabel"] == "Drivetrain")
        layout = view["layoutData"]["nodeLayoutData"]
        require(len(layout) == 9 and layout[drivetrain["id"]]["size"] == {"width": 207, "height": 101},
                "winning geometry changed")


def main():
    parser = ArgumentParser()
    parser.add_argument("--archive", type=Path,
                        default=HERE.parent / "archive-corrected-save-race" / "winning-207.zip")
    args = parser.parse_args()
    raw = bounded(HERE / "observation.json", 8192)
    require(digest(raw) == OBSERVATION_SHA256, "observation seal changed")
    verify_observation(json.loads(raw, object_pairs_hook=unique))
    verify_archive(bounded(args.archive, 131072))
    print("PASS: interrupted Save leaves an unpublished draft and complete retry restores the exact archive")


if __name__ == "__main__":
    main()
