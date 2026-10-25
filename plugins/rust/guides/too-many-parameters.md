### too-many-parameters

Functions taking too many parameters were detected:

{% include "includes/line_level_issues.md" %}

**Why this matters:**
Functions with more than 4 parameters increase cognitive load at call-sites, are prone to argument-order bugs (especially with primitive types), and complicate refactoring.

**Rust Idioms & Coaching:**
1. **Config / Parameter Struct**: Group related arguments into a dedicated struct (e.g. `struct RequestOptions { ... }`) implementing `Default` where appropriate.
2. **Builder Pattern**: For complex construction with optional settings, implement a typed Builder pattern (e.g. `ServerBuilder::new().port(8080).build()`).
3. **Context Objects**: If multiple functions share the same environment parameters (database pool, logger, configuration), pass a single shared context reference `&AppContext`.
