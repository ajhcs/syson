# SysON committed a Documentation edit before the browser received a response

In Eclipse SysON v2026.9.3, a forwarded General View **Documentation** edit changed the selected PostgreSQL row before Chromium's request was aborted. A separate read-only PostgreSQL transaction saw the expected value and a new row version while the upstream success response was still withheld. Chromium recorded one failed fetch and zero successful responses for that edit. [Eclipse SysON release](https://github.com/eclipse-syson/syson/releases/tag/v2026.9.3) · [Recorded observation](observation.json)

| Phase | Browser observation | Database observation |
| --- | --- | --- |
| Acknowledged baseline | One forwarded edit and one successful response | Established the initial Documentation value |
| Unforwarded control | One aborted fetch and no forwarded edit | The selected row, value and row version stayed the same |
| Forwarded edit | One request forwarded, upstream success withheld, then one aborted fetch | The same row's value and version changed before the abort |

The control ended at the same row state used to start the forwarded phase. Both comparisons used separate read-only repeatable-read observations over a PostgreSQL Unix socket. The release and acknowledged restart baseline are pinned to the evidence in [#307](https://github.com/ajhcs/syson/pull/307).

The replacement Java process logged a startup banner. The host resource guard stopped the run before the fresh browser phase, leaving no post-restart browser log, page, archive or terminal success record. The controller preserved the failure and confirmed its owned scope was empty or absent. A previous resource hold is retained separately. It produced no sealed forwarded-edit barrier. Post-restart browser reopening, archive comparison and the complete recovery journey remain open for this case. [Acknowledged-edit restart evidence](../restart/full-backend-restart-v2026.9.3.json)

## Verify the retained projection

The verifier checks the pinned baseline and the sanitized event order, request counts, row comparisons and stop status. It rejects seven altered records, including a false browser acknowledgment and a false restart completion. The projection contains private artifact digests in place of request bodies, database identities, browser logs and host details. These commands check the retained evidence without rerunning SysON.

```sh
python3 -IB porting/oracle/no-ack-commit/verify.py
python3 -OIB porting/oracle/no-ack-commit/verify.py
```
