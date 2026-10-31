"""Return the actual storage guard module, including its sealed POLICY globals."""
import hashlib, sys, types
from pathlib import Path

root = Path(__file__).resolve().parents[1]
path = root / 'composition/registration/prepare.py'
if path.resolve() != path or path.is_symlink() or path.stat().st_size != 22249:
    raise ValueError('fixed registration helper path/size changed')
with path.open('rb') as stream:
    raw = stream.read(22250)
if len(raw) != 22249 or hashlib.sha256(raw).hexdigest() != '61f8deab0d64a62a092b93bdf7c5d82cd3d10f9dfc2358eccff62d0a19e6bf59':
    raise ValueError('fixed registration helper whole-source pin changed')
provider = types.ModuleType('storage_registration_alias_provider')
provider.__file__ = str(path)
exec(compile(raw, str(path), 'exec'), provider.__dict__)
files = {}
for name in ['tools/preparation_splits.py', 'composition/storage_split_guard.py']:
    path = root / name
    provider.require(path.resolve() == path and path.is_file() and path.stat().st_size <= 262144,
                     'canonical bounded actual provider required')
    with path.open('rb') as stream:
        files[name] = stream.read(262145)
    provider.require(len(files[name]) <= 262144, 'provider changed past bound')
sys.modules[__name__] = provider.storage_guard(files)
