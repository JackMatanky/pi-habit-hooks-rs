### undocumented-unsafe

Unsafe code blocks without explicit safety documentation detected:

{% include "includes/line_level_issues.md" %}

**Why this matters:**
`unsafe` code bypasses the Rust compiler's memory and type safety guarantees. Without a documented safety rationale, future maintainers and code reviewers cannot verify whether memory invariants (pointer validity, alignment, aliasing rules) are preserved.

**Rust Idioms & Coaching:**
1. **Add `// SAFETY:` Comment**: Every `unsafe` block must be preceded by a comment explaining why the preconditions are satisfied:
   ```rust
   // SAFETY: `ptr` is non-null, properly aligned, and points to an initialized `Header`
   // instance whose lifetime is guaranteed by `self.buffer`.
   unsafe { *ptr }
   ```
2. **Document `unsafe fn` Invariants**: For functions marked `unsafe fn`, add a `# Safety` section to doc comments specifying caller obligations.
3. **Minimize Scope**: Keep `unsafe` blocks as small as possible — encompass only the exact unsafe operation, not surrounding safe logic.
