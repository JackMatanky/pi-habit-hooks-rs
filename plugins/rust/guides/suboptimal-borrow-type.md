### suboptimal-borrow-type

Functions accepting concrete container references instead of slice types detected:

{% include "includes/line_level_issues.md" %}

**Why this matters:**
Accepting concrete container references like `&String`, `&Vec<T>`, or `&PathBuf` restricts API consumers. Callers with string literals (`"hello"`), fixed arrays (`[1, 2, 3]`), or slice references cannot pass them without forcing a heap allocation.

**Rust Idioms & Coaching:**
1. **Use `&str` instead of `&String`**:
   ```rust
   // Bad: forces caller to create a String
   fn parse_header(header: &String) { ... }

   // Good: accepts string slices, String references, and literals
   fn parse_header(header: &str) { ... }
   ```
2. **Use `&[T]` instead of `&Vec<T>`**:
   ```rust
   // Good: accepts slices, vectors, and fixed arrays
   fn sum(numbers: &[i64]) -> i64 { numbers.iter().sum() }
   ```
3. **Use `&Path` instead of `&PathBuf`**:
   Accept `&Path` or `impl AsRef<Path>` for flexible filesystem path arguments.
