# Feature Specification: Lefthook Pre-commit Hooks

**Feature Branch**: `008-lefthook-precommit-hooks`

**Created**: 2026-09-03

**Status**: Draft

**Input**: User description: "Configure lefthook to run cargo fmt on commit, other quick lints, checks and verifications that does not compromise DX, deny using '--no-verify' on commit."

## Clarifications

### Session 2026-09-03

- Q: How should the system detect and block `--no-verify` bypass attempts, given that lefthook runs as a Git hook and `--no-verify` skips Git hooks entirely? → A: Use `prepare-commit-msg` hook (which is NOT bypassed by `--no-verify`) as a non-bypassable backstop to run `cargo fmt --all` with `stage_fixed: true`. Combine with policy documentation, CI/CD enforcement, and lefthook configuration.
- Q: Beyond `cargo fmt --check` and `cargo clippy`, which specific checks should the pre-commit hooks include to maintain DX while catching common issues early? → A: Use `cargo fmt` (without `--check`) to auto-apply formatting and re-stage modified files. Ship with focused checks (fmt + clippy + cargo check) as default, allow opt-in to additional checks via lefthook config.
- Q: What should happen when a developer attempts to commit but lefthook is not installed or configured in their local environment? → A: Deferred to a future dev setup spec that will handle installation of required/suggested dev tooling including lefthook.
- Q: What defines a "typical commit" for the performance target of under 5 seconds (SC-002 and FR-004)? → A: Measure against up to 10 files changed / 500 lines as the upper bound benchmark, but the expected typical case is 1 file per commit (project workflow convention).
- Q: When a commit contains no Rust files (e.g., only documentation or config changes), should the pre-commit hooks skip entirely or still run on the workspace? → A: Skip entirely: if no `.rs` files in the commit's changed file list (via `git diff --name-only`), skip all Rust checks for fast non-Rust commits.
- Q: When `cargo fmt` fails to format a file (e.g., due to a syntax error), should the pre-commit hook block the commit entirely or allow it to proceed with a warning? → A: Allow the commit to proceed with a warning if `cargo fmt` fails. Syntax errors will be caught by `cargo check` and `cargo clippy` anyway, so blocking on fmt failure would be redundant. The warning provides feedback without blocking the developer.
- Q: Should the pre-commit checks (`cargo fmt`, `cargo clippy`, `cargo check`) run in parallel or sequentially? → A: Run checks in parallel using lefthook's native parallel execution support. This ensures the 5-second performance target is met. Output interleaving is acceptable since each tool's output is prefixed.
- Q: Should there be any documented, emergency-only mechanism to bypass pre-commit hooks (e.g., when broken code on main blocks all commits)? → A: No built-in bypass mechanism. Maintainers use the existing workflow: clone the repo locally, apply the fix, push directly (maintainers have push access), and let CI validate. This keeps the system simple while acknowledging maintainers have alternative paths for emergencies.
- Q: When pre-commit checks all pass successfully, should the hooks display any output or remain silent? → A: Always show a brief "all checks passed" summary by default. This is configurable via lefthook settings for developers who prefer silent success.
- Q: The spec uses `prepare-commit-msg` as a non-bypassable formatting backstop. Research confirms this technically works (git writes the tree AFTER this hook runs), but git docs explicitly state "it should not be used as a replacement for the pre-commit hook." How should we approach this? → A: Keep `prepare-commit-msg` as a secondary safety net (last-chance formatting for `--no-verify` scenarios), but clarify that `pre-commit` is the primary hook for all formatting and checks.
- Q: What should the `pre-push` hook include for this Rust project? → A: Mirror CI exactly: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo doc --workspace --no-deps`, and `cargo deny check`. Run in parallel using lefthook's `parallel: true`.
- Q: Given the proposed CI/CD chain (cocogitto, audit, vet, nextest, llvm-cov, semver-checks, release-plz, mutants), which checks should pre-commit vs pre-push include? → A: Pre-commit stays fast (fmt + clippy + check, ~5s). Pre-push runs heavier checks (nextest, deny, audit, doc, llvm-cov). cocogitto enforces Conventional Commits in commit-msg hook.
- Q: Should the spec include `post-merge` and/or `post-checkout` hooks for build cache warming? → A: Add both `post-merge` and `post-checkout` hooks that run `cargo build` in the background to pre-populate the build cache after pull/merge and branch switches.
- Q: Research revealed that lefthook can be bypassed entirely via `LEFTHOOK=0 git commit`. Should the spec address this bypass vector? → A: Document `LEFTHOOK=0` as a known bypass vector; rely on CI/CD as ultimate enforcement. Client-side hooks are best-effort, CI is the real gate.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Automatic Code Formatting on Commit (Priority: P1)

As a developer, I want code formatting to be automatically checked when I commit changes so that I don't have to manually run formatting commands and CI doesn't fail on formatting violations.

**Why this priority**: Formatting violations are the most common CI failure. Catching them locally before commit saves CI resources and developer time.

**Independent Test**: Can be fully tested by making a commit with improperly formatted code and observing that formatting is automatically applied and the commit succeeds with formatted code.

**Acceptance Scenarios**:

1. **Given** a developer has unformatted code changes, **When** they attempt to commit, **Then** formatting is automatically applied and the modified files are re-staged
2. **Given** a developer has properly formatted code changes, **When** they attempt to commit, **Then** the commit succeeds without formatting-related delays
3. **Given** a developer attempts to bypass pre-commit hooks with `--no-verify`, **When** they commit, **Then** the `prepare-commit-msg` backstop still applies formatting via `stage_fixed: true`

---

### User Story 2 - Fast Local Quality Gates (Priority: P2)

As a developer, I want quick lint and verification checks to run on commit so that common issues are caught early without slowing down my workflow.

**Why this priority**: Fast feedback on common mistakes (unused imports, trivial type issues) prevents wasted CI runs and keeps the codebase clean.

**Independent Test**: Can be tested by introducing a minor lint violation and observing the commit is blocked with a clear error message.

**Acceptance Scenarios**:

1. **Given** a developer has code with a clippy warning, **When** they attempt to commit, **Then** the commit is blocked with the specific lint violation
2. **Given** a developer has code that passes all quick checks, **When** they attempt to commit, **Then** the commit completes within seconds

---

### User Story 3 - Bypass Prevention (Priority: P2)

As a project maintainer, I want to prevent developers from bypassing pre-commit hooks with `--no-verify` so that all code committed to the repository passes quality gates.

**Why this priority**: Bypassing hooks defeats the purpose of local quality checks and allows unformatted or non-compliant code to reach CI.

**Independent Test**: Can be tested by attempting to commit with `--no-verify` flag and observing the bypass is blocked.

**Acceptance Scenarios**:

1. **Given** a developer attempts to commit with `--no-verify`, **When** the commit is processed, **Then** the `prepare-commit-msg` backstop still applies formatting (bypass is mitigated, not fully blocked — `LEFTHOOK=0` is a known residual vector; CI/CD is the ultimate enforcement gate)
2. **Given** a developer commits normally (without `--no-verify`), **When** the commit is processed, **Then** pre-commit hooks run as expected

---

### Edge Cases

- What happens when the developer has a slow machine? → Pre-commit hooks MUST complete within 5 seconds (benchmark: up to 10 files / 500 lines changed; expected typical: 1 file per commit)
- What happens when there are no Rust files changed? → Pre-commit hooks MUST skip entirely when no `.rs` files are in the commit's changed file list (via `git diff --name-only`), enabling fast commits for documentation/config changes
- What happens when the developer is on a non-Rust project file? → Pre-commit hooks MUST only run when `.rs` files are detected in the changed file list
- What happens when `cargo fmt` fails on a file (e.g., syntax error)? → Emit a warning to the developer but allow the commit to proceed; syntax errors will be caught by `cargo check` and `cargo clippy` which block the commit

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST run `cargo fmt` (without `--check`) on pre-commit to auto-apply formatting and re-stage modified files via `stage_fixed: true`. If `cargo fmt` fails on a file (e.g., syntax error), emit a warning but do not block the commit (syntax errors are caught by `cargo check` and `cargo clippy`).
- **FR-002**: System MUST run `cargo clippy --workspace --all-targets -- -D warnings` and `cargo check --workspace` on pre-commit to catch lint and type errors, blocking commits that fail with clear error messages. All checks MUST run in parallel to meet the 5-second performance target
- **FR-003**: System MUST use `pre-commit` as the **primary** hook for all formatting and checks (see FR-001 for fmt behavior details). System MUST additionally use `prepare-commit-msg` hook as a **secondary, non-bypassable safety net** to run `cargo fmt --all` for `--no-verify` scenarios. While git documentation notes that `prepare-commit-msg` "should not be used as a replacement for the pre-commit hook," this spec uses it as a complementary last-chance formatting backstop — not a replacement. **Implementation note**: `stage_fixed: true` is only supported by lefthook in `pre-commit`, so `prepare-commit-msg` MUST use `cargo fmt --all && git add -u` to explicitly re-stage formatted files. Client-side hooks are best-effort; `LEFTHOOK=0` is a known bypass vector that skips all lefthook hooks — CI/CD is the ultimate enforcement gate for this vector. No built-in bypass mechanism exists; maintainers use direct clone + push + CI validation for emergency scenarios.
- **FR-004**: System MUST complete pre-commit checks within 5 seconds. Benchmark: up to 10 files / 500 lines changed. Expected typical case: 1 file per commit (per project workflow convention). See SC-002 for the measurable outcome. **Validation**: Performance is validated manually via quickstart.md Scenario 11 (timed commit).
- **FR-005**: System MUST only run Rust-specific checks when `.rs` files are present in the commit's changed file list. Implementation: use lefthook's `glob: "*.rs"` pattern to filter commands; when no `.rs` files match, lefthook skips the command automatically, enabling fast commits for documentation/config changes
- **FR-006**: System MUST surface each tool's native output (stdout/stderr) as the error message when checks fail, ensuring developers see the exact lint/type violation with file and line information. On success, display a brief "all checks passed" summary (configurable via lefthook settings). **Note**: This is lefthook's default behavior — failed commands print their output and block; successful commands can be summarized via lefthook's `skip_output` or similar settings. No custom error formatting required.
- **FR-007**: System MUST ship with focused checks (fmt + clippy + cargo check) as default, and allow opt-in to additional checks via lefthook configuration. **Opt-in mechanism**: Additional commands (e.g., `cargo audit`, `cargo deny check`) are included in `lefthook.yml` as commented-out commands under each hook. Developers uncomment to enable. This keeps the default fast while making extended checks discoverable.
- **FR-008**: System MUST run a `pre-push` hook with heavier checks that are too slow for pre-commit: `cargo nextest run` (full test suite), `cargo deny check` (license/security), `cargo audit` (vulnerability scanning), `cargo doc --workspace --no-deps` (docs compile check), and `cargo llvm-cov` (code coverage). All pre-push checks MUST run in parallel using lefthook's `parallel: true` to minimize push latency. Pre-push hook MUST block the push if any check fails, with clear output identifying which check failed and why.
- **FR-009**: System MUST run a `commit-msg` hook that uses `cocogitto` to enforce Conventional Commit format, ensuring all commit messages follow the project's commit message convention as required by the constitution.
- **FR-010**: System MUST run `post-merge` and `post-checkout` hooks that execute `cargo build` (workspace root, debug mode) in the background to pre-populate the build cache after pull/merge operations and branch switches, improving subsequent build performance without blocking the developer. **Note**: Concurrent builds from rapid merge/checkout operations may overlap; this is acceptable as `cargo build` is idempotent and the worst case is redundant work, not corruption.

### Key Entities

- **Pre-commit Hook**: The automation that runs before a commit is finalized
- **Lefthook Configuration**: The configuration file that defines which checks run and when
- **Bypass Attempt**: Any attempt to skip pre-commit hooks using `--no-verify` or similar flags

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Zero formatting violations reach CI. Measured by CI `cargo fmt --check` pipeline passing on every PR. Client-side fmt (FR-001) is the first gate; CI is the ultimate backstop for the `LEFTHOOK=0` bypass vector
- **SC-002**: Pre-commit hooks complete within 5 seconds for typical commits. Measures FR-004 performance target.
- **SC-003**: When lefthook is active, all `--no-verify` commits still receive formatting via the non-bypassable `prepare-commit-msg` backstop. Residual vectors: `LEFTHOOK=0` bypasses all lefthook hooks (CI/CD is the ultimate enforcement gate for this vector)
- **SC-004**: Developer workflow is not hindered: (1) hooks auto-activate via `lefthook install` (no per-command manual setup), (2) non-Rust commits skip checks in <1s (FR-005), (3) post-merge/post-checkout cache warming runs in background (FR-010). Validated via quickstart.md Scenarios 5, 9, 10

## Assumptions

- Developers have lefthook installed and configured in their local environment
- The project uses Rust stable edition 2024 with workspace lints configured
- CI pipeline already runs the same checks (pre-commit hooks complement, not replace, CI)
- Developers have access to `cargo fmt` and `cargo clippy` tools
- The project follows Conventional Commit format for commit messages (enforced by this feature via `cocogitto` in commit-msg hook — see FR-009; this spec introduces/amplifies the convention per Development Workflow requirements)

### Test-First Validation (Constitution Principle V)

This feature is infrastructure configuration (no library or binary crates). Per Constitution Principle V (Test-First, NON-NEGOTIABLE), validation is performed via the manual quickstart scenarios in [quickstart.md](./quickstart.md) rather than automated unit/integration tests. Each scenario is executed via git operations before merge, serving as the test-first validation suite for this infrastructure config feature.

## Technical Tooling

- **Hook Manager**: lefthook (lightweight, fast git hook manager)
- **Formatting**: `cargo fmt` (auto-applies formatting, no `--check`) with `stage_fixed: true` to re-stage modified files
- **Linting**: `cargo clippy --workspace --all-targets -- -D warnings` (Rust linter)
- **Type Checking**: `cargo check --workspace` (fast compile check without producing binaries)
- **Bypass Prevention**: See FR-003 for the full bypass prevention strategy (pre-commit primary + prepare-commit-msg secondary backstop, `LEFTHOOK=0` known vector, CI/CD ultimate gate).
- **Pre-push Hook**: Heavier checks too slow for pre-commit — `cargo nextest run` (full test suite), `cargo deny check` (license/security), `cargo audit` (vulnerability scanning), `cargo doc --workspace --no-deps` (docs compile), `cargo llvm-cov` (code coverage) — all running in parallel via lefthook's `parallel: true`
- **Commit-msg Hook**: `cocogitto` (binary: `cog`) enforces Conventional Commit format per constitution requirement — the `cog verify --file {1}` command validates the commit message file
- **Post-merge & Post-checkout Hooks**: Background `cargo build` to pre-populate build cache after pull/merge and branch switches
- **Configurability**: Default checks (fmt + clippy + cargo check) with opt-in to additional checks via lefthook config
