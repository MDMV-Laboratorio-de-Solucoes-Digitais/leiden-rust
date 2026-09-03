# Quickstart Validation: Lefthook Pre-commit Hooks

**Feature**: Lefthook Pre-commit Hooks (008)
**Date**: 2026-09-03

---

## Purpose

This document provides runnable validation scenarios to prove the lefthook configuration works end-to-end. Execute these scenarios after implementation to verify all functional requirements are met.

---

## Prerequisites

- [ ] lefthook installed (`lefthook version` works)
- [ ] Rust toolchain installed (`cargo --version` works)
- [ ] cocogitto installed (`cocogitto --version` works)
- [ ] cargo-nextest installed (`cargo nextest --version` works)
- [ ] cargo-deny installed (`cargo deny --version` works)
- [ ] cargo-audit installed (`cargo audit --version` works)
- [ ] cargo-llvm-cov installed (`cargo llvm-cov --version` works)
- [ ] Repository cloned and on branch `008-lefthook-precommit-hooks`
- [ ] lefthook installed in repo (`lefthook install` run)

---

## Validation Scenarios

### Scenario 1: Pre-commit Formatting (FR-001)

**Goal**: Verify `cargo fmt` auto-applies formatting on commit.

**Steps**:
1. Create a test file with improper formatting:
   ```bash
   echo 'fn main() { println!("hello"); }' > /tmp/test_fmt.rs
   cp /tmp/test_fmt.rs crates/<crate>/src/test_fmt.rs
   ```
2. Stage the file: `git add .`
3. Commit: `git commit -m "test: add unformatted file"`
4. Observe: Formatting is auto-applied, commit succeeds

**Expected Outcome**:
- Commit succeeds
- File is reformatted in the commit
- Output shows "all checks passed" summary

**Pass Criteria**: `git log -1 --format="%B"` shows the commit message, and `git show HEAD:<path>` shows formatted code.

---

### Scenario 2: Pre-commit Blocks Clippy Warnings (FR-002)

**Goal**: Verify clippy blocks commits with warnings.

**Steps**:
1. Introduce a clippy warning (e.g., `let x = 1; let y = x;` when `y` could use `x` directly)
2. Stage: `git add .`
3. Commit: `git commit -m "test: introduce clippy warning"`

**Expected Outcome**:
- Commit is blocked
- Output shows specific clippy warning

**Pass Criteria**: Commit does not appear in `git log`.

---

### Scenario 3: Pre-commit Blocks Type Errors (FR-002)

**Goal**: Verify `cargo check` blocks commits with type errors.

**Steps**:
1. Introduce a type error (e.g., `let x: i32 = "not a number";`)
2. Stage: `git add .`
3. Commit: `git commit -m "test: introduce type error"`

**Expected Outcome**:
- Commit is blocked
- Output shows type error

**Pass Criteria**: Commit does not appear in `git log`.

---

### Scenario 4: `--no-verify` Bypass Backstop (FR-003)

**Goal**: Verify `prepare-commit-msg` still formats when `--no-verify` is used.

**Steps**:
1. Create an unformatted file
2. Stage: `git add .`
3. Commit with bypass: `git commit --no-verify -m "test: bypass formatting"`

**Expected Outcome**:
- `prepare-commit-msg` hook runs `cargo fmt --all`
- Formatted files are re-staged
- Commit may or may not include formatted code (depends on git internals)

**Pass Criteria**: `lefthook` output shows `prepare-commit-msg` ran.

---

### Scenario 5: Non-Rust Files Skip Checks (FR-005)

**Goal**: Verify checks skip when no `.rs` files are changed.

**Steps**:
1. Edit a markdown file: `echo "# Test" >> README.md`
2. Stage: `git add README.md`
3. Commit: `git commit -m "docs: update README"`

**Expected Outcome**:
- Commit succeeds quickly (< 1 second)
- No Rust checks run

**Pass Criteria**: Commit succeeds, no clippy/check output.

---

### Scenario 6: Pre-push Runs Heavier Checks (FR-008)

**Goal**: Verify pre-push runs nextest, deny, audit, doc, llvm-cov.

**Steps**:
1. Make a valid commit
2. Push to a test branch: `git push origin HEAD:test-pre-push`

**Expected Outcome**:
- Pre-push hooks run
- All checks pass (or fail with clear output)

**Pass Criteria**: Push completes or fails with specific check output.

---

### Scenario 7: Commit-msg Enforces Conventional Commits (FR-009)

**Goal**: Verify cocogitto blocks non-conventional commit messages.

**Steps**:
1. Make a valid change
2. Stage: `git add .`
3. Commit with bad message: `git commit -m "this is not conventional"`

**Expected Outcome**:
- Commit is blocked
- Output shows conventional commit format requirement

**Pass Criteria**: Commit does not appear in `git log`.

---

### Scenario 8: Valid Conventional Commit Passes (FR-009)

**Goal**: Verify valid conventional commits are accepted.

**Steps**:
1. Make a valid change
2. Stage: `git add .`
3. Commit with valid message: `git commit -m "feat: add new feature"`

**Expected Outcome**:
- Commit succeeds

**Pass Criteria**: `git log -1` shows the commit.

---

### Scenario 9: Post-merge Cache Warming (FR-010)

**Goal**: Verify post-merge runs `cargo build`.

**Steps**:
1. Create a branch, make a commit, switch back
2. Merge the branch: `git merge test-branch`

**Expected Outcome**:
- Post-merge hook triggers
- `cargo build` runs in background

**Pass Criteria**: lefthook output shows post-merge command ran.

---

### Scenario 10: Post-checkout Cache Warming (FR-010)

**Goal**: Verify post-checkout runs `cargo build`.

**Steps**:
1. Switch branches: `git checkout main`

**Expected Outcome**:
- Post-checkout hook triggers
- `cargo build` runs in background

**Pass Criteria**: lefthook output shows post-checkout command ran.

---

### Scenario 11: Performance Target (SC-002)

**Goal**: Verify pre-commit completes in <5 seconds for typical commits.

**Steps**:
1. Make a small change (1 file)
2. Time the commit: `time git commit -m "test: performance check"`

**Expected Outcome**:
- Commit completes in <5 seconds

**Pass Criteria**: `time` output shows <5s for the hook portion.

---

### Scenario 12: Parallel Execution (FR-002)

**Goal**: Verify pre-commit checks run in parallel.

**Steps**:
1. Run with lefthook verbose: `LEFTHOOK_VERBOSE=1 git commit -m "test: parallel check"`

**Expected Outcome**:
- Output shows commands starting concurrently

**Pass Criteria**: Log output shows parallel execution.

---

## Regression Tests

After all scenarios pass, verify these edge cases:

| Edge Case | Command | Expected |
|-----------|---------|----------|
| Empty commit | `git commit --allow-empty -m "chore: empty"` | Succeeds |
| Multiple files | Change 5 `.rs` files, commit | All formatted/checked |
| Merge commit | `git merge --no-ff branch` | Hooks run appropriately |
| Amend commit | `git commit --amend` | Hooks run on amend |

---

## Cleanup

After validation:
1. Remove test commits: `git reset --hard origin/dev`
2. Delete test branches: `git branch -D test-branch`
3. Remove test files if any remain

---

## Sign-off

| Scenario | Pass | Notes |
|----------|------|-------|
| 1. Pre-commit Formatting | ☐ | |
| 2. Clippy Blocks | ☐ | |
| 3. Type Check Blocks | ☐ | |
| 4. `--no-verify` Backstop | ☐ | |
| 5. Non-Rust Skip | ☐ | |
| 6. Pre-push Checks | ☐ | |
| 7. Conventional Commit Block | ☐ | |
| 8. Conventional Commit Pass | ☐ | |
| 9. Post-merge Cache | ☐ | |
| 10. Post-checkout Cache | ☐ | |
| 11. Performance Target | ☐ | |
| 12. Parallel Execution | ☐ | |
