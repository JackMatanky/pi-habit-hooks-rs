### cyclic-dependency

Circular module dependencies detected:

{% include "includes/line_level_issues.md" %}

**Why this matters:**
Circular dependencies tightly couple modules, making independent unit testing and reasoning impossible, and increasing compile times.

**Rust Idioms & Coaching:**
1. **Extract Leaf Module**: Move shared types, traits, or domain models into a leaf module that neither parent depends on mutually.
2. **Inversion of Control**: Introduce a trait seam where module A defines the trait and module B implements it, breaking direct mutual dependency.
