# Research: Lefthook Pre-commit Hooks

**Feature**: Lefthook Pre-commit Hooks (008)
**Date**: 2026-09-03
**Status**: Complete

---

## Research Questions & Decisions

### Q1: Can `prepare-commit-msg` modify the commit tree?

**Decision**: Yes, `prepare-commit-msg` CAN modify the tree. Git writes the tree object AFTER this hook runs.

**Rationale**: Analysis of git's source code (`builtin/commit.c`) shows:
- `prepare-commit-msg` hook runs at approximately line 1116-1118
- `write_index_as_tree()` (which creates the tree object from the index) runs at line 1706

Since line 1706 comes after line 1116, `git add` during `prepare-commit-msg` can modify the index, and those changes WILL be included in the current commit's tree.

**Alternatives Considered**:
- Using only `pre-commit` for formatting (rejected: bypassable via `--no-verify`)
- Using `commit-msg` for formatting (rejected: bypassable via `--no-verify`, cannot modify tree)

**Source**: Git source code (`builtin/commit.c`), [DeepWiki analysis](https://deepwiki.com/git/git/3.1-commit-creation)

---

### Q2: What is the correct hook execution order and bypass behavior?

**Decision**: Hook order is `pre-commit` → `prepare-commit-msg` → `commit-msg` → `post-commit`. `--no-verify` bypasses `pre-commit` and `commit-msg` but NOT `prepare-commit-msg`.

**Rationale**: Git documentation confirms:
- `pre-commit`: Bypassed by `--no-verify`, can modify tree, can abort commit
- `prepare-commit-msg`: NOT bypassed by `--no-verify`, can modify tree and message, can abort commit
- `commit-msg`: Bypassed by `--no-verify`, can modify message, can abort commit
- `post-commit`: NOT bypassed, cannot affect outcome (notification only)

**Alternatives Considered**:
- Relying solely on `pre-commit` (rejected: bypassable)
- Using `post-commit` for formatting (rejected: cannot affect the commit)

**Source**: [Git docs - githooks](https://git-scm.com/docs/githooks)

---

### Q3: What checks should pre-push include?

**Decision**: Run heavier checks too slow for pre-commit: `cargo nextest run`, `cargo deny check`, `cargo audit`, `cargo doc --workspace --no-deps`, `cargo llvm-cov`. All in parallel.

**Rationale**: Pre-push is the appropriate place for checks that would slow down the commit cycle too much but should pass before code is shared. These align with the constitution's CI pipeline.

**Alternatives Considered**:
- Mirror CI exactly including fmt/clippy/check (rejected: redundant with pre-commit)
- Run only tests (rejected: misses deny, audit, doc, coverage)

**Source**: [Git docs - pre-push](https://git-scm.com/docs/githooks#_pre_push), Rust CI best practices

---

### Q4: How to enforce Conventional Commits?

**Decision**: Use `cocogitto` in the `commit-msg` hook to validate commit message format.

**Rationale**: The constitution requires Conventional Commit format. `cocogitto` is a dedicated tool for this purpose and integrates with lefthook's `commit-msg` hook (receives the message file path as `{1}`).

**Alternatives Considered**:
- Custom regex in commit-msg (rejected: reinventing the wheel, less robust)
- `commitlint` (rejected: Node.js dependency, cocogitto is Rust-native)

**Source**: [cocogitto docs](https://github.com/cocogitto/cocogitto)

---

### Q5: Should post-merge/post-checkout warm the build cache?

**Decision**: Yes, run `cargo build` in background for both `post-merge` and `post-checkout`.

**Rationale**: After pulling new code or switching branches, the build cache is often stale. Background `cargo build` pre-populates the cache without blocking the developer.

**Alternatives Considered**:
- Skip cache warming (rejected: DX impact of slow builds after pull)
- Only warm on post-merge (rejected: branch switches also benefit)

**Source**: Git hooks best practices

---

### Q6: How to handle the `LEFTHOOK=0` bypass vector?

**Decision**: Document `LEFTHOOK=0` as a known bypass vector; rely on CI/CD as ultimate enforcement.

**Rationale**: Client-side hooks are inherently best-effort. A determined developer can always bypass them (`LEFTHOOK=0`, `--no-verify`, manual hook removal). CI/CD is the real gate that prevents non-compliant code from being merged.

**Alternatives Considered**:
- Attempt to block `LEFTHOOK=0` via wrapper scripts (rejected: fragile, cat-and-mouse game)
- Add detection in `prepare-commit-msg` (rejected: `LEFTHOOK=0` bypasses all lefthook hooks including this)

**Source**: [Lefthook docs - usage](https://lefthook.dev/usage/)

---

## Technology Choices

| Choice | Tool | Rationale |
|--------|------|-----------|
| Hook Manager | lefthook | Lightweight, fast, cross-platform, YAML config |
| Conventional Commits | cocogitto | Rust-native, integrates with lefthook |
| Test Runner | cargo-nextest | Faster than `cargo test`, structured output |
| License/Security | cargo-deny | Already in constitution's CI pipeline |
| Vulnerability Scanning | cargo-audit | Already in constitution's CI pipeline |
| Code Coverage | cargo-llvm-cov | LLVM-based, accurate coverage |

---

## Open Questions (Deferred to Implementation)

| Question | Impact | Deferred Because |
|----------|--------|----------------|
| Exact cocogitto configuration (which commit types allowed) | Medium | Requires team input on commit type policy |
| Whether to skip cache-warming on large workspaces | Low | Can be tuned post-implementation |
| How to handle `cargo llvm-cov` setup (requires `cargo-llvm-cov` install) | Medium | Future dev setup spec handles tooling installation |
