"""Fixed actual .12 provider admission; no catalogue or policy activation."""
import hashlib
import re
import sys
import types
from pathlib import Path

PACKAGE = '_syson_identity_partition_1_0_12'
FACADE_SHA256 = 'a4c564d11a7301dc20361d1c6eb4cd621a712b0da877cca48c4256066b28c5b3'
SOURCE_TOTAL = 384 * 1024


def load():
    if any(name == PACKAGE or name.startswith(PACKAGE + '.') for name in sys.modules):
        raise ValueError('identity provider namespace collision')
    pins = {'derive': '4bfa0cf3080e341efc342fbda38ac674ee106e99ff0d123f98f63861442fe1b4',
            'guard': '125b82688181a1e8a89e3b129530761baafeec32d61ea4858e95e7592eed462c',
            'construct': 'badd49826cdc8488cf46ff76c3fd906e1a5fff7d4afc3f975931fee2ad6dfcba',
            'capture': 'f7f68d493de4461af6f940c824f8f9d548de6951f99efc9cfa25230459a2de8c',
            'facade': FACADE_SHA256}
    if not all(type(pin) is str and re.fullmatch('[0-9a-f]{64}', pin) for pin in pins.values()):
        raise ValueError('qualified identity source pins pending or invalid')
    entry = Path(__file__)
    if (entry.resolve() != entry or entry.is_symlink() or not entry.is_file() or
            entry.stat().st_size > 65536):
        raise ValueError('canonical actual identity loader required')
    directory = entry.parents[1] / 'composition/identity_partition_prepare'
    paths = {name: directory / (name + '.py') for name in pins}
    sizes = {}
    for name, path in paths.items():
        if path.resolve() != path or path.is_symlink() or not path.is_file():
            raise ValueError('canonical actual identity provider required')
        sizes[name] = path.stat().st_size
    if any(size > 65536 for size in sizes.values()) or sum(
            size + 128 + 6 * len(name) for name, size in sizes.items()) > SOURCE_TOTAL:
        raise ValueError('early identity source byte framing bound')
    def read(path):
        if path.resolve() != path or path.is_symlink() or not path.is_file():
            raise ValueError('identity source origin changed')
        with path.open('rb') as stream:
            raw = stream.read(65537)
        if type(raw) is not bytes or len(raw) > 65536:
            raise ValueError('bounded identity source grew')
        return raw
    sources = {}
    for name, path in paths.items():
        raw = read(path)
        if len(raw) != sizes[name]:
            raise ValueError('identity source size changed')
        normalized = raw
        if name in ('guard', 'construct'):
            field = 'POLICY_SHA256' if name == 'guard' else 'CATALOGUE_SHA256'
            pattern = rb"(?m)^" + field.encode() + rb" = '(?:PENDING_ROOT_FREEZE|[0-9a-f]{64})'$"
            if len(re.findall(pattern, raw)) != 1:
                raise ValueError('single strict identity seal required')
            normalized = re.sub(pattern, field.encode() + b" = 'PENDING_ROOT_FREEZE'", raw)
        if hashlib.sha256(normalized).hexdigest() != pins[name]:
            raise ValueError('whole actual identity provider changed')
        sources[name] = raw
    if any(read(paths[name]) != raw for name, raw in sources.items()):
        raise ValueError('identity source changed before execution')
    if any(name == PACKAGE or name.startswith(PACKAGE + '.') for name in sys.modules):
        raise ValueError('identity provider namespace collision')
    package = types.ModuleType(PACKAGE)
    package.__path__ = [str(directory)]
    if sys.modules.setdefault(PACKAGE, package) is not package:
        raise ValueError('identity package registration collision')
    for name, raw in sources.items():
        module = types.ModuleType(PACKAGE + '.' + name)
        module.__file__, module.__package__ = str(paths[name]), PACKAGE
        if sys.modules.setdefault(module.__name__, module) is not module:
            raise ValueError('identity child registration collision')
        setattr(package, name, module)
        exec(compile(raw, str(paths[name]), 'exec'), module.__dict__)
    return package.facade


if __name__ == 'identity_partition_splits':
    sys.modules[__name__] = load()
