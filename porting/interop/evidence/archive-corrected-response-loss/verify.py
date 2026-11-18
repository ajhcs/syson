import hashlib
import json
import sys
from pathlib import Path

OBSERVATION_SHA256 = '63b88989e6fcbe0a9d61da61aa162170aa843b49d88a09517e8c71bcfc2f2c4d'
ARCHIVE_SHA256 = 'e24d7f1f4fe5ddf4e5047dc62b89c2cca029e367efaefbbc915656c9b08b1abf'
ROOT = Path(__file__).resolve().parent


def require(condition, message):
    if not condition:
        raise ValueError(message)


def verify(data, archive):
    require(hashlib.sha256(data).hexdigest() == OBSERVATION_SHA256, 'observation changed')
    require(hashlib.sha256(archive).hexdigest() == ARCHIVE_SHA256, 'parent archive changed')
    observation = json.loads(data)
    require(observation['parent_commit'] == '02223020295cd540b13144ad3a296b7fd8ac8a88', 'wrong parent')
    require(observation['fixture'] == {'archive_sha256': ARCHIVE_SHA256, 'nodes': 9,
                                       'model_objects': 431, 'width': 206, 'height': 101}, 'wrong fixture')
    withheld = observation['withheld']
    require(withheld['upstream_requests'] == 1 and withheld['server_status'] == 303 and
            withheld['response_bytes'] == 0 and withheld['same_route'] is True, 'missing upstream acknowledgment')
    require(withheld['chromium_statuses'] == [] and withheld['chromium_failures'] == ['net::ERR_CONNECTION_CLOSED'],
            'unexpected Chromium outcome')
    require(withheld['retry'] == {'status':409,'missingOriginControl':403,'unchanged':True,'existingRevisionMessage':True},
            'wrong restart retry outcome')
    delivered = observation['delivered']
    require(delivered['chromium_statuses'] == [303, 409], 'missing normal delivery control')
    for case in (withheld, delivered):
        require(case['new_server_process'] is True and case['scope_removed'] is True and
                case['fresh_browser_download_sha256'] == ARCHIVE_SHA256, 'incomplete fresh reopen evidence')


def main():
    archive = ROOT.parent / 'archive-corrected-publication' / 'rust-corrected.zip'
    if len(sys.argv) == 2:
        archive = Path(sys.argv[1])
    elif len(sys.argv) != 1:
        raise ValueError('usage: verify.py [parent rust-corrected.zip]')
    verify((ROOT/'observation.json').read_bytes(), archive.read_bytes())
    print('PASS: withheld Save response, restart retry, normal delivery, exact fresh download')


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, KeyError, TypeError) as error:
        sys.exit(f'REFUSE: {error}')
