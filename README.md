# pi-habit-hooks-rs (`pi-habit-hooks`)

High-performance native Rust core runtime and Pi/OMP extension for [Habit Hooks](https://github.com/habit-hooks/habit-hooks) — a code-quality coaching engine translating tool-specific lint violations into a canonical smell vocabulary routed to outcome-oriented AI coaching prompts.

## Features

- **Native Rust CLI Runtime**: Zero-dependency standalone binaries (`habit-hooks`, `habit-sensors`, `habit-mapper`, `habit-snooze`).
- **Pi / OMP Extension**: Integrated `@earendil-works/pi-coding-agent` tools (`habit_hooks`, `habit_snooze`), slash commands (`/habit-hooks`, `/snooze`), status indicators, and system prompt guidelines.
- **Declarative Rust Plugin**: Full support for `clippy`, `messrust`, `cargo-modules`, `cargo-machete`, and AST comment analysis.
- **Snooze Engine with Lapse Tracking**: Automatic exemption expiration when the affected file's Git blob SHA changes.
- **Shipped Generic Plugin**: Duplicate code detection powered by `jscpd` via local package dependency (no global npm required).

## Installation & Build

```bash
# Build native binaries and TypeScript extension
npm run build

# Run all test suites (Rust tests, drift gate, and Node extension tests)
npm test
```

## CLI Usage

```bash
# Initialize configuration in a project
habit-hooks init

# Run coaching checks on working tree changes
habit-hooks

# Run coaching checks on all files in the project
habit-hooks --all

# Snooze active findings
habit-sensors --all | habit-snooze --snooze --reason "Legacy code"

# List active snoozes with lapse status
habit-snooze --list
```
