# Eclipse Filesystem Maven range offers

Maven 3.9.16's `DefaultVersionRangeResolver` offered 25 versions of `org.eclipse.platform:org.eclipse.core.filesystem` for `[1.7.0,2.0.0)` using the exact metadata captured in #383. This was one fewer than the 26 XML declarations. Maven omitted only `1.6.1`, which lies below the inclusive lower bound. It added no versions. The remaining offers were in the same order as the XML, from `1.7.0` through `1.11.500`.

The pinned case saw no local metadata and associated all 25 offers with the frozen Central response. The withholding case supplied neither local nor Central metadata and yielded zero offers. Both cases reached the offers-only terminal. The controller exited successfully and confirmed teardown.

The included Java probe uses the Maven range resolver and a pinned metadata callback with the session offline. A kernel witness confirmed socket and `io_uring_setup` denial. No POM or binary resolution occurred. The uppermost offer is a candidate for later collection. Final version selection, dependency mediation, binary trust, and native compilation remain open.

Run both offline checks from this evidence directory.

```sh
python3 -IB verify.py
python3 -OIB verify.py
```

The verifier compares both recorded traces to #383's declarations and rejects altered offer order, scope, and origin. The public observation keeps the exact trace data and custody hashes. Raw captured metadata, host fields, and controller logs remain private.
