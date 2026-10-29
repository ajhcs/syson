The AST adapter had no operation applying the retained OperatorExpression
postprocessor. This adds structural postprocessing on synthetic kernel nodes:
falsey operands normalize to an empty array, and an empty array with one of the
four exact classification operators gets the source-shaped three-node implicit
feature-reference chain. Existing unary/binary/nested operand order is preserved;
repeated application is idempotent.

F09's actual registry selects only this handler. A kernel transaction rolls back
structural writes and new nodes on failure, while spent IDs/work retain the
kernel policy. Native child-cache aliases and immediate parent mutation remain
unsupported; structural parenting finalizes through the immutable kernel arena.
This is not an expression-text parser or execution of the other four callbacks.

Validation: 25 tests pass (12 new operator tests and 13 inherited F11), plus
formatting and all-target Clippy offline/locked. Three retained source documents,
19 associated static obligation tuples, 38 executed input hashes and original
streams/runner are recorded. Those tuples are not classification grammar
coverage; F08's ancestor-order qualification and upstream-runtime limits remain.

The PR targets the frozen F09/F11 prerequisite snapshot. Land both prerequisites
(#35 and #40) before retargeting to integration, inspecting the new diff and
running appropriate checks. Never merge into the snapshot branch. No held writer
path, package baseline, service or integration branch is changed.
