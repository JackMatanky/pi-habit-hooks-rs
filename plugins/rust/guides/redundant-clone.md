### redundant-clone

Excessive or unnecessary `.clone()` operations detected:

{% include "includes/line_level_issues.md" %}

**Why this matters:**
Cloning data structures (`Vec`, `String`, large structs) unnecessarily allocates heap memory, copies byte buffers, and wastes CPU cycles.

**Rust Idioms & Coaching:**
1. **Borrow Instead of Clone**: If the destination only needs to read or inspect the data, pass a reference `&T` instead of cloning to satisfy the borrow checker.
2. **Move via `mem::take` / `mem::replace`**: When extracting an owned value out of a mutable reference (`&mut T`), use `std::mem::take` (if it implements `Default`) or `std::mem::replace`:
   ```rust
   // Good: Moves buffer without allocating a new Vec
   let buffer = std::mem::take(&mut self.buffer);
   ```
3. **Reference-Counted Sharing**: For shared immutable state, clone the pointer wrapper `Arc::clone(&state)` rather than cloning the inner payload.
