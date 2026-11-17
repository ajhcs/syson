# SPDX-License-Identifier: EPL-2.0
"""Resolve and validate the native executables for the SysON reference."""

from dataclasses import dataclass
import hashlib
import os
from pathlib import Path
import re
import subprocess

JAR_SHA256 = "85c031352695d6ac8f059f0215ca493c58a16805a111233998dd443d6990c9e1"
PATH_KEYS = {
    "jar": "SYSON_ORACLE_JAR",
    "java": "SYSON_ORACLE_JAVA",
    "pg_bin": "SYSON_ORACLE_PG_BIN",
    "pg_share": "SYSON_ORACLE_PG_SHARE",
    "pg_extension_lib": "SYSON_ORACLE_PG_EXTENSION_LIB",
    "node_bin": "SYSON_ORACLE_NODE_BIN",
}
PG_LOADER_LIB = "SYSON_ORACLE_PG_LOADER_LIB"


@dataclass(frozen=True)
class Runtime:
    jar: Path
    java: Path
    pg_bin: Path
    pg_share: Path
    pg_extension_lib: Path
    node_bin: Path
    pg_loader_lib: Path | None = None

    def postgres_env(self, base):
        env = dict(base)
        if self.pg_loader_lib is not None:
            env["LD_LIBRARY_PATH"] = str(self.pg_loader_lib)
        return env


def choose_runtime(env, default):
    if not set(env).intersection([*PATH_KEYS.values(), PG_LOADER_LIB]):
        return default
    missing = [key for key in PATH_KEYS.values() if not env.get(key)]
    if missing:
        raise ValueError("Incomplete SysON oracle runtime; set " + ", ".join(missing))

    def absolute(key):
        path = Path(env[key])
        if not path.is_absolute():
            raise ValueError(f"{key} must be an absolute path")
        return path

    paths = {field: absolute(key) for field, key in PATH_KEYS.items()}
    if env.get(PG_LOADER_LIB):
        paths["pg_loader_lib"] = absolute(PG_LOADER_LIB)
    return Runtime(**paths)


def sha256(path):
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def check_runtime(runtime, pins):
    for field in ("pg_bin", "pg_share", "pg_extension_lib", "node_bin", "pg_loader_lib"):
        path = getattr(runtime, field)
        if path is not None and not path.is_dir():
            key = PG_LOADER_LIB if field == "pg_loader_lib" else PATH_KEYS[field]
            raise ValueError(f"{key} directory is missing: {path}")
    for path, key in [
        (runtime.jar, PATH_KEYS["jar"]),
        (runtime.pg_share / "postgres.bki", PATH_KEYS["pg_share"]),
        (runtime.pg_share / "postgresql.conf.sample", PATH_KEYS["pg_share"]),
        (runtime.pg_extension_lib / "plpgsql.so", PATH_KEYS["pg_extension_lib"]),
    ]:
        if not path.is_file() or not os.access(path, os.R_OK):
            raise ValueError(f"{key} file is missing or unreadable: {path}")

    for path, digest in pins.items():
        if sha256(path) != digest:
            raise ValueError(f"pinned asset changed: {path}")

    base_env = {"LANG": "C", "LC_ALL": "C", "PATH": "/usr/bin:/bin"}
    pg_env = runtime.postgres_env(base_env)
    checks = (
        (runtime.java, "-version", r'^(?:openjdk|java) version "21(?:[.+-]|")', base_env),
        (runtime.pg_bin / "initdb", "--version", r"^initdb \(PostgreSQL\) 15(?:[. ]|$)", pg_env),
        (runtime.pg_bin / "postgres", "--version", r"^postgres \(PostgreSQL\) 15(?:[. ]|$)", pg_env),
        (runtime.node_bin / "node", "--version", r"^v24\.", base_env),
    )
    for path, flag, version, env in checks:
        if not path.is_file() or not os.access(path, os.X_OK):
            raise ValueError(f"SysON oracle executable is missing or not executable: {path}")
        try:
            result = subprocess.run([str(path), flag], capture_output=True, text=True,
                                    check=False, timeout=15, env=env)
        except (OSError, subprocess.TimeoutExpired) as error:
            raise ValueError(f"cannot start SysON oracle executable {path}: {error}") from error
        if result.returncode != 0:
            detail = (result.stderr or result.stdout).strip()
            raise ValueError(f"cannot run {path} {flag}: {detail or result.returncode}")
        output = result.stdout + result.stderr
        if not re.match(version, output.strip()):
            raise ValueError(f"unexpected version from {path}: {output.strip().splitlines()[:1]}")
