# Retry a Save that ended before its body reached the server

The captured Rust filesystem prototype received six of the seven declared bytes
of a same-origin Save form. The loopback client then closed its write side.
The server reported a short read and returned 500. The staged revision
remained hidden from downloads until a complete Save. Its archive route returned
404 before the interruption, afterward, and after a fresh service opened
the same revision directory. All existing revision files retained their
identities and bytes.

A complete retry returned 303 and added one ZIP and one confirmation record.
A new service downloaded the byte-identical 207 × 101 ZIP from the earlier
save-race comparison. A stale retry returned 409 with a link to the saved
revision and preserved the archive and marker. The owned scope ended after
all four services were reaped.

Verify the recorded outcome and committed comparator from the repository root.

```sh
python3 -IB porting/interop/evidence/archive-incomplete-save/verify.py
python3 -OIB porting/interop/evidence/archive-incomplete-save/verify.py
```

The commands validate this recorded projection and archive without starting
the prototype. The observation covers one loopback HTTP request that ended
before the prototype received its complete body. Application SQL transactions,
browser error presentation, and power-loss behavior remain open.
