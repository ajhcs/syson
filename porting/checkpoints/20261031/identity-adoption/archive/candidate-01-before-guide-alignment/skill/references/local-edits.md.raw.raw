# Local edits and change measurement

Use an exact-span plan when a file revision needs a reproducible record. The bundled Python 3 tool reconstructs a separate edited file from the source and selected changes. Text outside those changes is preserved, including whitespace and line endings.

```text
python <editor-skill>/scripts/edit_locally.py original.txt --plan edits.json --output edited.txt
python <editor-skill>/scripts/edit_locally.py original.txt --compare existing-edit.txt
```

Without `--output`, a plan is checked and measured without writing the candidate. Compare mode measures an already written edit; it does not establish that the edit followed a local process. The tool never overwrites an existing output or the source.

The plan is a JSON object with `source_sha256` and `edits`. Each edit contains exactly `start`, `end`, `before`, `after` and `reason`. Start and end are zero-based Unicode code-point offsets into the original decoded UTF-8 text; the end is exclusive. They are not byte offsets or JavaScript UTF-16 positions. Use the exact source substring as `before`. An empty `before` inserts text; an empty `after` deletes it. An empty edit list preserves the source.

```json
{
  "source_sha256": "<64-character SHA-256 of the original bytes>",
  "edits": [
    {"start": 0, "end": 1, "before": "<source substring>", "after": "<replacement>", "reason": "<the particular problem this change resolves>"}
  ]
}
```

Prepare offsets from the source rather than guessing them. The tool rejects a stale source hash, mismatched substring, overlap, ambiguous same-start edits, unchanged replacement, invalid schema or invalid UTF-8. It reports locations and the actual selected spans. Reasons are supplied editorial judgments; validating a reason field does not establish that the change is worthwhile.

Counts use whitespace-separated tokens. Inserted and removed counts come from a deterministic token alignment, with its convention declared. The report also shows the amount of each original blank-line block selected by a plan. A whole-block replacement is visible, not automatically forbidden: an unnecessary standalone paragraph may properly be deleted. A plan can still contain excessive edits, and small edits can still damage meaning.

There is no reduction target. Compare an actual deletion with its original and inspect what the reader loses or gains. Explain material growth when the brief calls for concision. Preserve source-defined quantities, conditions and assertion states, using the separate meaning review. A clean receipt verifies the named mechanical properties; correctness and usefulness remain unassessed.
