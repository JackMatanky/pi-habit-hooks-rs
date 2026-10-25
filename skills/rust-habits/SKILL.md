---
name: rust-habits
description: Deep three-view architectural audit for Rust codebases: Topology (cargo-modules), Ownership (LSP/borrows), and Hotspots (messrust/cyclomatic complexity).
---

# Rust Habits: Three-View Architectural Audit

## Overview
Perform comprehensive codebase health audits by examining Rust systems across three distinct dimensions:
1. **Topology View**: Module graph structure, orphan files, and circular dependencies.
2. **Ownership & Lifetimes View**: Borrow ergonomics, unnecessary allocations, clone avoidance.
3. **Hotspots & Churn View**: High complexity functions intersecting with high Git churn.

## 1. Topology View (Module Graph)
Examine module relationships and structural cohesion:
- **Orphan File Detection**: Run `cargo modules orphans` to ensure every `.rs` file in `src/` is connected to the module hierarchy.
- **Cycle Detection**: Run `cargo modules dependencies --lib --acyclic` to verify that module dependencies form a directed acyclic graph.
- **Remediation**:
  - Extract common domain types into a leaf module.
  - Introduce trait seams to decouple interdependent modules.

## 2. Ownership & Lifetimes View (Ergonomics)
Audit memory efficiency and borrow patterns:
- **Clone Audit**: Search for `.clone()` calls. Verify whether ownership is truly required or if borrowing `&T` suffices.
- **In-place Updates**: Replace `.clone()` on mutable references with `std::mem::take` or `std::mem::replace`.
- **Iterator Laziness**: Eliminate intermediate collections (`.collect().into_iter()`). Chain adapters lazily and collect once.
- **Boolean Method Naming**: Ensure predicate methods returning `bool` start with `is_`, `has_`, `can_`, or `should_`.

## 3. Hotspots & Complexity View (Quality Gates)
Identify code areas prone to regression:
- **Cyclomatic Complexity**: Run `messrust <files> json codesize,design` to highlight functions with complexity > 10.
- **Parameter Lists**: Refactor functions with > 4 parameters into dedicated configuration structs or builder patterns.
- **Characterization Tests**: Before refactoring a hotspot function, write characterization tests covering all existing behavior.
