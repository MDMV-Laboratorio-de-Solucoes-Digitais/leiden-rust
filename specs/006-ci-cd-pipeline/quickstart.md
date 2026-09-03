# Quickstart Validation: CI/CD Pipeline — Conditional Documentation

**Feature**: CI/CD Pipeline (006-ci-cd-pipeline)
**Date**: 2026-09-03
**Purpose**: Runnable validation scenarios that prove the conditional doc generation feature works end-to-end

---

## Prerequisites

- Repository cloned locally
- Git configured with push access to a fork or feature branch
- GitHub Actions enabled on the repository
- The CI workflow file `.github/workflows/ci.yml` exists with the `detect-changes` and `docs` jobs

---

## Scenario 1: Single-Crate Change Triggers Targeted Docs

**Goal**: Verify that modifying one crate triggers only that crate's documentation build.

### Steps
1. Create a test branch:
   ```bash
   git checkout -b test/single-crate-docs
   ```

2. Make a documentation change to a single crate (e.g., add a doc comment to `crates/leiden-cli/src/main.rs`):
   ```rust
   /// Test doc comment for CI validation
   fn documented_function() {}
   ```

3. Commit and push:
   ```bash
   git add .
   git commit -m "test: validate single-crate doc generation"
   git push origin test/single-crate-docs
   ```

4. Open a pull request (or observe push-triggered CI)

### Expected Outcome
- `detect-changes` job runs and outputs `doc-scope=crate`, `crate=leiden-cli`
- `docs` job runs `cargo doc -p leiden-cli --no-deps`
- Docs for `leiden` and `leiden-tui` are NOT built
- Job succeeds if documentation is valid; fails if `missing_docs = deny` is violated

---

## Scenario 2: Multi-Crate Change Triggers Workspace Docs

**Goal**: Verify that modifying two or more crates triggers workspace-wide documentation.

### Steps
1. Create a test branch:
   ```bash
   git checkout -b test/multi-crate-docs
   ```

2. Make a doc change to two crates:
   - `crates/leiden/src/lib.rs` — add a doc comment
   - `crates/leiden-tui/src/app.rs` — add a doc comment

3. Commit and push:
   ```bash
   git add .
   git commit -m "test: validate workspace doc generation"
   git push origin test/multi-crate-docs
   ```

4. Open a pull request (or observe push-triggered CI)

### Expected Outcome
- `detect-changes` job outputs `doc-scope=workspace`
- `docs` job runs `cargo doc --workspace --no-deps`
- All three crates' documentation is built
- Job succeeds if all documentation is valid

---

## Scenario 3: Workspace Config Change Triggers Workspace Docs

**Goal**: Verify that touching `Cargo.toml` always triggers workspace-wide docs.

### Steps
1. Create a test branch:
   ```bash
   git checkout -b test/config-docs
   ```

2. Add a harmless comment to `Cargo.toml` (or touch the file without changes):
   ```bash
   echo "# CI validation" >> Cargo.toml
   ```

3. Commit and push:
   ```bash
   git add Cargo.toml
   git commit -m "test: validate config-triggered docs"
   git push origin test/config-docs
   ```

4. Observe CI

### Expected Outcome
- `detect-changes` job outputs `doc-scope=workspace` (due to `workspace-config` filter)
- `docs` job runs `cargo doc --workspace --no-deps`
- Workspace-wide docs are built even though no source files changed

---

## Scenario 4: Non-Code Change Skips Docs

**Goal**: Verify that documentation-only or fixture-only changes skip the docs job.

### Steps
1. Create a test branch:
   ```bash
   git checkout -b test/non-code-skip
   ```

2. Modify a non-code file:
   ```bash
   echo "# Test" >> README.md
   ```

3. Commit and push:
   ```bash
   git add README.md
   git commit -m "test: validate docs skip for non-code changes"
   git push origin test/non-code-skip
   ```

4. Observe CI

### Expected Outcome
- `detect-changes` job outputs `doc-scope=skip`
- `docs` job is SKIPPED (not failed — the job card shows "Skipped")
- `check-and-test` job still runs (fmt, clippy, tests execute normally)

---

## Scenario 5: Missing Documentation Fails the Build

**Goal**: Verify that `missing_docs = deny` is enforced in CI doc generation.

### Steps
1. Create a test branch:
   ```bash
   git checkout -b test/missing-docs-fail
   ```

2. Add a public item WITHOUT a doc comment to `crates/leiden-cli/src/main.rs`:
   ```rust
   pub struct UndocumentedStruct {
       pub field: String,
   }
   ```

3. Commit and push:
   ```bash
   git add .
   git commit -m "test: validate missing_docs failure"
   git push origin test/missing-docs-fail
   ```

4. Observe CI

### Expected Outcome
- `docs` job runs `cargo doc -p leiden-cli --no-deps`
- Build FAILS with error: `error: missing documentation for a struct`
- CI status check shows red/failed

### Cleanup
- Revert the change after observing the failure:
   ```bash
   git revert HEAD
   git push origin test/missing-docs-fail
  ```

---

## Scenario 6: Proptest Regression Cache Determinism

**Goal**: Verify that proptest regression seeds are cached and replayed.

### Steps
1. Observe CI logs for the cache step on first run (cold cache):
   - Expect: "Cache not found for input keys: ..."

2. If a proptest fails and writes a regression file, the post-step uploads it

3. On the next run, observe:
   - Expect: "Cache restored from key: proptest-regressions-..."
   - Proptest replays the cached seed first, then continues with random exploration

### Expected Outcome
- First run: no cache, proptest runs full exploration
- Subsequent runs: cache hit, seeds replayed deterministically
- If a cached seed no longer fails, proptest removes it from the regression file

---

## Validation Checklist

| Scenario | Expected Result | Pass Criteria |
|----------|-----------------|---------------|
| 1: Single-crate change | `cargo doc -p leiden-cli --no-deps` runs | Docs job shows single-crate command in logs |
| 2: Multi-crate change | `cargo doc --workspace --no-deps` runs | Docs job shows workspace command in logs |
| 3: Config change | Workspace docs build | `doc-scope=workspace` in detect-changes output |
| 4: Non-code change | Docs job skipped | Job card shows "Skipped" |
| 5: Missing docs | Build fails | CI status red, error mentions `missing_docs` |
| 6: Proptest cache | Seeds replayed | Cache hit message in logs, deterministic behavior |

---

## Troubleshooting

| Issue | Likely Cause | Fix |
|-------|--------------|-----|
| `docs` job always skips | `detect-changes` not outputting `doc-scope` correctly | Check that `dorny/paths-filter` outputs are wired to job outputs |
| Workspace docs fail with name collision | Cargo#6313 bug | Build per-crate docs sequentially as workaround |
| Proptest regressions not cached | Cache key mismatch | Verify `hashFiles('crates/leiden/tests/*.rs')` path matches actual test files |
| `doc-scope` is empty | Shell step didn't write to `GITHUB_OUTPUT` | Check step has `id: scope` and uses `echo "key=value" >> "$GITHUB_OUTPUT"` |
