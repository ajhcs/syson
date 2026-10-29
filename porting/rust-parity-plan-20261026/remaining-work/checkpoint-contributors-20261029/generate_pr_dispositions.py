"""Derive a dated review queue; contacts and base refs do not grant source leases."""
from pathlib import Path
import hashlib
import json

base = Path(__file__).resolve().parent
raw = (base / 'pr-queue-audit.json').read_bytes()
if hashlib.sha256(raw).hexdigest() != '922ab47d50420ceafdcba1f9d2a012b0656b6948a96cadbde9e83630da7f7d66':
    raise RuntimeError('The reviewed queue audit changed')
output = base / 'pr-dispositions.json'
if output.exists():
    raise RuntimeError('Preserve the existing queue disposition')
audit = json.loads(raw)
closed = {row['pr']: row for row in json.loads((base / 'superseded-closeout.json').read_bytes())}
rows = []
for item in audit['rows']:
    number = item['number']
    status = 'NEEDS_DEPENDENCY_OWNERSHIP_AND_QUALIFICATION_REVIEW'
    if number in [27, 29, 36, 39]:
        status = 'RECONCILE_WITH_LANDED_SLICES;PRESERVE_UNIQUE_EVIDENCE'
    elif number == 130:
        status = 'BLOCKED_PREREQUISITES_AND_HELD_OWNERSHIP'
    elif number == 148:
        status = 'DIAGNOSTIC_CAPTURE;UNRESOLVED_QUALIFICATION'
    elif number == 1:
        status = 'HISTORICAL_ROLLUP;DUPLICATE_MERGE_PROHIBITED'
    row = {'pr': number, 'url': 'https://github.com/ajhcs/syson/pull/' + str(number),
           'title': item['title'], 'maintainer_contact': '@ajhcs',
           'implementation_owner': 'UNVERIFIED;CONFIRM_EXCLUSIVE_LEASE_BEFORE_WORK',
           'recorded_base_ref': item['base_ref'], 'recorded_base_commit': item['base'],
           'prerequisite_checkout': 'https://github.com/ajhcs/syson/tree/' + item['base'],
           'other_prerequisites': 'UNVERIFIED;REVIEW_ORIGINAL_PR_AND_UNIT_BRIEF',
           'status': status}
    if number in closed:
        row.update(status='CLOSED_SUPERSEDED;BRANCH_RETAINED',
                   replacement_pr=closed[number]['replacement_pr'],
                   disposition_comment=closed[number]['comment_url'])
    rows.append(row)
if len(rows) != 110 or len({row['pr'] for row in rows}) != 110:
    raise RuntimeError('Incomplete reviewed PR coverage')
output.write_text(json.dumps({'audit_main': audit['main'],
    'meaning': 'Dated contact/base/status routing; base refs are not a complete dependency audit or source ownership proof',
    'rows': rows}, indent=2) + '\n')
