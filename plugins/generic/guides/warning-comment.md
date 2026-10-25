### warning-comment

Unresolved warning markers (TODO, FIXME, HACK, todo!(), unimplemented!()) detected:

{% include "includes/line_level_issues.md" %}

**Why this matters:**
Unresolved TODOs and temporary hacks left in non-test production paths risk runtime panics, incomplete features, and unaddressed technical debt.

**Coaching guidance:**
- Complete the required logic if in scope for this turn.
- If deferred, file a tracked issue with clear reproduction steps and link the issue URL in the comment.
- Replace `todo!()` with explicit typed errors or fallback paths.
