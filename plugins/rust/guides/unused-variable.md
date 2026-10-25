### unused-variable

Unused local variables or bindings detected:

{% include "includes/line_level_issues.md" %}

**Why this matters:**
Unused variables indicate incomplete refactorings, dead code paths, or logic errors where a calculated value was meant to be used but forgotten.

**Rust Idioms & Coaching:**
- Remove the variable if its value is unnecessary.
- Prefix with `_` (e.g. `_unused`) only if required to satisfy a trait signature or tuple pattern without consuming.
