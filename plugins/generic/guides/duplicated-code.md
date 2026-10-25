### duplicated-code

Duplicated code blocks were detected across the following locations:

{% include "includes/line_level_issues.md" %}

**Why this matters:**
Copy-paste duplication leads to shotgun surgery when business logic changes, creates subtle divergence bugs when one copy is patched but another is forgotten, and bloats the codebase.

**Coaching guidance:**
- Identify the common abstraction between the duplicate fragments.
- Extract shared helper functions, utility modules, or reusable parameterized abstractions.
- Replace duplicate call-sites with the shared implementation.
