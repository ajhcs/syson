"""Alias the actual fixed .11 guard; importing reads only verified D/G sources."""
import hashlib
import re
import sys
import types
from pathlib import Path

PACKAGE = '_syson_source_next_1_0_11'
if any(name == PACKAGE or name.startswith(PACKAGE + '.') for name in sys.modules):
    raise ValueError('source-next qualified namespace collision')
loader = Path(__file__)
if loader.resolve() != loader or loader.is_symlink():
    raise ValueError('canonical source-next loader required')
directory = loader.parents[1] / 'composition/source_next_prepare'
pins = {'derive': 'fbb0352325dc948fdb1625348db69a4810dbfa61cc276a9c235716cecc8c9f17',
        'guard': 'eb80f827c8014cee147ed42c6efafa1f3856ff250fd2ea99a275cfd46aef12d8'}
sources = {}
for name, expected in pins.items():
    path = directory / (name + '.py')
    if (path.resolve() != path or path.is_symlink() or not path.is_file() or
            path.stat().st_size > 65536):
        raise ValueError('canonical bounded source-next provider required')
    with path.open('rb') as stream:
        raw = stream.read(65537)
    normalized = raw
    if name == 'guard':
        seal = rb"(?m)^POLICY_SHA256 = '(?:PENDING_ROOT_FREEZE|[0-9a-f]{64})'$"
        if len(re.findall(seal, raw)) != 1:
            raise ValueError('single strict source-next policy seal required')
        normalized = re.sub(seal, b"POLICY_SHA256 = 'PENDING_ROOT_FREEZE'", raw)
    if len(raw) > 65536 or hashlib.sha256(normalized).hexdigest() != expected:
        raise ValueError('source-next provider whole-source pin changed')
    sources[name] = (path, raw)

package = types.ModuleType(PACKAGE)
package.__path__ = [str(directory)]
if any(name == PACKAGE or name.startswith(PACKAGE + '.') for name in sys.modules):
    raise ValueError('source-next qualified namespace collision')
if sys.modules.setdefault(PACKAGE, package) is not package:
    raise ValueError('source-next qualified namespace collision')
for name, (path, raw) in sources.items():
    module = types.ModuleType(PACKAGE + '.' + name)
    module.__file__, module.__package__ = str(path), PACKAGE
    if sys.modules.setdefault(module.__name__, module) is not module:
        raise ValueError('source-next qualified namespace collision')
    setattr(package, name, module)
    exec(compile(raw, str(path), 'exec'), module.__dict__)
sys.modules[__name__] = package.guard
