#!/usr/bin/env python3
"""Apply an exact-span edit plan to a separate file, or measure an existing edit.

Usage: edit_locally.py SOURCE (--plan PLAN_JSON | --compare AFTER) [--output NEW]
Plans bind Unicode code-point spans to the SHA-256 of the original UTF-8 bytes.
No existing file is overwritten. Measurements are not editorial quality scores.
"""

import argparse
from bisect import bisect_right
from difflib import SequenceMatcher
import hashlib
import json
from pathlib import Path
import re
import sys

VERSION = "0.1.0"


class InputError(ValueError):
    pass


class Parser(argparse.ArgumentParser):
    def error(self, message):
        raise InputError(message)


def strict_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise InputError("Duplicate JSON key: %s" % key)
        result[key] = value
    return result


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def location(text, offset):
    starts = [0] + [m.end() for m in re.finditer(r"\r\n|\r|\n", text)]
    line = bisect_right(starts, offset) - 1
    return {"line": line + 1, "column": offset - starts[line] + 1}


def check_plan(plan, text, source_hash):
    if not isinstance(plan, dict) or set(plan) != {"source_sha256", "edits"}:
        raise InputError("Plan needs exactly source_sha256 and edits.")
    expected = plan["source_sha256"]
    if not isinstance(expected, str) or not re.fullmatch(r"[0-9a-fA-F]{64}", expected):
        raise InputError("source_sha256 must be a 64-character hexadecimal digest.")
    if expected.lower() != source_hash:
        raise InputError("Source hash mismatch; the plan does not describe these bytes.")
    if not isinstance(plan["edits"], list):
        raise InputError("edits must be an array.")
    edits = []
    for index, item in enumerate(plan["edits"]):
        if not isinstance(item, dict) or set(item) != {"start", "end", "before", "after", "reason"}:
            raise InputError("Edit %s has invalid fields." % index)
        start, end = item["start"], item["end"]
        if type(start) is not int or type(end) is not int or not 0 <= start <= end <= len(text):
            raise InputError("Edit %s has invalid code-point bounds." % index)
        if any(not isinstance(item[k], str) for k in ["before", "after", "reason"]):
            raise InputError("Edit %s text and reason must be strings." % index)
        if not item["reason"].strip():
            raise InputError("Edit %s needs a particular editorial reason." % index)
        if text[start:end] != item["before"]:
            raise InputError("Edit %s does not match its source substring." % index)
        if item["before"] == item["after"]:
            raise InputError("Edit %s makes no change; omit it." % index)
        item["after"].encode("utf-8", errors="strict")
        edits.append({**item, "plan_index": index, "location": location(text, start)})
    edits.sort(key=lambda e: (e["start"], e["end"]))
    for previous, current in zip(edits, edits[1:]):
        if current["start"] < previous["end"] or current["start"] == previous["start"]:
            raise InputError("Edits overlap or share an ambiguous starting offset.")
    return edits


def reconstruct(text, edits):
    parts = []
    cursor = 0
    for edit in edits:
        parts.extend([text[cursor:edit["start"]], edit["after"]])
        cursor = edit["end"]
    parts.append(text[cursor:])
    return "".join(parts)


def token_measurements(before, after):
    first, second = re.findall(r"\S+", before), re.findall(r"\S+", after)
    added = removed = 0
    for kind, a, b, c, d in SequenceMatcher(None, first, second, autojunk=False).get_opcodes():
        if kind in ("delete", "replace"):
            removed += b - a
        if kind in ("insert", "replace"):
            added += d - c
    return {
        "convention": "Whitespace-separated tokens; SequenceMatcher alignment with autojunk=False. Punctuation attached to a token counts with it.",
        "before_words": len(first), "after_words": len(second),
        "net_words_removed": len(first) - len(second),
        "net_reduction_percent": round(100 * (len(first) - len(second)) / len(first), 3) if first else None,
        "words_inserted": added, "words_removed_or_replaced": removed,
        "original_tokens_removed_or_replaced_percent": round(100 * removed / len(first), 3) if first else None,
        "interpretation": "A lexical alignment, not a unique attribution of edits, a preservation proof or a usefulness score.",
    }


def block_spans(text):
    spans = []
    start = None
    offset = 0
    for line in text.splitlines(keepends=True):
        if line.strip():
            if start is None:
                start = offset
        elif start is not None:
            spans.append((start, offset))
            start = None
        offset += len(line)
    if start is not None:
        spans.append((start, len(text)))
    # Whitespace-only separators are not part of a block's selected-text ratio.
    result = []
    for start, end in spans:
        value = text[start:end]
        left = start + len(value) - len(value.lstrip())
        right = end - len(value) + len(value.rstrip())
        if left < right:
            result.append((left, right))
    return result


def block_measurements(text, edits):
    result = []
    for index, (start, end) in enumerate(block_spans(text)):
        selected = sum(max(0, min(end, e["end"]) - max(start, e["start"])) for e in edits)
        insertions = sum(len(e["after"]) for e in edits if e["start"] == e["end"] and start <= e["start"] < end)
        result.append({"block": index + 1, "start": start, "end": end,
                       "original_characters": end - start, "original_characters_selected": selected,
                       "selected_percent": round(100 * selected / (end - start), 3),
                       "whole_block_selected": selected == end - start,
                       "characters_inserted_at_zero_width_spans": insertions})
    return result


def run(argv=None):
    report = {
        "tool": "edit-with-purpose-local-edit-record", "version": VERSION,
        "result": "unavailable", "source": None, "plan": None, "comparison": None,
        "candidate": None, "output": None, "changes": [], "errors": [],
        "assessment": {
            "conformance": {"status": "LIMITED", "scope": "The recorded hash, input schema, exact spans, nonoverlap, reconstruction and output behavior only."},
            "correctness": {"status": "NOT_ASSESSED"}, "usefulness": {"status": "NOT_ASSESSED"},
        },
    }
    parser = Parser(description=__doc__)
    parser.add_argument("source")
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--plan")
    mode.add_argument("--compare")
    parser.add_argument("--output")
    try:
        args = parser.parse_args(argv)
        if args.output and not args.plan:
            raise InputError("--output is only available with --plan.")
        source_path = Path(args.source)
        raw = source_path.read_bytes()
        text = raw.decode("utf-8", errors="strict")
        report["source"] = {"path": str(source_path), "sha256": sha(raw), "bytes": len(raw)}
        edits = None
        if args.plan:
            plan_raw = Path(args.plan).read_bytes()
            plan = json.loads(plan_raw.decode("utf-8-sig"), object_pairs_hook=strict_object)
            report["plan"] = {"path": args.plan, "sha256": sha(plan_raw)}
            edits = check_plan(plan, text, sha(raw))
            after = reconstruct(text, edits)
            report["changes"] = edits
            report["outside_named_spans"] = "Preserved by reconstruction from the checked original, without newline normalization."
            report["original_blocks"] = block_measurements(text, edits)
            report["block_convention"] = "Nonempty blocks separated by blank lines; not a full Markdown or sentence parser. Selection extent is informational, not a prohibition on deleting a whole block."
        else:
            after_raw = Path(args.compare).read_bytes()
            after = after_raw.decode("utf-8", errors="strict")
            report["comparison"] = {"path": args.compare, "sha256": sha(after_raw), "bytes": len(after_raw)}
            report["outside_named_spans"] = "NOT_ASSESSED in comparison mode."
        candidate_raw = after.encode("utf-8", errors="strict")
        report["candidate"] = {"sha256": sha(candidate_raw), "bytes": len(candidate_raw)}
        report["measurements"] = token_measurements(text, after)
        if args.output:
            output = Path(args.output)
            if output.resolve() in {source_path.resolve(), Path(args.plan).resolve()}:
                raise InputError("Output must be separate from source and plan.")
            with output.open("xb") as stream:
                stream.write(candidate_raw)
            report["output"] = {"path": str(output), "sha256": sha(candidate_raw), "bytes": len(candidate_raw)}
        report["result"] = "completed"
        return report, 0
    except (InputError, OSError, UnicodeError, ValueError, RecursionError) as error:
        report["result"] = "input_failure"
        report["errors"].append({"type": type(error).__name__, "message": str(error)})
        return report, 2


if __name__ == "__main__":
    result, code = run()
    print(json.dumps(result, ensure_ascii=True, indent=2))
    sys.exit(code)
