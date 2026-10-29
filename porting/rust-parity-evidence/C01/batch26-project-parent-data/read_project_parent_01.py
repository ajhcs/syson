from pathlib import Path
import hashlib
import json
import os
import subprocess
import xml.etree.ElementTree as ET

base = Path(__file__).parent
out = base / 'c01-project-parent-data-01'
os.umask(0o077)
out.mkdir(mode=0o700)
url = 'https://repo.maven.apache.org/maven2/org/springframework/boot/spring-boot-starter-parent/4.1.1/spring-boot-starter-parent-4.1.1.pom'
body = out / 'parent.pom'
argv = ['/usr/bin/curl', '-q', '--proxy', '', '--noproxy', '*', '--proto', '=https',
        '--max-time', '30', '--max-filesize', '262144', '--max-redirs', '0',
        '--silent', '--show-error', '--fail', '--header', 'Accept-Encoding: identity',
        '--cacert', '/etc/ssl/certs/ca-certificates.crt', '--dump-header', str(out / 'headers.txt'),
        '--output', str(body), '--write-out', '%{http_code}', url]
record = {'status': 'FAIL_PRESERVED', 'argv': argv, 'code_executed_from_response': False}
try:
    result = subprocess.run(argv, env={'PATH': '/usr/bin:/bin', 'HOME': os.environ['HOME'],
        'LANG': 'C.UTF-8'}, capture_output=True, timeout=40)
    (out / 'stdout.txt').write_bytes(result.stdout)
    (out / 'stderr.txt').write_bytes(result.stderr)
    record.update(exit=result.returncode, http_status=result.stdout.decode())
    if result.returncode != 0 or result.stdout != b'200' or body.stat().st_size > 262144:
        raise RuntimeError('literal bounded parent HTTP request failed')
    raw = body.read_bytes()
    if b'<!DOCTYPE' in raw.upper() or b'<!ENTITY' in raw.upper():
        raise RuntimeError('parent XML declaration refused')
    root = ET.fromstring(raw)
    parent = root.find('{*}parent')
    record['parent'] = None if parent is None else {key: parent.findtext('{*}' + key)
        for key in ['groupId', 'artifactId', 'version']}
    record.update(status='PASS_PARENT_XML_DATA_ONLY;REVIEW_BEFORE_MODEL_EXECUTION', bytes=len(raw),
        sha256=hashlib.sha256(raw).hexdigest(), root_tag=root.tag,
        extension_elements=[ET.tostring(node, encoding='unicode') for node in root.iter()
            if node.tag.rsplit('}', 1)[-1] in ['extensions', 'activation']])
except BaseException as error:
    record['error'] = repr(error)
    raise
finally:
    (out / 'receipt.json').write_text(json.dumps(record, indent=2) + '\n')
print(json.dumps(record))
