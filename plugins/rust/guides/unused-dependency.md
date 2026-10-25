### unused-dependency

Unused dependencies declared in Cargo.toml detected:

{% include "includes/line_level_issues.md" %}

**Why this matters:**
Unused dependencies inflate compilation times, increase binary size, expand supply chain attack surface, and complicate dependency updates.

**Coaching guidance:**
- Remove the declared crate from `[dependencies]` or `[dev-dependencies]` in `Cargo.toml`.
- If used only in specific features, scope under `[features]` or `[target]`.
