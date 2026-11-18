# F26 changelog correspondence for SysON 2026.9.3

The official SysON 2026.9.3 application JAR contains all 29 changelog resources
from the [F26 source declaration plan](https://github.com/ajhcs/syson/pull/38)
byte for byte. The embedded Sirius infrastructure JAR supplies all 50,655 XML
bytes. The application JAR has no resources under `BOOT-INF/classes/db/`.

An independent traversal reproduced the plan's 28 ordered includes with their
attributes intact. All 20 changeset identities match the observed PostgreSQL
`DATABASECHANGELOG` by filename, ID, author, and execution order. Liquibase
4.33.0 recorded each as `EXECUTED`. The published projection contains only the
historical identity, order, and execution fields.

The retained Sirius source is `3191e2b8990373afee3415f6b26f8c5039a28321`.
The release source label is `b380dada415ce05155150f23cb3b74ae21cc45e4`.
The target source is `6ba2ac64e6b16ee2b05752ad55395eebb87d4910`. Their 125-path
Git delta includes no XML, properties, SQL, YAML, or TOML files. Both source
revisions share the root POM and ordinary and dev application properties, which
select the same `classpath:db/db.changelog-master.xml`.

The comparison binds these changelog bytes and the recorded history to this
release. The exact target build, effective dependency closure, and B05/B06
application acceptance remain open. Liquibase checksums and applied schema are
outside the scope of this verifier.

## Verify from a checkout

Copy `/syson-application.jar` from the pinned Linux amd64 release image
`eclipsesyson/syson@sha256:ff8fb1e44aab8150a45abd6225c6bfc057d4bcf1b680796f0a635174d9c54880`.
If you already have the JAR, use its path in the verification commands below.
The verifier checks the outer and nested JAR hashes and uses Python 3.10 or
later and Git. It reads the exact public F26 donor and the published history
projection without running a JAR or connecting to PostgreSQL.
[Official SysON image](https://hub.docker.com/r/eclipsesyson/syson/tags) · [Release](https://github.com/eclipse-syson/syson/releases/tag/v2026.9.3)

With Docker available, the following commands copy the JAR without starting the
container.

```sh
image=eclipsesyson/syson@sha256:ff8fb1e44aab8150a45abd6225c6bfc057d4bcf1b680796f0a635174d9c54880
docker pull --platform linux/amd64 "$image"
container=$(docker create --platform linux/amd64 "$image")
docker cp "$container:/syson-application.jar" ./syson-application-2026.9.3.jar
docker rm "$container"
```

From the repository root, fetch the exact F26 donor and run both verification modes.

```sh
git fetch https://github.com/ajhcs/syson.git refs/pull/38/head
git cat-file -e 'fc15b4bdd4363af8e29ea5a842981bb21378a497^{commit}'
python3 -IB porting/persistence/evidence/f26-release-changelog/verify.py --jar /path/to/syson-application-2026.9.3.jar
python3 -OIB porting/persistence/evidence/f26-release-changelog/verify.py --jar /path/to/syson-application-2026.9.3.jar
```
