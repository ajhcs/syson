# Offline JaCoCo plugin dependency collection

Maven 3.9.16's plugin resolver invoked Resolver 1.9.27's depth-first collector for JaCoCo 0.8.14. The collector built descriptors from frozen POMs with the pinned ModelBuilder and Maven's descriptor conversion. The harness denied network access at the kernel and stopped at the artifact-resolution call boundary. The plugin's own model from the parent draft supplies the nine direct declarations.

Baseline and CI produced the same 13-node graph, including the plugin root. The collector visited 14 POM descriptors. Its selector excluded all three provided declarations before reading their descriptors. The graph retained direct Plexus Utils 3.0.24 over the transitive 4.0.2 descriptor and retained the JaCoCo agent's `runtime` classifier. Withholding file-management 3.2.0 or supplying a synthetic version range blocked collection before artifact resolution. The original compiler refusal and subsequent mismatch between the probe's exception expectation and Maven's debug-dependent wrapping are retained in the private run history. Both positive trace hashes remained identical after tightening the negative-control check.

A separate path inventory found regular files for three of those 13 artifact coordinates in the 16 inspected trees. Ten coordinates had no matching file. This inventory establishes path presence only. Binary validation, acquisition, execution, transitive mediation elsewhere in the target, and the pinned native compile remain open.

## Verify the recorded projection

Run both commands from this directory.

```sh
python3 -IB verify.py
python3 -OIB verify.py
```

These commands verify the sanitized observation against the preceding model evidence using the checked-in JSON files.
