### swallowed-exception

Silent error discard or wildcard match detected:

{% include "includes/line_level_issues.md" %}

**Why this matters:**
Swallowing errors (e.g. `let _ = ...`, `match res { Err(_) => () }`) hides root causes of production bugs, leaves resources in inconsistent states, and eliminates diagnostic breadcrumbs.

**Rust Idioms & Coaching:**
1. **Propagate Errors**: Use the `?` operator to propagate errors up the call stack to where they can be properly handled or logged.
2. **Typed Domain Errors**: Define domain errors using `thiserror`:
   ```rust
   #[derive(Debug, thiserror::Error)]
   pub enum ServiceError {
       #[error("database connection failed: {0}")]
       Db(#[from] sqlx::Error),
   }
   ```
3. **Explicit Handling**: If ignoring an error is genuinely intended by design, document the rationale in a comment and log at debug/trace level rather than silently dropping it.

### Recommended Fix Command
Run compiler auto-fixes on working changes:
```bash
cargo clippy --fix --allow-no-vcs
cargo fmt
```
