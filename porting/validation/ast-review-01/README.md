# Independent native AST-kernel run

Ten public-operation tests passed against exact implementation
`9e2337e4ec0a74c675f9e53c98df008a3dc50036` and independent harness
`183d77bf06e7a40a926cf73aefac918c0742c3bf`.
`receipt.json` records actual UTC, exact executable source hashes, command,
exit and raw/portable stream hashes. No upstream code was executed.

These tests establish bounded operation behavior and verify the ownership and
flat-array fixes. They do not qualify parser integration, grammar derivation
selection, AST wire equality or semantic services. All source dispositions
remain UNPORTED. See `../M8-INDEPENDENT-REVIEW.md` for findings and limitations.
