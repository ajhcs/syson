# RR07 source preparation

This is an unexecuted source proposal. Root owns formatting, builds, runtime,
review receipts and publication. No test or compiler was run by this author.

The RR06 exporter retains native NameHelper quoting and adds the actual F11
lexer's shared `is_reserved_word` predicate. Root owns exposing that query over
the existing generated RESERVED authority, adding the direct path dependency,
and updating two historical refusal expectations in existing tests. No production
keyword copy, lexer decision, NameHelper behavior or importer limit is changed.
The quote flag is already shared by rendered declarations, UTF-16 candidate
length counting, qualified-reference segments and both RR06 rendering passes.

This is an intentional export improvement: a quoted keyword accepted as a name
by F14 can now be exported and reimported instead of being rendered bare and
rejected by the exporter's own importer gate. The standalone native NameHelper
continues to print its narrower keyword set as before; this does not claim a
complete native serializer/deresolver comparison.

A counting correction is required: the actual target lexer has 170 words and
NameHelper has 117, but the lexer-only set is **54**, not 53. `constant` belongs
only to NameHelper, so 53 was the net count difference, not the set difference.
The source-derived regression fixture enumerates all 54 extra words and verifies
the two source sets, the native-only word and their 171-word union. Production
uses the shared parser query and existing NameHelper set, not the fixture list.

Seven new tests are prepared: the quoted class counterexample; exact authority
sets; every extra word in declaration and all qualified-reference positions;
one 171-keyword actual-importer roundtrip; ordinary/case/Unicode/punctuation
controls; shortest-reference/CRLF/indentation behavior; and exact 65536-byte
output with added quotes plus a one-byte-over Editor rename refusal. Every
successful export is also passed through actual F14 import and re-export; tests
retain the input document unchanged. This is code/oracle preparation only.

Root must keep the original failed baseline counterexample, run the new suite
and affected original suites on exact final source, qualify allocation guards
from RR06, preserve failures and obtain independent review. No native runtime,
UI, full text-language or SysON feature acceptance is implied by this proposal.
