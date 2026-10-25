# F11 finite package/part AST construction

`syson-ast-adapter` constructs an immutable `syson-ast` kernel arena from a finite
text family. It is a small frontend over source-derived grammar obligations, F08
mandatory metadata and F10 primitive conversion. It does not call a native parser,
resolve a type, mutate a SysML resource or claim complete SysML text import.

The accepted declaration family is:

```text
model        := declaration*
declaration  := 'package' name? (';' | '{' declaration* '}')
              | 'part' 'def' name? (';' | '{' declaration* '}')
              | 'part' name ':' name ('::' name)* ';'
```

Names are ASCII regular identifiers admitted by a conservative frozen keyword
exclusion, or raw `UNRESTRICTED_NAME` quoted strings using the retained terminal's
escape set. Whitespace outside names is ASCII whitespace. Quoted names can contain
Unicode scalars. Quotes and escapes remain in `declaredName`; F10's later name
sanitation is deliberately separate. Unpaired UTF-16 input cannot be represented
by Rust `str`. The static keyword union may reject identifiers a native runtime
would admit; it is an explicit Rust admission policy, not a native lexer result.

The root is an anonymous `Namespace`. Root/package bodies wrap every selected
declaration in `OwningMembership`. Inside a `PartDefinition`, selected part usages
use `FeatureMembership`; nested definitions/packages still use `OwningMembership`.
This distinction follows `UsageMember` versus `OccurrenceUsageMember` in the
retained grammar. All wrapper targets and encounter-ordered children are distinct
nodes. Typing adds a `FeatureTyping` heritage node containing a `TypeReference`;
qualified names stay unresolved rather than becoming semantic links. The grammar's
static cross-reference target is `Element`; F08's custom positional expectation
is `Element` for earlier segments and `Type` for the final segment. The kernel's
`target_type` marker retains that expectation. It is adapter metadata, not a native
AST field or a resolved reference.

Whole-node and primitive lexemes retain half-open byte and UTF-16 spans in the
same immutable input. Mandatory arrays and booleans come from F08; optional absent
fields remain absent. The kernel derives parent/property/index relationships at
freeze. F08's ancestor ordering remains an unqualified source-derived short-sort
candidate, exposed by `ordering_qualification()`. Child caches, parser callbacks,
CST identities, document state and native `$container` objects are not reproduced.
The kernel's `$children` default stays empty; it is not advertised as a computed
native child cache.

Malformed finite-family input returns an offset and expected component. Syntax
outside the family returns `Unsupported`, including modifiers, comments, short
names, untyped usages, usage bodies and general expressions. Such an outcome does
not assert that the full language rejects the text. No recovery arena or partial
construction escapes a failure.

Input bytes, tokens, declaration count, recursive depth and kernel node/work
budgets are explicit. Depth is limited to at most 64 by Rust admission, not by a
native grammar limit. Cancellation is checked before construction, during lexical
and recursive processing, and through the kernel. These logical limits are not a
CPU deadline, process memory cap or crash guarantee.

The [source manifest](source-manifest.json) records four retained documents,
14 finite assignment/reference obligations selected from the 848-entry source
inventory, whole-input hashes and 170 generated keyword exclusions. This is a
whitelist, not a source inventory completion claim. The source inventory's native
execution/qualification requirements remain unmet and unchanged. Fixture expected
shapes are manually source-derived; no JavaScript/native AST golden was run.

Reproduce source preparation without compiling:

```sh
python3 -B porting/follow-ons/F11/prepare.py --check
```

Coordinator-run checks use the existing offline toolchain and exact local path
dependencies. Their original commands/streams and frozen input hashes are recorded
in `VALIDATION.json` and `validation/`. The initial zero-keyword preparation draft
was rejected and corrected before compiler execution. Its original standalone file
was not independently retained; the exact write-command reconstruction and short
verbatim tool output are labeled honestly in `validation/preparation-01`.

This PR targets the immutable F08+F10 dependency snapshot. Land both prerequisites
first, then retarget to merged integration, inspect the new diff and rerun relevant
checks. Do not merge this PR into the temporary snapshot branch. Existing writer
holds, shared manifests and the execution backlog are unchanged.

The final `f11-02` packet passes all 13 Rust tests, formatting and all-target Clippy.
The preceding `f11-01` packet retains its passing tests and overall lint failure;
only two expression-form refactors separate its source from the final source.
`source-freeze.json` is the historical checkpoint before the first compiler run;
actual final source hashes and complete captured dependency inputs are in the
final receipt and `VALIDATION.json`. Thirteen test functions are counted once,
not doubled by the two executions.
