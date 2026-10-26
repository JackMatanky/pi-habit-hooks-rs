### non-const-binding

Variable binding does not need to be mutable:

{% include "includes/line_level_issues.md" %}

**Why this matters:**
Variables marked `mut` that are never modified weaken compile-time reasoning, signal accidental intent, and complicate concurrency analysis.

**Rust Idioms & Coaching:**
- Remove the `mut` keyword from the binding (`let x = ...` instead of `let mut x = ...`).
- Constrain mutation to the smallest possible lexical scope.

### Recommended Fix Command
Run compiler auto-fixes on working changes:
```bash
cargo clippy --fix --allow-no-vcs
cargo fmt
```
