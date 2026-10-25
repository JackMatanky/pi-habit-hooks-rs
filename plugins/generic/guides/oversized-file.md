### oversized-file

The following files exceed length thresholds:

{% include "includes/file_level_issues.md" %}

**Why this matters:**
Oversized files blur single-responsibility boundaries, make code review difficult, and increase merge conflicts in teams.

**Coaching guidance:**
- Separate domain entities, data serialization, and business logic into dedicated modules.
- Extract utility methods and characterization tests into companion test files or child modules.
