# Contract: CI Workflow — Conditional Documentation Generation

**Feature**: CI/CD Pipeline (006-ci-cd-pipeline)
**Type**: GitHub Actions Workflow Contract
**Date**: 2026-09-03

---

## Contract: `ci.yml` — Docs Job

### Trigger
- Push to `main` or `dev` branches
- Pull requests to `main` or `dev` branches

### Inputs (from `detect-changes` job)
| Input | Type | Description |
|-------|------|-------------|
| `doc-scope` | `string` | `"workspace"`, `"crate"`, or `"skip"` |
| `crate` | `string` | Crate name when `doc-scope == "crate"` |

### Behavior

#### Case 1: `doc-scope == "skip"`
- Job is skipped entirely (no doc build)
- Condition: `if: needs.detect-changes.outputs.doc-scope != 'skip'`

#### Case 2: `doc-scope == "workspace"`
- Runs: `cargo doc --workspace --no-deps`
- Triggered when:
  - `Cargo.toml` or `Cargo.lock` changed, OR
  - 2 or more crate directories changed

#### Case 3: `doc-scope == "crate"`
- Runs: `cargo doc -p ${{ needs.detect-changes.outputs.crate }} --no-deps`
- Triggered when exactly 1 crate directory changed

### Environment Variables
| Variable | Value | Purpose |
|----------|-------|---------|
| `CARGO_TERM_COLOR` | `always` | Force colored output |
| `RUSTFLAGS` | `"-D warnings"` | Treat warnings as errors (enforces `missing_docs = deny`) |

### Outputs
| Output | Type | Description |
|--------|------|-------------|
| (none) | — | This job produces no outputs for downstream jobs |

### Failure Conditions
- `cargo doc` exits non-zero (missing docs, broken intra-doc links, doctest failures)
- Job exceeds 30-minute timeout

### Dependencies
- `needs: [detect-changes, check-and-test]`
- Docs only build if tests pass

---

## Contract: `detect-changes` Job — Doc Scope Computation

### Trigger
- Always runs (no condition)

### Inputs (from `dorny/paths-filter`)
| Input | Type | Description |
|-------|------|-------------|
| `leiden` | `string` | `"true"` if `crates/leiden/**` changed |
| `leiden-cli` | `string` | `"true"` if `crates/leiden-cli/**` changed |
| `leiden-tui` | `string` | `"true"` if `crates/leiden-tui/**` changed |
| `workspace-config` | `string` | `"true"` if `Cargo.toml` or `Cargo.lock` changed |

### Behavior

```bash
COUNT=0
CRATES=""
[[ "${{ steps.filter.outputs.leiden }}" == "true" ]] && { COUNT=$((COUNT+1)); CRATES="$CRATES leiden"; }
[[ "${{ steps.filter.outputs.leiden-cli }}" == "true" ]] && { COUNT=$((COUNT+1)); CRATES="$CRATES leiden-cli"; }
[[ "${{ steps.filter.outputs.leiden-tui }}" == "true" ]] && { COUNT=$((COUNT+1)); CRATES="$CRATES leiden-tui"; }

if [[ "${{ steps.filter.outputs.workspace-config }}" == "true" ]]; then
  echo "doc-scope=workspace" >> "$GITHUB_OUTPUT"
elif [[ "$COUNT" -ge 2 ]]; then
  echo "doc-scope=workspace" >> "$GITHUB_OUTPUT"
elif [[ "$COUNT" -eq 1 ]]; then
  echo "doc-scope=crate" >> "$GITHUB_OUTPUT"
  echo "crate=$(echo $CRATES | xargs)" >> "$GITHUB_OUTPUT"
else
  echo "doc-scope=skip" >> "$GITHUB_OUTPUT"
fi
```

### Outputs
| Output | Type | Description |
|--------|------|-------------|
| `doc-scope` | `string` | `"workspace"`, `"crate"`, or `"skip"` |
| `crate` | `string` | Crate name (only set when `doc-scope == "crate"`) |

---

## Contract: Proptest Regression Cache

### Trigger
- Runs in `check-and-test` job before test execution

### Cache Configuration
```yaml
- name: Cache proptest regressions
  uses: actions/cache@v4
  with:
    path: |
      target/proptest-regressions/
      crates/*/target/proptest-regressions/
    key: proptest-regressions-${{ runner.os }}-${{ hashFiles('crates/leiden/tests/*.rs') }}
    restore-keys: |
      proptest-regressions-${{ runner.os }}-
```

### Behavior
| Scenario | Cache State | Proptest Behavior |
|----------|-------------|-------------------|
| Primary key match | Full restore | Replay all seeds, then random exploration |
| Restore-keys match only | Partial restore | Replay older seeds, then random exploration |
| No match (cold cache) | No restore | Full random exploration; on failure, writes new seed file |

### Post-Step Behavior
- On job completion, `actions/cache` uploads `target/proptest-regressions/` to the cache
- Cache entry keyed by the primary key
- TTL: 7 days since last access (GitHub default)

---

## Contract: Concurrency and Cancellation

### Configuration
```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

### Behavior
- Pushing new commits to the same branch/PR cancels any in-progress workflow run
- Only one run per branch/PR executes at a time
- Prevents wasted runner minutes on stale commits

---

## Contract: Pipeline Timeout

### Configuration
```yaml
jobs:
  <job-name>:
    timeout-minutes: 30
```

### Behavior
- Any job exceeding 30 minutes is forcibly terminated by GitHub Actions
- Applies to all jobs in the workflow
- Prevents hung builds from consuming runner resources (SC-010)
