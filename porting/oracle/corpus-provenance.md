# Sources for the differential corpus

The Phase 1 [corpus](corpus.json) has 61 cases at commit
`05a4be35d4842272d0249ab43b9e49295591ef1e`. Its 42 inline cases isolate
specific behaviors. The 19 files exercise full inputs from Eclipse SysON,
including its shipped Batmobile template.

| Cases | Origin | License |
| --- | --- | --- |
| 8 `p01`–`p08` cases | Synthetic parser audit reproducers, first committed with the [Phase 0 oracle](https://github.com/ajhcs/syson/commit/4b9f7cbcf3fde98859efc3b7e35c4fd93ca896e6) | [EPL-2.0](../../LICENSE) |
| 5 `s01`–`s05` cases | Synthetic small structural controls from the same [Phase 0 oracle](https://github.com/ajhcs/syson/commit/4b9f7cbcf3fde98859efc3b7e35c4fd93ca896e6) | [EPL-2.0](../../LICENSE) |
| 29 remaining inline cases | Synthetic minimal inputs for the [Phase 1 issues](https://github.com/ajhcs/syson/issues/269), added in [the corpus expansion](https://github.com/ajhcs/syson/commit/89886cf753521293e723bea0bdab2d1e54e93c7c) | [EPL-2.0](../../LICENSE) |
| 18 `ASTTransformerTest` files | [Eclipse SysON import tests at the Rust source pin](https://github.com/eclipse-syson/syson/tree/6ba2ac64e6b16ee2b05752ad55395eebb87d4910/backend/application/syson-sysml-import/src/test/resources/ASTTransformerTest) | [EPL-2.0 project notice](https://github.com/eclipse-syson/syson/blob/6ba2ac64e6b16ee2b05752ad55395eebb87d4910/NOTICE) |
| `templates-Batmobile` | [Eclipse SysON's bundled template at the Rust source pin](https://github.com/eclipse-syson/syson/blob/6ba2ac64e6b16ee2b05752ad55395eebb87d4910/backend/application/syson-application-configuration/src/main/resources/templates/Batmobile.sysml), attributed in the template to [MBSE4U's example](https://github.com/MBSE4U/dont-panic-batmobile) | [Apache-2.0](https://github.com/MBSE4U/dont-panic-batmobile/blob/main/LICENSE) |

The runner reads the 19 paths directly from the checkout. All 19 file
contents match Eclipse SysON commit
`6ba2ac64e6b16ee2b05752ad55395eebb87d4910` byte for byte. Use this check
from a checkout containing the pinned Git object.

~~~sh
python3 - <<'PY'
from pathlib import Path
from subprocess import check_output

import json

pin = "6ba2ac64e6b16ee2b05752ad55395eebb87d4910"
corpus = json.loads(Path("porting/oracle/corpus.json").read_text(encoding="utf-8"))
paths = corpus["paths"]
for path in paths:
    original = check_output(["git", "show", f"{pin}:{path}"])
    if Path(path).read_bytes() != original:
        raise SystemExit(f"Input differs from {pin}: {path}")
print(f"{len(paths)} file cases match {pin}; {len(corpus['inline'])} inline cases")
PY
~~~

Record the source revision and applicable license when adding a borrowed
input. Preserve existing notices. When an input remains external, record
its retrieval URL and content digest. A live reference result records what
the tested SysON release did with the input. The [oracle runbook](README.md)
identifies the release separately from the Rust source pin.
