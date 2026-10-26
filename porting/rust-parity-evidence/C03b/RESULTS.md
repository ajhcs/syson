# C03b results

Ten consuming integration tests, formatting, all-target Clippy and the release observer build pass at 59 unchanged inputs. The exact four-migration prerequisite passes 36 tests at 109 unchanged inputs. C03a verification is retained as a labeled transcription of actual tool output, without inventing a raw stdout capture.

Fresh combined and foreign-first projects match source-compiled native b/a/c plus d for whole JSON values and the first namespace key. A late nested-array case retains native partial mutation while Rust publishes no project. The oracle runs two deduplicated semantic inputs and two control inputs. Derived local references avoid library-ID replacements: B01 runs, but its native library/history/SQL equivalence is not inferred. Rust integration tests separately exercise actual B01 replacements, independent stamps, replay and private-data rollback.

Observation01 remains a FAIL: Java stdout under LC_ALL=C replaced supplementary Unicode with a question mark. Observation02 explicitly configures Java21 stdout/stderr UTF8 and matches unchanged source values. No source or comparison normalization hides that failure.

| Semantic nodes | Diagram nodes | Median preparation ms | p95 ms | Whole observer peak RSS KiB |
| ---: | ---: | ---: | ---: | ---: |
| 256 | 6 | 1.7735 | 2.253 | 5184 |
| 1024 | 6 | 6.8880 | 6.923 | 11328 |
| 4096 | 6 | 29.1005 | 29.305 | 37612 |

Each size uses 12 raw samples, discards two warmups and reports ten measured samples. Timings include only `prepare`; generation, envelope parsing and output encoding are excluded. RSS covers the complete observer process. These are synthetic flat inputs; no native/before performance ratio or user/UI capacity claim is made. All raw packets, sample outputs, RSS files and exact runners are retained.
