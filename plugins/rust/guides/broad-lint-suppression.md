### broad-lint-suppression

Broad or un-scoped lint suppressions detected:

{% include "includes/line_level_issues.md" %}

**Why this matters:**
File-level `#![allow(...)]` or broad module-level attributes blind compilers and linters across entire compilation units, allowing new violations to accumulate silently.

**Rust Idioms & Coaching:**
1. **Refactor First**: Address the underlying smell so suppression is unnecessary.
2. **Use `#[expect]` with Reason**: If suppression is truly necessary (e.g. FFI boundary or macro expansion edge case), use `#[expect(lint_name, reason = "...")]` scoped to the exact item or statement rather than `#![allow(...)]`.
