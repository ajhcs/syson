# Eclipse Filesystem default Maven descriptor reader

Maven 3.9.16's `DefaultArtifactDescriptorReader` read all 25 original Filesystem POMs at its default model validation level 0 and returned 93 direct dependency edges. The direct ModelBuilder route in #391 used validation level 30 and returned 90 edges from 24 models. For those 24 releases, the two runs returned identical ordered dependency tuples after projecting the ModelBuilder optional field to the boolean used by the descriptor reader.

The extra three edges came from Filesystem 1.9.500, whose unchanged POM declares `modelVersion` `4.0`. The default reader returned the Equinox Common, Equinox Registry, and OSGi compile dependencies, each with range `[3.2.0,4.0.0)`. The guarded offline session used Maven's default ignore-invalid and ignore-missing flags and reported zero invalid descriptor events. An out-of-cohort request for 1.6.1 exited 42 before the descriptor reader.

The first attempt emitted the full descriptor census, but its launcher required empty stderr and stopped on Maven's three-line SLF4J missing-binding and NOP fallback notice. The independent out-of-cohort control was still unrun. The successor accepted empty stderr or that exact 210-byte notice for the census, preserving refusal of any other stderr there. It reproduced the same census and notice byte for byte, ran the control, and confirmed cleanup.

Run both offline checks from this evidence directory.

```sh
python3 -IB verify.py
python3 -OIB verify.py
```

The verifier reconstructs both stdout traces and the exact notice and compares the edge tuples with #391. The evidence covers direct descriptor reads with pinned POM-only callbacks. Graph collection, final selection, mediation, binary trust, and native compilation remain open. Raw POMs, host fields, and controller logs remain private.
