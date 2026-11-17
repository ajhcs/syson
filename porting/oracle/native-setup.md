# Run the live reference oracle on Linux

The live oracle starts the Eclipse SysON v2026.9.3 release with Java 21,
PostgreSQL 15 and Node.js 24. It creates a fresh PostgreSQL database under
your run directory and binds both servers to loopback. Run it with Python
3.10 or later as a regular Linux user.

You can replay the [recorded results](README.md#replay-without-a-running-reference)
with only Python and Rust. Use the live oracle to collect a new reference
result or investigate a difference.

## Get the pinned SysON JAR

Use the JAR from the [official SysON v2026.9.3 image](https://hub.docker.com/r/eclipsesyson/syson/tags).
Its SHA-256 must be
`85c031352695d6ac8f059f0215ca493c58a16805a111233998dd443d6990c9e1`.
The release image records upstream source revision
`b380dada415ce05155150f23cb3b74ae21cc45e4` in its labels. The
[reference audit](../audits/20261026/native-reference.json) records the image
manifest and layer digests.

To extract the JAR without a Docker daemon, use
[Skopeo](https://github.com/containers/skopeo/blob/main/docs/skopeo-copy.1.md)
and GNU tar. The command pins the Linux/amd64 image manifest that contains
the expected JAR. The other runtimes below should match your host architecture.

```bash
set -euo pipefail
assets="$PWD/syson-reference-assets"
mkdir -m 700 "$assets"
image=sha256:ff8fb1e44aab8150a45abd6225c6bfc057d4bcf1b680796f0a635174d9c54880
layer=2125b79e5544395c0b34e28bda903b578cf691955b917482a8a8d725c56e15e4
skopeo copy --preserve-digests \
  "docker://docker.io/eclipsesyson/syson@$image" "oci:$assets/image:reference"
tar -xOzf "$assets/image/blobs/sha256/$layer" syson-application.jar \
  > "$assets/syson-application.jar"
printf '%s  %s\n' \
  85c031352695d6ac8f059f0215ca493c58a16805a111233998dd443d6990c9e1 \
  "$assets/syson-application.jar" | sha256sum -c -
export SYSON_ORACLE_JAR="$assets/syson-application.jar"
```

## Configure the native runtimes

Use a [Java 21 JRE or JDK](https://adoptium.net/temurin/releases?version=21),
a [PostgreSQL 15 server distribution](https://www.postgresql.org/download/),
and a [Node.js 24 release](https://nodejs.org/en/download). A PostgreSQL
client package alone lacks `initdb`, the server binary and the share files.
Use one PostgreSQL distribution for every PostgreSQL path. If it provides
[pg_config](https://www.postgresql.org/docs/15/app-pgconfig.html), its
`--bindir`, `--sharedir` and `--pkglibdir` report the required directories.

Set all six variables in the same shell. The PostgreSQL paths below show a
Debian or Ubuntu server package layout. Replace the Java and Node directories
with your installations.

```bash
export SYSON_ORACLE_JAR="$assets/syson-application.jar"
export SYSON_ORACLE_JAVA=/absolute/path/to/java21/bin/java
export SYSON_ORACLE_PG_BIN=/usr/lib/postgresql/15/bin
export SYSON_ORACLE_PG_SHARE=/usr/share/postgresql/15
export SYSON_ORACLE_PG_EXTENSION_LIB=/usr/lib/postgresql/15/lib
export SYSON_ORACLE_NODE_BIN=/absolute/path/to/node24/bin

"$SYSON_ORACLE_JAVA" -version
"$SYSON_ORACLE_PG_BIN/initdb" --version
"$SYSON_ORACLE_PG_BIN/postgres" --version
"$SYSON_ORACLE_NODE_BIN/node" --version
```

For a relocated PostgreSQL runtime whose dynamic loader needs a separate
dependency directory, also set `SYSON_ORACLE_PG_LOADER_LIB` to its absolute
path. The oracle supplies that loader path only to PostgreSQL. Preserve the
distribution's relative layout when relocating it. For example, keep
`usr/lib/postgresql/15/bin` and `usr/share/postgresql/15` at their original
locations relative to the extraction root, including any shared files that
the package references by symlink. For a normal system installation, leave
the loader variable unset.

## Start, compare and stop

Use a short absolute run path inside a work directory you own. PostgreSQL's
Unix socket filename, including its parent directories, is limited to
107 bytes on Linux. Pick a fresh run directory for an independent capture.

```bash
set -euo pipefail
export SYSON_ORACLE_RUNS=/absolute/short/work/directory/syson-oracle/runs
export CARGO_TARGET_DIR=/absolute/work/directory/syson-target

cargo build --locked --example oracle_import \
  --manifest-path rust/crates/syson-text-import/Cargo.toml

python3 -IB porting/oracle/oracle.py up
trap 'python3 -IB porting/oracle/oracle.py down' EXIT
python3 -IB porting/oracle/oracle.py status
python3 -IB porting/oracle/differential.py \
  --rust-bin "$CARGO_TARGET_DIR/debug/examples/oracle_import"
```

The differential prints the result directory. Check its `summary.json` for
harness errors. The current corpus includes known Rust refusals and a semantic
difference. See the [verdict definitions](README.md#what-a-verdict-means).
The oracle stores process state beside the run directory. Teardown removes
that state and keeps run data and logs for inspection.
