# Data Model: Lefthook Pre-commit Hooks

**Feature**: Lefthook Pre-commit Hooks (008)
**Date**: 2026-09-03

---

## Overview

This feature is infrastructure configuration with no runtime data model. The "model" is the `lefthook.yml` configuration structure. This document defines the schema and validation rules for that configuration.

---

## Configuration Structure

### `lefthook.yml` Schema

```yaml
# Root: hook types map
pre-commit:       # Runs before commit is finalized
  parallel: true  # Run commands concurrently
  commands:
    <name>:       # Command identifier
      glob: "*.rs"                    # File filter (optional)
      run: <command>                  # Shell command to execute
      stage_fixed: true               # Auto re-stage modified files (optional)

prepare-commit-msg:  # Non-bypassable backstop
  commands:
    <name>:
      run: <command>
      stage_fixed: true

commit-msg:       # Validates commit message
  commands:
    <name>:
      run: <command>  # {1} = path to commit message file

pre-push:         # Runs before push to remote
  parallel: true
  commands:
    <name>:
      run: <command>

post-merge:       # Runs after merge/pull
  commands:
    <name>:
      run: <command>

post-checkout:    # Runs after branch checkout
  commands:
    <name>:
      run: <command>
```

---

## Entities

### Hook

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `pre-commit` | object | Yes | Primary checks before commit |
| `prepare-commit-msg` | object | Yes | Non-bypassable formatting backstop |
| `commit-msg` | object | Yes | Commit message validation |
| `pre-push` | object | Yes | Heavier checks before push |
| `post-merge` | object | Yes | Cache warming after merge (FR-010) |
| `post-checkout` | object | Yes | Cache warming after checkout (FR-010) |

### Command

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `run` | string | Yes | Shell command to execute |
| `glob` | string | No | File glob filter (only runs if matching files changed) |
| `stage_fixed` | boolean | No | Auto `git add` modified files (pre-commit only) |
| `parallel` | boolean | No | Run commands concurrently (hook-level) |
| `skip` | array | No | Conditions to skip (merge, rebase, branch ref) |
| `only` | array | No | Conditions to run (merge, rebase, branch ref) |

---

## Validation Rules

| Rule | Description | Enforcement |
|------|-------------|-------------|
| VR-001 | `pre-commit` MUST have at least one command | lefthook config validation |
| VR-002 | `stage_fixed: true` only valid for `pre-commit` | lefthook runtime |
| VR-003 | `parallel: true` cannot combine with `piped: true` | lefthook config validation |
| VR-004 | `commit-msg` commands receive message file path as `{1}` | lefthook runtime |
| VR-005 | Commands with `glob` only run if matching files are in changed set | lefthook runtime |

---

## State Transitions

### Commit Flow

```
Developer runs git commit
        │
        ▼
┌─────────────────┐
│   pre-commit    │ ← Bypassed by --no-verify
│ (fmt, clippy,   │
│  check)         │
└────────┬────────┘
         │ (if passed or bypassed)
         ▼
┌─────────────────┐
│prepare-commit-  │ ← NOT bypassed by --no-verify
│msg (fmt backstop)│
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   commit-msg    │ ← Bypassed by --no-verify
│ (cocogitto)     │
└────────┬────────┘
         │
         ▼
   Commit Created
         │
         ▼
┌─────────────────┐
│   post-commit   │ ← Notification only
└─────────────────┘
```

### Push Flow

```
Developer runs git push
        │
        ▼
┌─────────────────┐
│    pre-push     │
│ (nextest, deny, │
│  audit, doc,    │
│  llvm-cov)      │
└────────┬────────┘
         │ (if passed)
         ▼
   Push Completes
```

### Merge/Checkout Flow

```
git pull / git merge / git checkout
        │
        ▼
┌─────────────────┐
│  post-merge or  │
│  post-checkout  │
│  (cargo build)  │
└─────────────────┘
         │
         ▼
   Background build
   (non-blocking)
```

---

## Configuration Examples

### Minimal (pre-commit only)

```yaml
pre-commit:
  parallel: true
  commands:
    fmt:
      glob: "*.rs"
      run: cargo fmt -- --check
      stage_fixed: true
    clippy:
      glob: "*.rs"
      run: cargo clippy --workspace --all-targets -- -D warnings
```

### Full (all hooks)

See [plan.md](./plan.md) for the complete implementation approach.

---

## File Locations

| File | Path | Purpose |
|------|------|---------|
| Main config | `lefthook.yml` | Root configuration |
| Scripts (if needed) | `.lefthook/` | Custom scripts directory |
| Local overrides | `lefthook-local.yml` | Developer-specific overrides (gitignored) |
