---
name: habit-hooks-review
description: Post-Habit-Hooks changeset review. Inspects git diff for semantic correctness, test completeness, boundary conditions, and idiomatic naming after Habit Hooks coaching passes cleanly.
---

# Habit Hooks Review Skill

## Overview
Habit Hooks enforces structural code hygiene (cyclomatic complexity, parameter count, file length, dead code, duplication, comment debt). Once Habit Hooks checks pass cleanly (`✅ Habit Hooks: automated checks passed`), use this skill to perform a semantic code review of the changed files.

## Workflow

### 1. Diff Inspection
Inspect changed files in scope:
- Run `git diff` on working changes or staged commits.
- Identify all modified interfaces, public structs, traits, and exported functions.

### 2. Semantic Correctness & Boundaries
Verify the logic changes:
- **Off-by-one errors**: Loop bounds, slice indexing, range expressions (`..` vs `..=`).
- **Error paths**: Ensure errors are returned with domain context (`?` with descriptive `thiserror` variants) rather than unwrap/expect in production paths.
- **Resource safety**: Check locks, file handles, and stream lifetimes for leak or deadlock risks.

### 3. Test Completeness
Ensure tests cover changed behavior:
- Every newly introduced branch or enum variant must have a corresponding test.
- Boundary conditions (empty collections, zero, maximum values) must be explicitly asserted.
- Tests must use `pretty_assertions::{assert_eq, assert_ne}` and descriptive verb-first naming (`[action]_[expected]_[condition]`).

### 4. Suppression Audit
Verify that no blanket lint suppressions (`#![allow(...)]`) were added.
If an `#[expect(lint, reason = "...")]` attribute is present:
- Confirm that the reason is technically accurate and specific.
- Confirm that the attribute is scoped to the smallest possible item or statement.
