# Read-only reference execution effect review

No confirmed blocker was found in the inspected bounded code path for the
exact prepared immutable corpus, subject to the execution conditions below.
This is code-effect preparation, not execution approval, a security scan,
license closure or a claim of observed runtime filesystem/network behavior.
No upstream code was executed by this review.

## Exact inputs

`manifest.json` pins the synthetic bytes, retained source hashes, candidate
`1326f234cae23b0e4777d5a48a069b17e9e332f1`, baseline
`6ba2ac64e6b16ee2b05752ad55395eebb87d4910`, AST plan revision/hash and reviewed
runner hash. The bundle SHA-256 is
`47d8fe11fda1b58b6257463087f663be953c2ab6c2b5e1e2ccc44161ed5cd0bd`.
The selected Node binary SHA-256 is
`7fde7b8afa198da66257f42ee2001d874c7355631e6d1579a5fb5ef1f246df4c`,
previously observed as v24.21.0 by the separate bounded reference run. This is
not the application's exact v24.20.0 engine pin. The runner must verify the
selected executable's exact bytes at invocation; this preparation did not
execute a fresh version probe.

## Existing and newly touched source paths

The actual pinned bundle was read directly, alongside the retained complete
candidate source and AST-VALUE-CONVERSION.md. Relevant bundle locations:

- CLI lines 114607–114629 concatenate the KerML and SysML extension lists and
  dispatch `dump` to `evalAction`. Metadata at 83693/83698 explicitly declares
  `.kerml`/`.sysml`; pc08 uses that existing route. Registry dispatch at
  113156 selects by URI. No new executable, plugin or server path is added.
- `extractDocument`, lines 114568–114603, checks suffix/existence, resolves the
  absolute fixture URI, gets one document and calls DocumentBuilder.build with
  `validationChecks: all` and `standardLibrary: none`. Node file provider
  113748–113778 reads file contents; it has directory-reading capability, but
  this route does not call workspace initialization/traversal.
- Builder lines 105726–105770 apply options and print statistics. Previously
  reviewed standard-library guards at 107724, 107890, 108109, 108124 and 108651
  skip corresponding implicit/metaclass library work for `none`. That flag is
  not a general download/filesystem prohibition.
- Newly exercised raw terminal conversion, default filling, assignments and
  actions are in-memory parser construction at 41020–41185 and conversion at
  41808–41920. Naming sanitation and CLI projection are later layers; their
  complete candidate source hashes are recorded without asserting full
  candidate-to-bundle runtime equivalence.
- Local and unresolved references exercise in-memory model/index resolution.
  `findGlobalElement`, lines 106385–106433, consults caches, exports and static
  scope members and tracks found document dependencies. The prepared names
  contain no URI/import request. No external model/library file is supplied.
- `extractDocument` exits on severity-one diagnostics before serializer entry.
  This makes recovered AST output unavailable in some negative cases; it does
  not make those cases safe to classify as absence of parser construction.

No source path inspected for these fixtures calls a filesystem write, package
manager, child process, download, listener or workspace initialization. The
expected application effects remain fixture reads, in-memory parsing/model
work and stdout/stderr emission; the Python parent additionally reads the
selected executable/bundle/script for hashes and captures/compresses streams.
The filesystem provider and other bundled capabilities exist outside the
specific path. Effects were not independently instrumented at runtime; source
review cannot prove no attempted effects.

## Conditions before any execution

1. Coordinator review must freeze the exact twelve-case manifest, fixture
   hashes, runner hash, command and bounds. Preserve reviewed files from
   concurrent mutation across the runner's checks and Node's later reads.
2. Invoke the exact selected Node/bundle in the **default managed
   network-restricted boundary**. Preparation or later evidence persistence
   may be separate actions; an unsandboxed escalation is not an equivalent
   oracle execution boundary. Do not retry outside it after rejection.
3. Keep fixed diagnostic flags, no imports/exposes, no extra datasets,
   credentials, dependencies, environment preloads or primary/default-library
   command substitution. The reviewed runner clears NODE_* but retains the
   other inherited environment; use the already reviewed runtime context.
4. Run serially, one explicitly listed fixture per process, at most 60 seconds
   each. Capture raw structured results once. No automatic retries, parallel
   processes or expansion beyond the twelve entries are prepared.
5. Verify decompressed stream byte lengths and SHA-256 against the runner
   report before persisting evidence separately. Retain raw/portable hashes
   separately if paths are redacted. A truncated tool result is not valid
   evidence. No hard stream-size or process-memory ceiling is supplied by this
   runner; do not claim one from the 1 KiB input bound.

Stop on changed/missing hashes, unsupported suffix/encoding, unreviewed case or
command, an unavailable restricted boundary or action-time approval rejection,
timeout, unexpected process/effect/resource behavior, or incomplete/corrupt
capture. Preserve the reported result; do not classify a negative reference
exit alone as a runner failure or retry it automatically. New effects or a
need to observe unavailable recovered AST require another bounded review.

Approval and invocation remain pending. All semantic outcomes, recovered ASTs,
wire values and Rust comparisons remain unobserved in this prepared corpus.
