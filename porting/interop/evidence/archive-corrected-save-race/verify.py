from argparse import ArgumentParser
from hashlib import sha256
from io import BytesIO
from pathlib import Path
from zipfile import ZipFile
import json


HERE = Path(__file__).resolve().parent
OBSERVATION_SHA256 = "41b2b123a3b1b343b052332c34214a545ed4e6fd82a4b7b56c2119c7aa72cc47"
BASELINE_SHA256 = "e24d7f1f4fe5ddf4e5047dc62b89c2cca029e367efaefbbc915656c9b08b1abf"
WINNER_SHA256 = "8283fdf1759eb841b519000828b58b4888762f96d4678066a4571b7a30c3774d"


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


def verify_observation(raw):
    require(digest(raw) == OBSERVATION_SHA256, "observation seal changed")
    data = json.loads(raw, object_pairs_hook=unique)
    require(data["writer_count"] == 2 and data["same_directory"],
            "distinct shared-directory intake required")
    require(data["locked_stage_statuses"] == data["locked_save_statuses"] == [503, 503]
            and data["lock_control_unchanged"], "held lock control changed")
    require(data["proposed_widths"] == [206, 207] and data["winning_size"] == [207, 101]
            and data["stage_statuses"] == [503, 303] and data["save_statuses"] == [303, 503]
            and data["loser_reasons"] == ["directory busy", "directory busy"]
            and data["client_windows_overlapped"] == [True, True], "race outcome changed")
    require(data["stage_retry_status"] == data["stale_save_status"] == 409
            and data["stage_retry_reason"] == "draft revision already exists"
            and data["stale_save_reason"] == "revision was already saved"
            and data["draft_download_statuses"] == [404, 404]
            and data["fresh_download_status"] == 200 and data["fresh_download_bytes_match"],
            "visibility or retry outcome changed")
    require(data["new_artifacts"] == [["draft checkpoint"], ["ZIP", "confirmation record"]]
            and data["prior_files_unchanged"] and data["marker_binds_archive"]
            and data["reaped_services"] == 4 and data["scope_removed"]
            and data["winning_zip_sha256"] == WINNER_SHA256, "storage outcome changed")


def verify_archives(baseline, winner):
    require(digest(baseline) == BASELINE_SHA256 and digest(winner) == WINNER_SHA256,
            "archive seal changed")
    with ZipFile(BytesIO(baseline)) as left, ZipFile(BytesIO(winner)) as right:
        names = left.namelist()
        require(len(set(names)) == len(names) == 3 and names == right.namelist()
                and left.comment == right.comment, "ZIP inventory changed")
        for old, new in zip(left.infolist(), right.infolist()):
            require(old.file_size <= 4 * 1024 * 1024 and new.file_size == old.file_size
                    and (old.date_time, old.create_system, old.external_attr, old.extra, old.comment)
                    == (new.date_time, new.create_system, new.external_attr, new.extra, new.comment),
                    "archive metadata changed")
        require(left.testzip() is None and right.testzip() is None, "invalid ZIP member CRC")
        representation, = (name for name in names if "/representations/" in name)
        original = left.read(representation)
        before = json.loads(original, object_pairs_hook=unique)["representation"]
        root, = before["nodes"]
        node, = (n for n in root["childNodes"] if n["targetObjectLabel"] == "Drivetrain")
        layout = before["layoutData"]["nodeLayoutData"]
        require(len(layout) == 9 and layout[node["id"]]["size"] == {"width": 206, "height": 101},
                "selected baseline node changed")
        require(original.count(b'"width":206.0') == 1 and
                original.replace(b'"width":206.0', b'"width":207.0', 1) == right.read(representation),
                "representation change exceeds the selected width")
        require(all(left.read(name) == right.read(name) for name in names if name != representation),
                "model or manifest bytes changed")


def main():
    parser = ArgumentParser()
    parser.add_argument("--baseline", type=Path,
                        default=HERE.parent / "archive-corrected-publication" / "rust-corrected.zip")
    args = parser.parse_args()
    verify_observation(bounded(HERE / "observation.json", 8192))
    verify_archives(bounded(args.baseline, 131072), bounded(HERE / "winning-207.zip", 131072))
    print("PASS: one correction, one Save, exact fresh ZIP and single width change")


if __name__ == "__main__":
    main()
