### change-risk-hotspot

High churn combined with high complexity identified:

{% include "includes/line_level_issues.md" %}

**Why this matters:**
Areas of high complexity that are frequently modified represent the greatest source of regressions in production software.

**Rust Idioms & Coaching:**
1. **Characterization Tests**: Write thorough unit and property-based characterization tests capturing existing behavior before refactoring.
2. **Decompose Responsibility**: Break down the hotspot into smaller, pure functions that can be tested in isolation with zero mocking.
