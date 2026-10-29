# Primitive conversion and borrowed-summary development comparisons

These development tools compare the public Rust primitive converter with original methods in the retained Syside bundle. They are separate from application delivery. The oracle evaluates the byte-identical module-definition prefix before the CLI entry and calls Langium's `DefaultValueConverter.convert`, then Syside's `sanitizeName` and `unsanitizeName` for their respective cases. It does not replace those methods or infer expected values from Rust.

The coordinator runs the frozen oracle with explicit absolute paths:

```text
node values_oracle.cjs ABSOLUTE_BUNDLE ABSOLUTE_CORPUS ABSOLUTE_NEW_OUTPUT
```

The output must be a new file on `/workspace`; existing outputs are refused. Bundle SHA-256, the two original module hashes, the unique excluded entry boundary, evaluated prefix, probe, corpus, oracle and actual Node executable are recorded. Each VM evaluation has a 30-second timeout. The harness invokes no CLI, parser, network operation, subprocess or package manager. Original barrel imports can import Node builtin I/O modules.

`values_corpus.json` has schema version 1 and a `cases` array. Every case has a unique `id` and an `op` of `convert`, `sanitize` or `unsanitize`. Input `text`, when present, must contain valid Unicode and therefore be representable as Rust UTF-8. Only `sanitize` may omit `text`, meaning native `undefined` and Rust `None`. Conversion cases name `dispatch` as `raw`, `unresolved` or `rule`; rule dispatch supplies `rule` and optionally `declared_type`. The oracle supplies an explicit terminal-rule descriptor and a minimal CST feature to the original converter. This exercises supplied dispatch, not real grammar/CST resolution.

Comparable output is `observations: [{id, value}]`. String values use `{kind: "string", units: [...]}` with one integer per UTF-16 code unit. Number values use `{kind: "number", bits: "16 lowercase hex digits"}` in big-endian order. Boolean and undefined values use `{kind: "boolean", value: ...}` and `{kind: "undefined"}`. Original exceptions use `{kind: "error", name, message}`. Strings undergo no Unicode or surrogate normalization. DataView captures every numeric result, including signed zero, infinities and observable NaN sign/payload bits; differences remain differences rather than being silently canonicalized.

Cases with `expected_restriction` appear in `restrictions`, with their original `native_value` recorded separately from the expected Rust error kind. Native BigInt values use `{kind: "bigint", decimal}`; native Date values use `{kind: "date", time_bits, valid}`. Unresolved rules and non-ASCII dispatch are also separate restrictions. A deliberate Rust refusal is not converted into a matching JavaScript return or exception.

The corpus retains the original 16-test suite's comparable case intent and adds numeric rounding, subnormal/overflow, radix guard/sticky/carry, each of the 25 JavaScript whitespace code points, malformed prefixes, signed NaN, Unicode slicing, dispatch precedence and name escaping cases. Lone-surrogate outputs arise from slicing valid astral input; the corpus itself has no lone-surrogate input. Original Rust checks remain necessary for `Presence::Null`/`Undefined`, byte-span association, budgets, cancellation and empty-name output limits. The Java literal-kind predicate and real importer also require separate evidence; the corpus includes distinct numeric spellings without claiming to exercise that Java path.

Oracle output is a pinned-bundle observation on the recorded current Node/V8 runtime. It does not bind the native application's Docker runtime, the whole deployed artifact, candidate TypeScript equivalence, consuming parser/importer workflows, durable lifecycle, resource capacity, performance or feature acceptance. Executed results, original failures and exact source bindings are retained separately in the main-batch18 evidence packet.


`values_dump.rs` calls the public Rust API on the same corpus and records source lexemes and byte spans separately from comparable values. `compare_values.py` compares ordered case IDs, exact typed results, floating-point bit strings and UTF-16 arrays; Rust refusals remain separate from native returns or exceptions. The original signed no-digit INT NaN discrepancy and its repaired capture are retained. Four actual CLI controls reject changed boolean types, signed-zero bits, UTF-16 units and case order. These selected controls do not establish complete JSON-schema or consuming-caller validation.

`summary_probe.rs` accepts an absolute dense-fixture path and `owned` or `borrowed`. It parses outside the measured interval, calls the actual model summary API, then checks the same five-field content hash outside API timing. Requested allocation counters describe the main thread's logical live requests; a realloc transition does not measure the allocator's transient overlap or metadata. Linux RSS/HWM are separate process observations. The coordinator records two warmups and seven fresh-process measurements per mode in alternating order. A zero drop counter is scoped bookkeeping, not leak proof. These results do not establish rendered UI latency, native equivalence, universal capacity or parent RR18b3a acceptance.

The executed coordinator recipes, exact manifests/locks, raw streams, compiled probe and before/after input maps are retained in the evidence archive. Existing native runtime/source and ownership gates remain mandatory. This tooling is excluded from the production application.
