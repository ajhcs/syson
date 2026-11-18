# Guarded verification of a current-source migration fixture

The Rust composition verifier checked an isolated same-base Git fixture built
from pinned current `main` migration inputs. The fixture had four disjoint
source commits, 281 selected files, 59 unchanged inputs and one generated
export file. The four exports reproduce the committed module file byte for
byte. The verifier used the six exact files from the guarded RR14 head.
[#77 source verifier](https://github.com/ajhcs/syson/pull/77) · [#96 guarded reader](https://github.com/ajhcs/syson/pull/96)

Formatting, all 15 named tests, all-target Clippy and the offline locked build
passed. The ordinary manifest and an ancestor directory alias produced the
same report. Six explicit controls refused stale refs, omitted owned files,
reordered exports, wrong blobs, a final symlink and a FIFO. The FIFO refused
in 3.039 ms within its two-second deadline. The kernel witness allowed the
anonymous local process handshake and refused external socket access. Git
refs, HEAD, index and configuration matched before and after the run. The
owned scope was empty or absent before the compiler lock was released.

The older same-base proof covered 93 selected files and one generated export file.
A separate ordered rehearsal checked the actual first-parent history of the
25-file migration crate through six merge transitions at the same pinned
main. The synthetic fixture exercises the existing verifier on current bytes.
Source adoption and the ordered verifier registration retain their existing
review gates.

Verify the sealed observation and its 12 record controls from this directory.

```sh
python3 -IB verify.py --self-test
python3 -OIB verify.py --self-test
```

The observation records the local receipt digests. The verifier checks this
projection offline. Reproducing the compiler and Git run requires its
corresponding source and fixture inputs.
