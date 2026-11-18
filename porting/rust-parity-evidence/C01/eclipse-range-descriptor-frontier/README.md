# Eclipse descriptor requests in the offline SysON core build replay

Maven 3.9.16 and Resolver 1.9.27 processed six captured Maven Central metadata
snapshots for the metamodel pinned at `6ba2ac64e6b16ee2b05752ad55395eebb87d4910`.
The metadata provider offered 193 versions from 214 literal entries. Each
ancestor case and the effective target case requested all 193 corresponding
POM descriptors. All 193 were absent from the frozen POM inputs.

| Eclipse range | Raw entries | Offered and requested |
| --- | ---: | ---: |
| JDT Core | 43 | 22 |
| JDT Launching | 40 | 40 |
| Core Resources | 35 | 35 |
| Core Runtime | 33 | 33 |
| Text | 29 | 29 |
| Debug Core | 34 | 34 |

The effective target also requested 21 other absent descriptors. Its Commons
Text 1.10.0 model separately requested an absent JUnit BOM 5.9.1 import. The
current frontier is therefore **215 unique POM coordinates**. Those POMs may
introduce further inputs. The target retains seven direct compile dependencies
and two direct test dependencies.

The withholding control supplied neither local nor Central metadata for JDT
Core. It returned zero offers and issued no descriptor request. All three
collector cases returned partial graphs. Their requests identify candidates
that Maven attempted to describe. Final target versions remain unresolved.

The replay serves one captured Maven Central origin and an empty local metadata
result. The target declares `Central`, two GitHub repositories and the inherited
`central` entry. The replay covers the captured Central metadata only. Full
repository policy, transitive closure, binary resolution and native compilation
remain open. No network transfer, binary resolution or Maven goal ran.

## Verify the recorded observation

The offline verifier checks the recorded projection, interval coverage and
descriptor frontier. It does not start Maven or recreate the runtime capture.

```sh
python3 -IB porting/rust-parity-evidence/C01/eclipse-range-descriptor-frontier/verify.py
python3 -OIB porting/rust-parity-evidence/C01/eclipse-range-descriptor-frontier/verify.py
```

Use `--frontier` with either command to list the exact repository-relative POM
paths, one per line, without accessing the network.
