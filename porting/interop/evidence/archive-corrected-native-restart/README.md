# Reopen the native corrected project after a database-clone restart

A fresh SysON v2026.9.3 backend started from an independent copy of the
cleanly shut down PostgreSQL database from the
[corrected archive import](../archive-corrected-native). The original
database retained all 1,030 files and 26 directories. The server opened
only the copy, whose inodes were distinct. No ZIP was uploaded during
this run.

The startup export and the export after opening fresh Chromium
retained all 452 native identities and the complete project state.
The browser rendered Drivetrain at (9, 46), sized 206 × 101, with nine
diagram nodes. Its measured position and size agreed with the saved
layout within 0.01 model units. The export preserved the user-resized
flag. Native initialization submitted one GraphQL mutation, and the
exported state remained identical.

Both services and their listeners shut down cleanly. This covers one
clean database-clone restart and one fresh Chromium context. The earlier
same-backend two-context run remains interrupted.

Run the offline archive and browser comparison from the repository root.

```sh
python3 -I -B porting/interop/evidence/archive-corrected-native-restart/verify.py
```
