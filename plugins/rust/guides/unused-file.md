### unused-file

Orphan source files unreferenced by any module hierarchy detected:

{% include "includes/line_level_issues.md" %}

**Why this matters:**
Files that are not linked into the crate tree via `mod <name>;` are not compiled, tested, or linted by `cargo`, leading to confusion and dead code.

**Coaching guidance:**
- Add `mod <file>;` to `lib.rs`, `main.rs`, or the appropriate parent `mod.rs`.
- If the file is obsolete, delete it to keep the repository clean.
