#!/usr/bin/env python3
"""Explicit reviewed empty BOOT.SF classification; original input stays intact."""
from pathlib import Path
import hashlib,json,zipfile
root=Path(__file__).resolve().parent
jar=Path('/workspace/build/builderbob/syson-audit-20261026/syson-application-2026.9.3.jar')
original=root/'packet-precompile.json'
packet=json.loads(original.read_text())
with jar.open('rb') as s:assert hashlib.file_digest(s,'sha256').hexdigest()==packet['artifact']['sha256']
with zipfile.ZipFile(jar) as z:
 raw=z.read('META-INF/BOOT.SF');assert raw==b''
marker={'entry':'META-INF/BOOT.SF','bytes':0,'sha256':hashlib.sha256(raw).hexdigest()}
assert packet['artifact']['signatures']==['META-INF/BOOT.SF']
assert packet['artifact']['manifest_digests']==[]
packet['artifact']['signatures']=[]
packet['artifact']['non_signature_markers']=[marker]
current=root/'packet-precompile-02.json';assert not current.exists()
current.write_text(json.dumps(packet,indent=2)+'\n')
receipt={'status':'REVIEWED_EMPTY_BOOT_MARKER_CLASSIFICATION_ONLY_COMPILE_PENDING',
 'input_sha256':hashlib.sha256(original.read_bytes()).hexdigest(),'output_sha256':hashlib.sha256(current.read_bytes()).hexdigest(),
 'marker':marker,'disposition':'Preserve unchanged; exact pinned outer artifact empty BOOT.SF only. Arbitrary/other/nonempty signature filenames and manifest digests remain forbidden.',
 'original_stager_and_packet':'Preserved unchanged','recipe_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
(root/'marker-classification-02.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps(receipt))
