# Private part definitions in qualified type references

A sibling package's PartUsage can name a public PartDefinition through its package. In the released Syside linker, changing that definition to private leaves the final name segment unresolved. A PartUsage in the declaring package still resolves the same private definition.

| Fixture | Released AST | Saved SysON model and converter report | Isolated Rust probe |
|---|---|---|---|
| Sibling package, private A::T | Package A found, T unresolved | FeatureTyping owned by x with typedFeature set, type omitted, one unresolved warning | Resolves and stores A::T |
| Sibling package, public A::T | A::T resolved | FeatureTyping has both endpoints, no warning | Resolves and stores A::T |
| Declaring package, private T | A::T resolved | FeatureTyping has both endpoints, no warning | Resolves and stores A::T |
| Sibling package, missing A::Missing | Package A found, Missing unresolved | FeatureTyping owned by x with typedFeature set, type omitted, one unresolved warning | Refuses at linking and reports no model |

The native comparison used the SysON v2026.9.3 CLI and released Java ASTTransformer. Each conversion received its own standards resource set and saved an EMF resource for inspection. The Rust side used the isolated visibility probe qualified in #340. Both native unresolved cases retain the edge without a type target. Changing Rust to reject the private lookup would prevent an incorrect reference but would still leave the converter's partial model and warning behavior open.

The [observation](observation.json) includes the exact four synthetic sources, checksums of raw captures, stored edge outcomes and exact warning text. The commands verify the archived projection. Live acquisition, app import responses, validator diagnostics, other name resolution or access rules and default importer adoption remain open.

~~~sh
python3 -IB porting/oracle/results/private-qualified-part-type/verify.py
python3 -OIB porting/oracle/results/private-qualified-part-type/verify.py
~~~
