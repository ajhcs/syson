# Competing corrected archives publish once

Two owned Rust HTTP listeners opened the same revision directory. While the test held the directory lock, both listeners returned 503 for Stage and for Save, and the directory contents stayed unchanged. After releasing the lock, the test issued competing 206 and 207 width proposals with overlapping client request windows. The 207 proposal returned 303 and created the only checkpoint. The other returned 503 with the busy message, then 409 on retry. Both listeners showed the winning draft and withheld its download.

Two Save requests then competed on that checkpoint. The other listener returned 303 and added one ZIP and one confirmation record. The losing Save returned 503 with the busy message. After both listeners stopped, a fresh Rust process returned the exact winning ZIP. Its stale Save returned 409 with a link to the saved revision. The archive and confirmation retained their identities and bytes, and the saved parent and discarded successor remained unchanged.

The captured ZIP differs from the committed 206 × 101 comparator only in Drivetrain's width, now 207. The model and manifest member bytes are identical. The scope was removed after all four services were reaped.

One earlier setup passed the wrong original PartDefinition name to the verifier and stopped before the race. A later admission attempt did not start the process because host I/O pressure exceeded the guard. Both attempts were retained privately.

From the repository root, verify the recorded projection and both committed archives.

```sh
python3 -IB porting/interop/evidence/archive-corrected-save-race/verify.py
python3 -OIB porting/interop/evidence/archive-corrected-save-race/verify.py
```

This observes one filesystem prototype race over loopback HTTP. Overlapping client windows establish competing requests. Kernel scheduling order, production database transactions, and power-loss behavior remain open.
