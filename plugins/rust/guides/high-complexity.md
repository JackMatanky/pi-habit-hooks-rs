### high-complexity

High cyclomatic or cognitive complexity detected:

{% include "includes/line_level_issues.md" %}

**Why this matters:**
Deeply nested logic, branches, and loop structures dramatically increase test surface area and the likelihood of regression bugs under edge cases.

**Rust Idioms & Coaching:**
1. **Flatten Pattern Matches**: Replace nested `if-let` / `match` chains with flat multi-pattern matches using match guards:
   ```rust
   match (event, state) {
       (Event::Click, State::Active) => { ... }
       (Event::Hover, _) => { ... }
   }
   ```
2. **Monadic Combinators**: Use standard iterator combinators (`filter_map`, `flat_map`, `fold`, `take_while`) instead of manual loops with mutable accumulator variables and nested conditions.
3. **Extract Single-Responsibility Helpers**: Break monolithic functions into private helper functions with clear inputs and outputs.
