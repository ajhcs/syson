"""Replay the verified zoom and pan observations."""
from hashlib import sha256
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def require(value, message):
    if not value:
        raise ValueError(message)


def check(view, mobile, archive):
    require(view['archive'] == view['afterZoom'] == archive, 'Viewport interaction changed the ZIP')
    zoom = view['zoom']
    mode = 'touch' if mobile else 'keyboard'
    actions = zoom['actions']
    require(set(row['label'] for row in actions) == {'Actual size','Zoom in','Zoom out','Fit diagram'}
            and all(row['mode'] == mode and row['trustedTouch'] == mobile
                    and row['width'] >= 44 and row['height'] >= 44 for row in actions),
            'Zoom controls differ')
    states = zoom['checkpoints']
    fit, actual, far, maximum = (states[name] for name in ('fit','actual','far','max'))
    require(fit == states['restored'] and fit['left'] == 0 and fit['maximum'] <= 1
            and all(row['visible'] for row in fit['texts']), 'Fit did not restore the overview')
    require(actual['label'] == '100%' and actual['left'] == 0 and maximum['label'] == '200%'
            and states['reduced']['maximum'] < maximum['maximum'], 'Zoom bounds differ')
    require(all(row['viewBox'] == fit['viewBox'] and row['documentWidth'] <= row['viewWidth']
                and 0 <= row['left'] <= row['maximum'] + 1
                for row in (fit,actual,far,maximum,states['reduced'])), 'Pan or document limits differ')
    first, = [row for row in actual['texts'] if row['name'] == 'Drivetrain']
    last, = [row for row in far['texts'] if row['name'] == 'Body']
    require(first['visible'] and first['height'] >= 14 and last['visible'] and last['height'] >= 14
            and far['left'] > 500, 'Near or far package name was clipped')
    if mobile:
        landscape = states['landscape']
        require(zoom['trustedMove'] and landscape['fit'] == landscape['restored']
                and landscape['max']['vertical'] > 0 and landscape['max']['maximum'] > 0,
                'Trusted swipe or short viewport restoration differs')
    else:
        require(states['keyboard']['left'] > 0 and far['left'] > states['keyboard']['left'],
                'Separate keyboard and wheel movement observations differ')


def check_scrollbars(rows, candidate):
    entries = {(row['mode'],row['profile']):row for row in rows}
    require(len(rows) == len(entries) == 4, 'Scrollbar comparisons are incomplete')
    for profile in ('native','native-nine'):
        before = entries['prior',profile]['browser']
        after = entries['candidate',profile]['browser']
        require(entries['candidate',profile]['binary_sha256'] == candidate
                and not before['errors'] and not after['errors'], 'Scrollbar binary or browser differs')
        require(after['enlarged']['outerWidth'] - after['enlarged']['clientWidth'] >= 14
                and after['initial']['nodes'] == after['settled']['nodes']
                and after['settled']['scrollHeight'] == after['settled']['clientHeight'],
                'Classic scrollbar or restored diagram evidence differs')
    old = entries['prior','native-nine']['browser']['settled']
    new = entries['candidate','native-nine']['browser']['settled']
    require(old['clientWidth'] - old['chartWidth'] >= 14 and new['clientWidth'] - new['chartWidth'] <= 1,
            'The wide diagram did not return to stable fitted width')
    old = entries['prior','native']['browser']['settled']
    require(old['scrollHeight'] > old['clientHeight'], 'Prior height clipping was not reproduced')


def main():
    pins = json.loads((ROOT / 'provenance.json').read_bytes())
    require(set(pins) == {'browser.json','scrollbar.json'}, 'Evidence inventory differs')
    for name, digest in pins.items():
        require(sha256((ROOT / name).read_bytes()).hexdigest() == digest, f'Evidence differs for {name}')
    receipt = json.loads((ROOT / 'browser.json').read_bytes())
    archive = sha256((ROOT.parent / 'archive-hierarchy/rust-successor.zip').read_bytes()).hexdigest()
    require(receipt['archive'] == archive and receipt['fixture']['model_objects'] == 431
            and receipt['fixture']['rendered_nodes'] == 9 and receipt['retained_earlier_files']
            and receipt['scope_removed'] and receipt['signals'] == ['SIGTERM','SIGKILL','SIGTERM','SIGTERM'],
            'Archive or recovery evidence differs')
    desktop, mobile = receipt['views']
    require(desktop['viewport'] == {'width':1440,'height':960}
            and mobile['viewport'] == {'width':320,'height':660}, 'Expected browser viewports differ')
    check(desktop, False, archive)
    check(mobile, True, archive)
    check_scrollbars(json.loads((ROOT / 'scrollbar.json').read_bytes()),
                     receipt['subject']['syson-archive-reader-experiment'])
    print('PASS zoom, keyboard and touch pan, stable Fit, and unchanged archive')


if __name__ == '__main__':
    main()
