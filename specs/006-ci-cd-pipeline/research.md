# Phase 0 Research: CI/CD Pipeline for Leiden-Rust

**Feature**: CI/CD Pipeline (006-ci-cd-pipeline)
**Date**: 2026-09-03
**Status**: Complete — all NEEDS CLARIFICATION resolved

---

## 1. Conditional Documentation Generation Scope

**Question**: How should the CI determine whether to build workspace-wide or crate-targeted docs?

**Decision**: Use a shell-based counting step after `dorny/paths-filter` that emits a `doc-scope` output (`workspace`, `crate`, or `skip`).

**Rationale**:
- GitHub Actions expressions lack `sum()` or counting capabilities — shell arithmetic is the only reliable way to count changed crates
- `dorny/paths-filter` outputs are strings `'true'`/`'false'`, not booleans
- The spec defines clear thresholds: 2+ crates = workspace, 1 crate = targeted, 0 = skip
- Workspace config changes (`Cargo.toml`, `Cargo.lock`) must always trigger workspace-wide docs because dependency/lint changes can affect all crates

**Alternatives considered**:
- *Matrix strategy with conditional builds* — rejected: complexity outweighs parallelism benefit for only 3 crates
- *Separate jobs per crate with individual conditions* — rejected: duplicates the counting logic and creates 3x the job overhead
- *Always build workspace docs* — rejected: violates SC-005 (60% reduction target for isolated changes)

**Concrete Pattern**:
```bash
COUNT=0
[[ "${{ steps.filter.outputs.leiden }}" == "true" ]] && COUNT=$((COUNT+1))
[[ "${{ steps.filter.outputs.leiden-cli }}" == "true" ]] && COUNT=$((COUNT+1))
[[ "${{ steps.filter.outputs.leiden-tui }}" == "true" ]] && COUNT=$((COUNT+1))

if [[ "${{ steps.filter.outputs.workspace-config }}" == "true" ]]; then
  echo "doc-scope=workspace" >> "$GITHUB_OUTPUT"
elif [[ "$COUNT" -ge 2 ]]; then
  echo "doc-scope=workspace" >> "$GITHUB_OUTPUT"
elif [[ "$COUNT" -eq 1 ]]; then
  echo "doc-scope=crate" >> "$GITHUB_OUTPUT"
  echo "crate=<name>" >> "$GITHUB_OUTPUT"
else
  echo "doc-scope=skip" >> "$GITHUB_OUTPUT"
fi
```

---

## 2. Proptest Regression Caching Strategy

**Question**: How should proptest regression files be cached to ensure deterministic re-testing?

**Decision**: Use `actions/cache@v4` with a hash-based key on test source files, with OS-level restore-keys fallback.

**Rationale**:
- Proptest writes seeds to `target/proptest-regressions/<crate>/<filename>.txt` only when failures occur
- These files are small (bytes each) — cache overhead is negligible
- `hashFiles('crates/leiden/tests/*.rs')` as part of the key ensures stale seeds are invalidated when test logic changes
- Cold cache is acceptable per spec clarification: "Fail immediately and write regression file"
- Committing regression files to the repo is rejected — pollutes git history and seeds can become stale

**Alternatives considered**:
- *Commit regression files to repo* — rejected: pollutes history, merge conflicts, stale seeds
- *Use Swatinem/rust-cache for regressions* — rejected: rust-cache purges stale artifacts and doesn't support stable keys for regression replay
- *Always replay all historical seeds* — rejected: seeds become invalid when test logic changes; causes false failures

**Cache Key Strategy**:
```yaml
key: proptest-regressions-${{ runner.os }}-${{ hashFiles('crates/leiden/tests/*.rs') }}
restore-keys: |
  proptest-regressions-${{ runner.os }}-
```

**Cold cache behavior**: On first run (or after test file changes), no cache hit → proptest runs fresh → on failure, writes regression file → `actions/cache` post-step uploads for next run. This matches the spec requirement exactly.

---

## 3. Headless Ratatui Testing Patterns

**Question**: How should TUI tests run reliably in CI without terminal hardware?

**Decision**: Three-tier approach based on test type:

| Test Type | Technique | CI-Safe | PTY Needed |
|-----------|-----------|---------|------------|
| Unit tests (state, logic) | Pure Rust `#[cfg(test)]` | Yes | No |
| Rendering tests (widgets, layout) | `TestBackend::new(w, h)` | Yes | No |
| Integration tests (full binary) | `portable-pty` or `script` | Yes (Unix) | Yes |

**Rationale**:
- `TestBackend` is Ratatui's official mock backend — zero syscalls, in-memory buffer
- The existing codebase already has excellent TestBackend coverage (15+ test files)
- PTY allocation is only needed for testing the actual `main()` binary entry point
- `portable-pty` works cross-platform (Linux/macOS/Windows); `script` is Unix-only fallback
- The codebase already enforces the Anti-Pattern 2 remedy: `ui::render()` takes `&App`, `main.rs` is the only place that binds `CrosstermBackend`

**Alternatives considered**:
- *Mock crossterm directly* — rejected: Crossterm has no mock backend; Ratatui's `Backend` trait is the correct abstraction
- *Use `ratatui::init()` in tests* — rejected: requires PTY, defeats headless purpose
- *Run integration tests without PTY* — rejected: `enable_raw_mode()` returns `ENOTTY` error without a PTY

**Geometry edge case testing**: Already handled by existing tests. `TestBackend::new(79, 23)` renders at below-minimum dimensions to assert the `TerminalDimensionGuard` overlay appears without panicking.

---

## 4. Dependency: cargo doc and Binary Name Collision

**Question**: Does `cargo doc --workspace` fail due to binary name collision between `leiden` (library) and `leiden-cli` (binary named `leiden`)?

**Decision**: Yes — this is a known Cargo issue (#6313). The conditional doc generation pattern (building per-crate when only 1 crate changes) avoids this collision.

**Rationale**:
- When only `leiden-cli` changes, `cargo doc -p leiden-cli --no-deps` builds only that crate's docs — no collision
- When workspace-wide docs are needed, the collision may surface. Two mitigations:
  1. Use `cargo doc --workspace --no-deps` (current workaround, works in most cases)
  2. Build per-crate docs sequentially and merge output (future improvement)

**Alternatives considered**:
- *Rename the `leiden-cli` binary* — rejected: breaking change to user-facing CLI
- *Use separate doc builds per crate always* — rejected: defeats the workspace-wide requirement for multi-crate changes

---

## 5. Integration with Existing CI Workflow

**Question**: How does the new `docs` job integrate with the existing `check-and-test` job in `.github/workflows/ci.yml`?

**Decision**: Add a separate `docs` job that depends on both `detect-changes` and `check-and-test`, with conditional execution based on `doc-scope` output.

**Rationale**:
- The existing `check-and-test` job already runs `cargo nextest run --workspace` and `cargo doc --workspace --no-deps`
- Moving doc generation to a dedicated job enables:
  - Conditional scope (workspace vs. crate)
  - Independent timeout and retry
  - Clearer failure attribution in GitHub Actions UI
- Job dependency `needs: [detect-changes, check-and-test]` ensures docs only build if tests pass

**Alternatives considered**:
- *Add doc steps to existing `check-and-test` job* — rejected: mixes concerns, harder to conditionally skip
- *Create separate workflow file* — rejected: adds complexity, reduces visibility into pipeline status

---

## 6. Cross-Platform Release: Cross-Compilation Approach

**Question**: How should the release pipeline handle cross-compilation for musl targets?

**Decision**: Use the `cross` tool for musl targets (`x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`), native compilation for macOS and Windows.

**Rationale**:
- `cross` uses Docker containers with the correct target toolchains pre-installed
- Avoids the complexity of installing musl toolchains on the runner
- Native compilation for macOS/Windows is faster and more reliable than cross-compilation
- The release matrix uses `fail-fast: false` to attempt all targets even if one fails

**Alternatives considered**:
- *Use `cargo-zigbuild` for musl targets* — rejected: adds Zig dependency, `cross` is more established
- *Install musl toolchain directly on runner* — rejected: slower, more fragile, requires sudo/root
- *Use GitHub Actions musl action* — rejected: `cross` handles this more robustly

---

## Summary of Decisions

| Area | Decision | Key Trade-off |
|------|----------|---------------|
| Doc scope | Shell counting step with `doc-scope` output | Shell required (GH Actions expressions can't count) |
| Proptest cache | `actions/cache` with hash-based key | Cold cache = first run replays no seeds |
| TUI testing | TestBackend (primary) + portable-pty (integration) | PTY tests are Unix-only |
| Doc collision | Per-crate builds avoid collision | Workspace-wide may still hit Cargo#6313 |
| CI integration | Separate `docs` job with conditional | Adds one job, improves clarity |
| Cross-compilation | `cross` tool for musl, native for others | Docker dependency for Linux targets |
