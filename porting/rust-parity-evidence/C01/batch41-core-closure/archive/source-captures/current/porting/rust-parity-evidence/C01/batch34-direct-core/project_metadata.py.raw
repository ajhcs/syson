"""Public metadata projection; original server cookies remain in private receipts."""
from pathlib import Path
import hashlib
import json
import sys


def project(value):
    if isinstance(value, list):
        return [project(item) for item in value]
    if not isinstance(value, dict):
        return value
    result = {key: project(item) for key, item in value.items()}
    if 'headers' in result:
        headers = []
        for name, text in result['headers']:
            if name.casefold() == 'set-cookie':
                raw = text.encode('utf-8')
                text = '[SERVER_COOKIE_REDACTED sha256=' + hashlib.sha256(raw).hexdigest() + ' bytes=' + str(len(raw)) + ']'
            headers.append([name, text])
        result['headers'] = headers
    return result


if __name__ == '__main__':
    source, destination = map(Path, sys.argv[1:])
    with destination.open('x') as output:
        json.dump(project(json.loads(source.read_bytes())), output, indent=2)
        output.write('\n')
