### unhandled-unwrap

Bare `.unwrap()` or panic-inducing error extraction detected:

{% include "includes/line_level_issues.md" %}

**Why this matters:**
Calling `.unwrap()` causes an unrecoverable thread panic when an `Err` or `None` is encountered. In production services and CLI tools, panics cause dropped requests, ungraceful process exits, and lack diagnostic error chains.

**Rust Idioms & Coaching:**
1. **Propagate via `?` Operator**: Propagate errors up the call stack to a central handler:
   ```rust
   let file = File::open(path)?;
   ```
2. **Provide Fallback Defaults**: Use `unwrap_or`, `unwrap_or_else`, or `unwrap_or_default`:
   ```rust
   let timeout = env::var("TIMEOUT_MS").ok().and_then(|v| v.parse().ok()).unwrap_or(5000);
   ```
3. **Use `.expect()` for True Invariants**: If failure indicates a program bug (an invariant that should be logically impossible to violate), use `.expect("descriptive invariant explanation")` instead of bare `.unwrap()`.
