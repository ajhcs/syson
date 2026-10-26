#!/usr/bin/env python3
"""Root-run, source-delta comparison only; no compilation or application launch.

Run without -O. Inputs are coordinator-owned, immutable during this operation.
Output directory must not exist; failures leave diagnostic partial files, never
overwrite the official JAR. A success receipt follows complete readback checks.
"""
from pathlib import Path
import copy
import hashlib
import io
import json
import sys
import zipfile

BATCH = Path('/workspace/build/builderbob/syson-rust-parity-batch5-20261026')
INPUT = BATCH / 'reference-inputs'
PLAN = BATCH / 'reference-plan-01/plan.json'
OFFICIAL = Path('/workspace/build/builderbob/syson-audit-20261026/syson-application-2026.9.3.jar')
OUT = BATCH / 'reference-repack-01'
PACKET_SHA = '8e3e97e0544c33432a9443477109dd23741984fb123ed756f644f513d9fc277b'
EMPTY = hashlib.sha256(b'').hexdigest()


def need(ok, message):
    if not ok:
        raise ValueError(message)


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def file_sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':')).encode()


def save(name, value):
    (OUT / name).write_text(json.dumps(value, indent=2) + '\n')


def safe(name):
    need(isinstance(name, str) and len(name) <= 1024, 'path length/type')
    need('\\' not in name and ':' not in name, 'unsafe path')
    need(not any(ord(c) < 32 or ord(c) == 127 for c in name), 'control path')
    need(all(p not in ('', '.', '..') for p in name.rstrip('/').split('/')), 'unsafe path component')
    return name


def metadata(i):
    # Exactly the executed-stager fields; physical offset/compressed size differ.
    return dict(filename=i.filename, orig_filename=i.orig_filename,
                date_time=list(i.date_time), compress_type=i.compress_type,
                comment_hex=i.comment.hex(), extra_hex=i.extra.hex(),
                create_system=i.create_system, create_version=i.create_version,
                extract_version=i.extract_version, reserved=i.reserved,
                flag_bits=i.flag_bits, volume=i.volume, internal_attr=i.internal_attr,
                external_attr=i.external_attr, CRC=i.CRC, file_size=i.file_size)


def inventory(z):
    infos = z.infolist()
    need(len(infos) <= 100000, 'entry count')
    need(len({i.filename for i in infos}) == len(infos), 'duplicate ZIP entry')
    rows = []
    for i in infos:
        safe(i.filename)
        need(i.flag_bits & ~2056 == 0, 'unsupported ZIP flags')
        need(i.compress_type in (0, 8) and i.file_size <= 128 * 1024 * 1024, 'entry method/size')
        raw = z.read(i)
        need(len(raw) == i.file_size, 'payload size')
        rows.append(dict(entry=i.filename, sha256=sha(raw), bytes=len(raw),
                         metadata=metadata(i), metadata_sha256=sha(canonical(metadata(i))),
                         header_offset=i.header_offset, compress_size=i.compress_size))
    return rows


def unsigned(z):
    for i in z.infolist():
        u = i.filename.upper()
        if u.startswith('META-INF/') and (u.endswith(('.SF', '.RSA', '.DSA', '.EC')) or u.split('/')[-1].startswith('SIG-')):
            need(i.filename == 'META-INF/BOOT.SF' and i.file_size == 0 and z.read(i) == b'', 'signed changed archive')
        if u == 'META-INF/MANIFEST.MF':
            need(i.file_size <= 4 * 1024 * 1024, 'manifest size')
            for line in z.read(i).decode('ISO-8859-1').splitlines():
                if line and not line.startswith(' ') and ':' in line:
                    key = line.split(':', 1)[0].lower()
                    need(not (key.endswith('-digest') or '-digest-' in key), 'manifest digest')


class PreservedInfo(zipfile.ZipInfo):
    def _encodeFilenameFlags(self):
        # zipfile resets flag_bits during writing; preserve the original UTF8 bit.
        flags = self.flag_bits | (self.original_flags & 2048)
        return self.filename.encode('utf-8' if flags & 2048 else 'ascii'), flags


def clone_info(original):
    result = PreservedInfo(original.filename, original.date_time)
    for name in zipfile.ZipInfo.__slots__:
        if hasattr(original, name):
            setattr(result, name, copy.copy(getattr(original, name)))
    result.original_flags = original.flag_bits
    return result


def write_entry(z, info, raw):
    saved_seekable = z._seekable
    z._seekable = not bool(info.original_flags & 8)
    original_attrs = info.external_attr
    try:
        z.writestr(info, raw)
        # _open_to_write defaults zero attributes; central header is written later.
        info.external_attr = original_attrs
    finally:
        z._seekable = saved_seekable


def repack(source, destination, replacements, removals):
    with zipfile.ZipFile(source) as src:
        before = inventory(src)
        unsigned(src)
        original_names = {r['entry'] for r in before}
        need(removals <= original_names, 'missing removal')
        with zipfile.ZipFile(destination, 'w') as dst:
            dst.comment = src.comment
            for old in src.infolist():
                if old.filename in removals:
                    continue
                write_entry(dst, clone_info(old), replacements.get(old.filename, src.read(old)))
            for name in sorted(set(replacements) - original_names):
                # New entries have explicit deterministic metadata, never pretend inherited.
                info = PreservedInfo(safe(name), (1980, 1, 1, 0, 0, 0))
                info.original_flags = 2048
                info.compress_type = zipfile.ZIP_DEFLATED
                info.create_system = 3
                info.external_attr = 0o100644 << 16
                write_entry(dst, info, replacements[name])
    with zipfile.ZipFile(source) as src, zipfile.ZipFile(destination) as dst:
        after = inventory(dst)
        unsigned(dst)
        need(src.comment == dst.comment, 'archive comment changed')
        old = {r['entry']: r for r in before}
        new = {r['entry']: r for r in after}
        expected_order = [r['entry'] for r in before if r['entry'] not in removals]
        expected_order += sorted(set(replacements) - original_names)
        need([r['entry'] for r in after] == expected_order, 'archive order changed')
        for name, row in new.items():
            if name not in old:
                need(name in replacements, 'unplanned addition')
                continue
            if name not in replacements:
                need(row['sha256'] == old[name]['sha256'] and row['metadata'] == old[name]['metadata'], 'unselected bytes/metadata changed: ' + name)
            else:
                need(row['sha256'] == sha(replacements[name]), 'replacement mismatch')
                a, b = dict(old[name]['metadata']), dict(row['metadata'])
                for key in ('CRC', 'file_size'):
                    a.pop(key); b.pop(key)
                need(a == b, 'replacement metadata changed: ' + name)
    return dict(before=before, after=after)


def main():
    need(not sys.flags.optimize, 'run without Python optimization')
    need(not OUT.exists(), 'output exists; preserve previous attempt')
    packet_raw = (INPUT / 'packet-final.json').read_bytes()
    need(sha(packet_raw) == PACKET_SHA, 'packet hash drift')
    packet = json.loads(packet_raw)
    plan_raw = PLAN.read_bytes()
    plan = json.loads(plan_raw)
    inspection_raw = (INPUT / 'archive-inspection.json').read_bytes()
    inspection = json.loads(inspection_raw)
    need(plan['schema'] == 1 and plan['artifact_sha256'] == packet['artifact']['sha256'], 'plan pin')
    for key in ('source_revision', 'target_revision'):
        need(plan[key] == packet['artifact'][key], 'revision drift')
    need(plan['claim'] == 'SOURCE_DELTA_DERIVED_REFERENCE_ONLY' and plan['independent_build'] is False and plan['C01'] == 'OPEN' and plan['whole_build_source_binding'] == 'OPEN', 'claim boundary')
    need(file_sha(OFFICIAL) == plan['artifact_sha256'], 'official artifact hash')
    need(len(plan['docs']) == 4 and all(d['disposition'] == 'DOCUMENTATION_ONLY_NOT_OVERLAID' for d in plan['docs']), 'documentation disposition')
    need(plan['non_signature_markers'] == [dict(entry='META-INF/BOOT.SF', bytes=0, sha256=EMPTY, disposition='PRESERVE_UNCHANGED_EMPTY_BOOT_MARKER')], 'marker disposition')
    need(len(plan['resources']) == 114 and len(plan['families']) == 7 and len(plan['unchanged_containers']) == 309, 'plan coverage')
    delta = {r['path']: r for r in packet['delta']}
    compiled = {r['entry']: r for r in packet['compiled']}
    need(len(compiled) == len(packet['compiled']), 'duplicate compiled class')
    containers = {r['entry']: r for r in packet['containers']}
    replacements, removals, consumed_classes = {}, {}, set()
    for row in plan['resources']:
        source = delta[row['source_path']]
        need(source['target']['sha256'] == row['new_sha256'], 'resource target hash')
        raw = (INPUT / 'target' / safe(row['source_path'])).read_bytes()
        need(sha(raw) == row['new_sha256'] and len(raw) == source['target']['bytes'], 'staged resource drift')
        target = replacements.setdefault(row['container'], {})
        need(row['entry'] not in target, 'duplicate resource operation')
        target[safe(row['entry'])] = raw
    for row in plan['families']:
        target = replacements.setdefault(row['container'], {})
        need(set(row['old_members']) == {h['entry'] for h in packet['hits'] if h['source_path'] == row['source_path']}, 'old family inventory drift')
        removals.setdefault(row['container'], set()).update(set(row['old_members']) - set(row['new_members']))
        need(set(row['new_members']) == {c['entry'] for c in packet['compiled'] if c['source_path'] == row['source_path']}, 'new family inventory drift')
        for entry in row['new_members']:
            c = compiled[entry]
            need(c['source_path'] == row['source_path'] and c['major'] == 65, 'compiled family drift')
            raw = (INPUT / 'classes' / safe(entry)).read_bytes()
            need(sha(raw) == c['sha256'] and len(raw) == c['bytes'] and raw[:4] == b'\xca\xfe\xba\xbe' and int.from_bytes(raw[6:8], 'big') == 65, 'compiled payload drift')
            need(entry not in target and entry not in consumed_classes, 'duplicate class operation')
            target[entry] = raw
            consumed_classes.add(entry)
    need(consumed_classes == set(compiled), 'unconsumed compiled class')
    changed = set(replacements)
    need(changed == set(inspection['changed_containers']) and len(changed) == 3, 'changed-container closure')
    need(set(containers) - changed == {r['entry'] for r in plan['unchanged_containers']}, 'unchanged closure')
    for row in plan['unchanged_containers']:
        need(row['sha256'] == containers[row['entry']]['sha256'], 'unchanged hash drift')
    frozen_paths = [OFFICIAL, PLAN, INPUT/'packet-final.json', INPUT/'archive-inspection.json', Path(__file__), *[INPUT/'target'/r['source_path'] for r in plan['resources']], *[INPUT/'classes'/e for e in compiled]]
    inputs_before = {str(p): file_sha(p) for p in frozen_paths}
    OUT.mkdir()
    (OUT / 'lib').mkdir()
    (OUT / 'executed-recipe.py').write_bytes(Path(__file__).read_bytes())
    (OUT / 'plan.json').write_bytes(plan_raw)
    (OUT / 'packet-final.json').write_bytes(packet_raw)
    save('recipe.json', dict(helper_sha256=file_sha(Path(__file__)), packet_sha256=PACKET_SHA,
                            plan_sha256=sha(plan_raw), inspection_sha256=sha(inspection_raw),
                            official=str(OFFICIAL), argv=sys.argv, claim=plan['claim'],
                            independent_build=False, C01='OPEN'))
    nested_receipts, outer_replacements = {}, {}
    with zipfile.ZipFile(OFFICIAL) as outer:
        before_outer = inventory(outer)
        need([r['metadata'] for r in before_outer] == [r['metadata'] for r in inspection['outer']], 'outer inspection drift')
        need({i.filename for i in outer.infolist() if i.filename.startswith('BOOT-INF/lib/') and i.filename.endswith('.jar')} == set(containers), 'all 312 library identities')
        for name, container in containers.items():
            original = outer.read(name)
            need(sha(original) == container['sha256'] and outer.getinfo(name).compress_type == 0, 'original nested JAR drift')
            if name in changed:
                source = io.BytesIO(original)
                with zipfile.ZipFile(source) as nested:
                    rows = inventory(nested)
                need([r['metadata'] for r in rows] == [r['metadata'] for r in inspection['changed_containers'][name]], 'nested inspection drift')
                need([{k: r[k] for k in ('entry', 'sha256', 'bytes', 'metadata_sha256')} for r in rows] == container['members'], 'nested member hash drift')
                destination = OUT / 'lib' / Path(name).name
                nested_receipts[name] = repack(source, destination, replacements[name], removals.get(name, set()))
                raw = destination.read_bytes()
                need(sha(raw) != container['sha256'], 'selected JAR unchanged')
                outer_replacements[name] = raw
            else:
                # Signed unmodified dependency JARs remain exact opaque bytes.
                (OUT / 'lib' / Path(name).name).write_bytes(original)
    destination = OUT / 'syson-source-delta-comparison.jar'
    outer_receipt = repack(OFFICIAL, destination, outer_replacements, set())
    need({a['entry'] for a, b in zip(outer_receipt['before'], outer_receipt['after']) if a['sha256'] != b['sha256']} == changed, 'global outer payload delta')
    with zipfile.ZipFile(destination) as outer:
        need(outer.read('META-INF/BOOT.SF') == b'', 'marker changed')
        for row in plan['resources']:
            with zipfile.ZipFile(io.BytesIO(outer.read(row['container']))) as nested:
                need(sha(nested.read(row['entry'])) == row['new_sha256'], 'final resource hash')
        for row in plan['families']:
            with zipfile.ZipFile(io.BytesIO(outer.read(row['container']))) as nested:
                for entry in row['new_members']:
                    need(sha(nested.read(entry)) == compiled[entry]['sha256'], 'final class hash')
    save('outer-inventory.json', outer_receipt)
    save('nested-inventories.json', nested_receipts)
    inputs_after = {str(p): file_sha(p) for p in frozen_paths}
    need(inputs_after == inputs_before, 'repack input drift')
    save('receipt.json', dict(status='PASS_SELECTED_SOURCE_DELTA_REPACK', inputs_before=inputs_before, inputs_after=inputs_after, source_unchanged=True,
                            artifact=str(destination), sha256=file_sha(destination),
                            original_sha256=file_sha(OFFICIAL), changed_containers=sorted(changed),
                            unchanged_nested_jars=309, extracted_jars=312, resources=114, classes=len(compiled),
                            marker='PRESERVED_UNCHANGED_EMPTY_BOOT_MARKER',
                            metadata='all declared fields exact except CRC/size of replacements; offsets/compressed sizes observed separately',
                            additions='explicit deterministic metadata; original entry order retained',
                            independent_build=False, C01='OPEN', whole_build_source_binding='OPEN'))
    print(json.dumps(dict(status='PASS_SELECTED_SOURCE_DELTA_REPACK', artifact=str(destination), sha256=file_sha(destination))))


if __name__ == '__main__':
    main()
