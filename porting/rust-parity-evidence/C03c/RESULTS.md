# C03c results

Coordinator c03c-01 passes nine integration tests, one prepared-only compile-fail doctest, formatting and all-target Clippy on 62 listed unchanged inputs. The codec lock has exactly the same 11 (name, version, checksum) registry tuples as its C03b parent. Source-only independent review accepted the frozen implementation/tests without reviewer execution.

Real C03b prepare→encode→decode→prepare tests preserve authoritative payload bytes, all six independent stamp families and replay eligibility. Foreign-first namespaces keep pending nulls; unrelated representation kinds keep future stamps. Inline RawValue retains opaque large integers, Unicode, interior whitespace and duplicate payload members. Required-null versus omitted metadata, duplicate/unknown fields, future format, wrong types, corruption/trailing data and exact 1 MiB/8 MiB/item/metadata/envelope boundary failures are exercised. Escaped-content output stays proportional in its concrete test; no elapsed-time or SLA claim follows.

These are local candidate transport bounds, not native capacity or archive compatibility. Decode produces untrusted ProjectInput for actual C03b admission. C01/C03/C06/C07, SQL, durable publication and native history remain open.

The inherited C03b fixture is immutable in parent 7f111c24 and its current bytes match that Git object. Its hash was checked separately after execution because the 62-input capture did not list that include_str payload; this supplemental binding is not mislabeled as a pre-execution capture. The original CONTRACT.md remains unchanged as a pre-execution contract.
