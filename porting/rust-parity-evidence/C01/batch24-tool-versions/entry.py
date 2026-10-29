from pathlib import Path
import hashlib
import sys
path = Path(sys.argv[1])
raw = path.read_bytes()
if hashlib.sha256(raw).hexdigest() != sys.argv[2]:
    raise RuntimeError('reviewed version runner changed before compilation')
descriptor = sys.argv[3]
sys.argv = [str(path), '--lock-fd', descriptor]
exec(compile(raw, str(path), 'exec'), {'__name__': '__main__', '__file__': str(path)})
