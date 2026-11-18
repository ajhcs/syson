# Archive save recovery

A disposable Rust archive service recovered the same Jante Sport draft
in three fault conditions. The completed ZIP matched the exact successor
that SysON v2026.9.3 accepted in the preceding evidence packet.

- Two Chromium tabs opened the same draft. The first saved it and the
  second received a conflict page. The second tab followed **Open saved
  revision** and downloaded the winning archive. A fresh process served
  the same file.
- A file sync failure before publication left the checkpoint intact and
  created no final ZIP. The **Retry save** button submitted the same draft
  and reported the persistent failure. After a service restart without
  the fault, the draft reopened and saved.
- A directory sync failure left the new ZIP visible on disk. The mobile
  browser offered **Confirm saved revision** and withheld downloads. The
  archive, manifest, model, and diagram endpoints also refused direct
  downloads. After a forced restart, a confirmation write failed and the
  archive stayed pending. An explicit confirmation in a fresh process
  verified the same ZIP bytes and inode. A later restart served it.

The three Chromium runs observed 28–93 ms for 21 keyboard actions and
92–239 ms for 15 page openings. Those loopback ranges include runs
executed concurrently on one host. They describe this fixture and
environment.

Run the offline evidence replay from the repository root.

```sh
python3 -I -B porting/interop/evidence/archive-save-recovery/verify.py
```

The replay verifies response sequences and the projected file custody
evidence against the archived ZIPs. The lab used an owner-controlled
directory and injected sync errors. End-to-end application auth, touch
interaction, representative project load, and physical power-loss
recovery require separate qualification.
