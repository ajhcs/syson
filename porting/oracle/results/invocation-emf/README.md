# SysON invocation conversion and EMF reload

Eclipse SysON v2026.9.3 converted three archived Syside ASTs into EMF models. Each model contained one `InvocationExpression` whose owned `FeatureTyping` linked it to the declared target in the same resource. `Feature.getType()` returned that target. The instantiated-type operation and getter, `getFunction()`, and `getResult()` returned null in all three cases. The recorded projections matched byte for byte after each model was serialized to JSON and reloaded into a fresh ResourceSet. [Parser fixtures and CLI evidence](../constructor-invocation/README.md) · [Eclipse SysON v2026.9.3](https://github.com/eclipse-syson/syson/releases/tag/v2026.9.3)

| Fixture | Target stored in `FeatureTyping` | Additional observation |
| --- | --- | --- |
| Concrete `A()` | `PartDefinition A` | `A.isAbstract` is false |
| Abstract `A()` | `PartDefinition A` | `A.isAbstract` is true |
| Typed calculation `F()` | `CalculationDefinition F` | Serialized return membership owns `r` typed by `AttributeDefinition N`. `F.getResult()` returned null. |

Each fixture had zero transformation messages and zero EMF resource errors or warnings. The harness called the released `ASTTransformer.convertResource` with the previously captured bare AST and the release's default libraries. It saved the EMF JSON before reading semantic getters, then loaded the saved bytes with the release's JSON resource factory and another library ResourceSet. The release classpath contained 312 index-ordered JARs, each checked against its embedded release entry. A kernel socket-denial self-test and separate process receipts cover the offline run. [Converter source](https://github.com/eclipse-syson/syson/blob/b380dada415ce05155150f23cb3b74ae21cc45e4/backend/application/syson-sysml-import/src/main/java/org/eclipse/syson/sysml/ASTTransformer.java)

The upstream CLI evidence establishes parsing and CLI diagnostics for these inputs. This run establishes the converter's link and derived-getter behavior for the same three positive parser fixtures. Editing-context AQL validation, SysON application upload and document creation, direct EMF construction, expression evaluation, and Rust importer behavior remain outside this result. [Editing-context validation rules](https://github.com/eclipse-syson/syson/blob/b380dada415ce05155150f23cb3b74ae21cc45e4/backend/application/syson-sysml-validation/src/main/java/org/eclipse/syson/sysml/validation/rules/SysMLValidationRule.java)

## Verify the retained output

Run both commands from the repository root with Python 3. The verifier reads the recorded parser inputs, serialized JSON, and conversion and reload projections. It checks the returned class, both `FeatureTyping` endpoints, resource fragments, semantic element IDs, and the calculation return parameter. It also rejects six changes to the saved or reloaded observations. [Recorded output](observation.json)

```sh
python3 -IB porting/oracle/results/invocation-emf/verify.py
python3 -OIB porting/oracle/results/invocation-emf/verify.py
```
