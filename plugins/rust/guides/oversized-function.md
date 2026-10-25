### oversized-function

Functions exceeding maximum length threshold detected:

{% include "includes/line_level_issues.md" %}

**Why this matters:**
Functions longer than 50 lines often interleave multiple concerns (parsing, validation, business rules, I/O), making them difficult to understand, test, and maintain.

**Rust Idioms & Coaching:**
- Extract discrete steps into private helper functions with meaningful names.
- Encapsulate data transformations into separate domain methods or traits.
