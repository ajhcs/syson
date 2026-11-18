# Escaped apostrophes in SysML names

Case p04-quoted-escape exposes two failures. Rust links the type but emits
the wrong declared name. SysON emits the same wrong name and loses the link.
Keep the case at DIFFER while Rust emits the wrong name.
[Tracking issue #253](https://github.com/ajhcs/syson/issues/253)

~~~sysml
package 'Road vehicle' {part def 'W\'heel';part axle:'W\'heel';}
~~~

## Expected result

KerML 1.0 §8.2.2.3 and Table 4 require escape decoding in unrestricted
names. The sequence `\'` represents one apostrophe. SysML 2.0 §8.2.2.1.2
inherits these lexical rules. The declared name is `W'heel`, and the
reference names that local definition under KerML §8.2.3.5.3–4.
[KerML 1.0](https://www.omg.org/spec/KerML/1.0/PDF#page=102)
[KerML name resolution](https://www.omg.org/spec/KerML/1.0/PDF#page=111)
[SysML 2.0 Part 1](https://www.omg.org/spec/SysML/2.0/Language/PDF#page=197)

## Observed result

The comparison used official SysON v2026.9.3, release source
b380dada415ce05155150f23cb3b74ae21cc45e4, JAR SHA-256
85c031352695d6ac8f059f0215ca493c58a16805a111233998dd443d6990c9e1,
and Rust at 05a4be35d4842272d0249ab43b9e49295591ef1e.

| Observable | Expected | SysON | Rust |
| --- | --- | --- | --- |
| Part definition name | `W'heel` | `W\heel` | `W\heel` |
| axle type | Links to its definition | Absent, with an unresolved-name warning | Links to its definition using its semantic ID |

The backslash is part of the emitted name, encoded as `"W\\heel"` in JSON.
Run the [differential oracle](../README.md) with `--only p04-quoted-escape`.
Inspect `PartDefinition.data.declaredName` and `FeatureTyping.data.type`
in the emitted `reference_document` and `rust_document`. The corpus source
SHA-256 is 7538f4423482cda4563d3864e0a4d39b575608ed5c25e5b4e8e90a0d7c8e9adf.
[Corpus](../corpus.json)

## Source trace

- SysON removes every apostrophe when translating name attributes. It
  places the reference spelling in a proxy URI fragment and passes the
  fragment to the namespace resolver. The final name-segment helper
  removes the outer quotes and leaves escape sequences in place.
  [Attribute translation](https://github.com/eclipse-syson/syson/blob/b380dada415ce05155150f23cb3b74ae21cc45e4/backend/application/syson-sysml-import/src/main/java/org/eclipse/syson/sysml/parser/translation/EAttributeTranslator.java#L239-L250)
  [Proxy creation](https://github.com/eclipse-syson/syson/blob/b380dada415ce05155150f23cb3b74ae21cc45e4/backend/application/syson-sysml-import/src/main/java/org/eclipse/syson/sysml/parser/NonContainmentReferenceHandler.java#L73-L82)
  [Proxy lookup](https://github.com/eclipse-syson/syson/blob/b380dada415ce05155150f23cb3b74ae21cc45e4/backend/application/syson-sysml-import/src/main/java/org/eclipse/syson/sysml/parser/ProxyResolver.java#L87-L118)
  [Name segment](https://github.com/eclipse-syson/syson/blob/b380dada415ce05155150f23cb3b74ae21cc45e4/backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/NamespaceImpl.java#L377-L392)
  [Quote stripping](https://github.com/eclipse-syson/syson/blob/b380dada415ce05155150f23cb3b74ae21cc45e4/backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/helper/NameHelper.java#L63-L75)
- Rust follows SysON's attribute translation when emitting the name. It
  resolves references from raw syntax through a separate linking path.
  [Stored name](https://github.com/ajhcs/syson/blob/05a4be35d4842272d0249ab43b9e49295591ef1e/rust/crates/syson-text-import/src/lib.rs#L199-L212)
  [Type linking](https://github.com/ajhcs/syson/blob/05a4be35d4842272d0249ab43b9e49295591ef1e/rust/crates/syson-text-import/src/lib.rs#L159-L169)

[Eclipse SysON #1219](https://github.com/eclipse-syson/syson/issues/1219)
tracks the escaped-apostrophe import failure. The related
[#1194](https://github.com/eclipse-syson/syson/issues/1194) tracks quoted
names containing `::`.

## Recommendation

Decode name tokens when forming model names and comparing qualified-name
segments. Keep the original spelling for source locations and diagnostics.
Require `W'heel` in the saved model and check that the type resolves to its
definition after save and reopen. Verify a literal backslash and a quoted
segment containing `::` as separate controls. Once Rust satisfies these
assertions, the oracle can identify the remaining difference against this
SysON release as expected. Resource-ID compatibility remains tracked in
[#248](https://github.com/ajhcs/syson/issues/248).
