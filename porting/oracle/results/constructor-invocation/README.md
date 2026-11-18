# Constructor syntax and invocation in the SysON text parser

The bundled Syside CLI recovered a partial AST for both `new A()` fixtures. It read `new` as an unresolved feature reference and stopped the part value before the following `A()`. The separate validation command reported the same syntax error first for the concrete and abstract targets. With the `new` prefix removed, the CLI parsed `A()` as an `InvocationExpression`, linked it to `P::A`, and returned no validation diagnostics for either target.

| Value and target | Bare dump | Separate validation with no standard library |
| --- | --- | --- |
| `new A()`, concrete part definition | Exit 0 with a recovered AST ending the value at `new` | Exit 1. First syntax error at `A` |
| `new A()`, abstract part definition | Exit 0 with the same recovery and an abstract definition marker | Exit 1. Same diagnostics as the concrete case |
| `A()`, concrete part definition | Invocation linked to `P::A` | Exit 0 |
| `A()`, abstract part definition | Invocation linked to `P::A` with an abstract definition marker | Exit 0 |
| `F()`, calculation with untyped fixture features | Invocation linked to `P::F` | Exit 1. Three feature-typing diagnostics |
| `F()`, calculation and usage typed by the local `N` | Invocation linked to `P::F` | Exit 0 |

The CLI has SHA-256 `47d8fe11fda1b58b6257463087f663be953c2ab6c2b5e1e2ccc44161ed5cd0bd` in both the pinned SysON source and the v2026.9.3 source. The two offline runs used the identical bundle and kept the bare and validation results separate, matching the two command contexts of the SysON upload adapter. The adapter's source uses a leading syntax diagnostic to warn of a partial import. The editing-context AQL rules separately constrain invocation targets to behaviors or features typed by a behavior. Both observations about the Java application come from source inspection. Converter output for these inputs, execution of the AQL rules, constructor evaluation, and Rust importer behavior remain unverified in this comparison. [Pinned CLI](https://github.com/eclipse-syson/syson/blob/6ba2ac64e6b16ee2b05752ad55395eebb87d4910/backend/application/syson-sysml-import/src/main/resources/syside-cli.js) · [Upload adapter](https://github.com/eclipse-syson/syson/blob/6ba2ac64e6b16ee2b05752ad55395eebb87d4910/backend/application/syson-sysml-import/src/main/java/org/eclipse/syson/sysml/SysmlToAst.java#L147-L207) · [AQL rule](https://github.com/eclipse-syson/syson/blob/6ba2ac64e6b16ee2b05752ad55395eebb87d4910/backend/application/syson-sysml-validation/src/main/java/org/eclipse/syson/sysml/validation/rules/SysMLValidationRule.java#L1509-L1523)

Use Python 3.10 or newer to verify the stored fixture bytes, parsed-expression projections, and diagnostic channels from the repository root.

```sh
python3 -IB porting/oracle/results/constructor-invocation/verify.py
python3 -OIB porting/oracle/results/constructor-invocation/verify.py
```
