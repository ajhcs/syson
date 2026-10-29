from pathlib import Path
import hashlib
import os
import sys
base = Path(__file__).resolve().parent
helper = base.parent / 'scope-ownership-repair/scope_ownership.py'
helper_raw = helper.read_bytes()
if hashlib.sha256(helper_raw).hexdigest() != 'ff18c5d2e6af6a8b21a4d59129401d9c9274f4d4727e2e448eae331c72f6b92d':
    raise RuntimeError('reviewed scope ownership helper changed')
ownership = {'__file__': str(helper)}
exec(compile(helper_raw, str(helper), 'exec'), ownership)
path = Path(sys.argv[1])
unit, cookie, destination = sys.argv[4:7]
ownership['attest'](unit, cookie, 'syson-c01-versions-', destination, os.environ)
raw = path.read_bytes()
if hashlib.sha256(raw).hexdigest() != sys.argv[2]:
    raise RuntimeError('reviewed body changed before compilation')
sys.argv = [str(path), '--lock-fd', sys.argv[3], '--scope-unit', unit]
exec(compile(raw, str(path), 'exec'), {'__name__': '__main__', '__file__': str(path)})
