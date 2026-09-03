# Feature Specification: CI/CD Pipeline for Leiden-Rust

**Feature Branch**: `006-ci-cd-pipeline`

**Created**: 2026-09-03

**Status**: Draft

**Input**: User description: "Building a professional CI/CD pipeline for leiden-rust—a multi-crate Cargo workspace comprising an algorithmic core library (leiden), a command-line tool (leiden-cli), and an interactive terminal application (leiden-tui) built with Ratatui—presents technical constraints that generic CI templates fail to address..."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Automated Quality Gates on Every Push (Priority: P1)

As a developer, I want every push and pull request to automatically run formatting checks, linting, security audits, and tests so that code quality issues are caught before they reach the main branch.

**Why this priority**: This is the foundational CI capability that prevents broken code from being merged. Without it, all other quality guarantees fail.

**Independent Test**: Can be fully tested by pushing a commit with a formatting error or lint violation and observing the CI pipeline reject it with clear error messages.

**Acceptance Scenarios**:

1. **Given** a pull request with improperly formatted code, **When** CI runs, **Then** the pipeline fails with a clear formatting error message
2. **Given** a pull request with a Clippy warning, **When** CI runs, **Then** the pipeline fails with the specific lint violation
3. **Given** a pull request with a dependency vulnerability, **When** CI runs, **Then** the pipeline fails with the advisory details
4. **Given** a pull request that passes all checks, **When** CI runs, **Then** the pipeline succeeds and shows a green status

---

### User Story 2 - Workspace-Aware Test Execution (Priority: P1)

As a developer, I want the CI pipeline to detect which crates changed and only run tests for affected crates so that trivial changes (like documentation or UI-only edits) don't waste time recompiling the entire workspace.

**Why this priority**: Monolithic CI jobs waste runner minutes and degrade developer feedback loops. This directly addresses the workspace isolation requirement.

**Independent Test**: Can be fully tested by making a documentation-only change and observing that only relevant test jobs execute while unrelated crate tests are skipped.

**Acceptance Scenarios**:

1. **Given** a change only to `crates/leiden-tui/`, **When** CI runs, **Then** only TUI-related tests execute and core/CLI tests are skipped
2. **Given** a change to `Cargo.toml` or workspace configuration, **When** CI runs, **Then** all crate tests execute
3. **Given** a change only to `crates/leiden/`, **When** CI runs, **Then** core tests execute and TUI tests are skipped

---

### User Story 3 - Headless TUI Testing (Priority: P2)

As a developer, I want the Ratatui TUI tests to run reliably in CI without requiring a real terminal so that interactive application code can be tested in automated environments.

**Why this priority**: Terminal user interfaces depend on capabilities not available in standard CI runners. This addresses the "Headless Terminal Trap" constraint.

**Independent Test**: Can be fully tested by running TUI tests in a CI environment without a PTY and observing that tests pass using in-memory rendering buffers.

**Acceptance Scenarios**:

1. **Given** a TUI component test using TestBackend, **When** CI runs, **Then** the test renders to an in-memory buffer and asserts on cell contents without terminal initialization
2. **Given** an integration test requiring terminal interaction, **When** CI runs, **Then** the test executes under a virtual PTY with fixed dimensions
3. **Given** a TUI test at below-minimum terminal dimensions (79x23), **When** the test runs, **Then** the dimension warning overlay renders and no panic occurs

---

### User Story 4 - Deterministic Property-Based Testing (Priority: P2)

As a developer, I want property-based tests to be deterministic across CI runs so that flaky tests don't cause false failures and discovered edge cases are consistently re-tested.

**Why this priority**: Proptest regression caching ensures that once a failing seed is found, it's immediately re-tested on subsequent runs, preventing regressions from slipping through.

**Independent Test**: Can be fully tests by observing that a proptest failure in one CI run causes the same seed to be re-tested in the next run.

**Acceptance Scenarios**:

1. **Given** a proptest discovers a failing edge case, **When** CI runs, **Then** the failing seed is cached and re-tested on subsequent runs
2. **Given** a CI run with no new proptest failures, **When** tests execute, **Then** all property tests pass with deterministic results

---

### User Story 5 - Cross-Platform Release Automation (Priority: P3)

As a maintainer, I want pushing a semantic version tag to automatically build, package, and publish binaries for Linux, macOS, and Windows so that releases are consistent and reproducible.

**Why this priority**: Manual error-prone release process. Automation ensures consistent artifacts with proper checksums.

**Independent Test**: Can be fully tested by pushing a version tag and observing that GitHub Release is created with all platform artifacts and SHA-256 checksums.

**Acceptance Scenarios**:

1. **Given** a tag `v1.2.0` is pushed, **When** the release pipeline triggers, **Then** binaries are built for Linux (x86_64 musl, aarch64 musl), macOS (aarch64, x86_64), and Windows (x86_64 MSVC)
2. **Given** a release build completes, **When** artifacts are packaged, **Then** each platform has a tarball/zip containing binaries, README, and LICENSE
3. **Given** artifacts are uploaded, **When** the release publishes, **Then** a SHA256SUMS.txt checksum manifest is included in the release

---

### Edge Cases

- What happens when a proptest regression file is found but the cache is cold (first run)? → Fail immediately and write regression file
- How does the system handle a TUI test that requires terminal dimensions below the minimum threshold? → Dimension warning overlay renders, no panic occurs
- What happens when a cross-platform build fails for one target but succeeds for others? → Entire release fails (no partial releases)
- How does the pipeline behave when only documentation or non-code files change? → Run formatting and linting checks only, skip tests
- What happens when a dependency audit finds a vulnerability in a transitive dependency? → Pipeline fails with advisory details (same as direct dependency)
- What happens when fixtures directory changes? → Parent crate tests are triggered
- What happens when TUI dimensions are exactly at minimum threshold (80x24)? → Normal rendering occurs without overlay
- What happens when TUI dimensions are ultrawide (240x60)? → Graph canvas scaling maintains aspect ratio constraints
- What happens when the CI cache expires (GitHub's 7-day TTL)? → Treat as cold cache: re-run full test exploration, write new regression files
- What happens when the cache exceeds GitHub's 10GB quota? → Log warning, proceed without caching (graceful degradation to standard dependency download)

## Appendix A: Path Filtering Patterns

| Pattern | Crate Affected | Tests Triggered |
|---------|----------------|-----------------|
| `crates/leiden/**` | leiden (core) | Core + CLI + TUI (downstream dependents) |
| `crates/leiden-cli/**` | leiden-cli | CLI only |
| `crates/leiden-tui/**` | leiden-tui | TUI only |
| `fixtures/**` | leiden (parent) | Core tests |
| `Cargo.toml`, `Cargo.lock` | All crates | Full workspace tests |
| `*.md`, non-code files | None | Formatting + linting only |

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST detect changed files and run tests for affected crates based on path filtering that respects workspace dependency topology (e.g., changes to the `leiden` core crate MUST trigger tests for downstream dependents `leiden-cli` and `leiden-tui`). Path filtering MUST use the following explicit patterns: `crates/leiden/**` (core crate), `crates/leiden-cli/**` (CLI crate), `crates/leiden-tui/**` (TUI crate), `fixtures/**` (triggers parent crate tests), `Cargo.toml` and `Cargo.lock` (triggers full workspace). When multiple crate changes are detected, the UNION of all affected crates MUST be tested. Documentation-only changes (`.md` files, non-code files) MUST run formatting and linting checks but skip tests. Workspace configuration changes MUST trigger full workspace test execution
- **FR-002**: System MUST enforce code formatting standards on all crates and fail on formatting violations
- **FR-003**: System MUST run static analysis (linting) on all targets and features with warnings treated as errors, failing on any lint violation
- **FR-004**: System MUST validate dependencies against security advisories and approved license policies, failing on any policy violation
- **FR-005**: System MUST execute unit tests, integration tests, and property-based tests for each crate with parallel test execution
- **FR-006**: System MUST cache property-based test regression seeds to ensure deterministic re-testing of discovered edge cases. Cached seeds MUST be replayed before random exploration on each test run. New failures MUST fail immediately and write the regression file. Cache MUST be invalidated when test source files or dependencies change
- **FR-007**: System MUST run TUI tests using in-memory TestBackend for unit tests without terminal initialization
- **FR-008**: System MUST allocate virtual PTY for integration tests that require terminal interaction (raw mode, input events). PTY MUST support configurable dimensions with default 80x24, below-minimum 79x23, and ultrawide 240x60 for geometry-sensitive test scenarios
- **FR-009**: System MUST verify benchmark harness compilation without executing benchmarks on shared runners
- **FR-010**: System MUST build release binaries for Linux x86_64 (static), Linux aarch64 (static), macOS aarch64, macOS x86_64, and Windows x86_64. If any target fails to build, the entire release MUST fail (no partial releases). Specific target triples and linkage requirements are documented in Technical Tooling
- **FR-011**: System MUST strip debug symbols from release binaries on Unix systems
- **FR-012**: System MUST generate SHA-256 checksums for all release artifacts
- **FR-013**: System MUST publish GitHub Release with all artifacts and checksum manifest when a semantic version tag is pushed
- **FR-014**: System MUST use dependency caching to minimize CI runtime. Cache MUST be saved on successful job completion and invalidated when `Cargo.lock` or dependency manifests change. On cache quota exceeded errors, system MUST log warning and proceed without caching (graceful degradation)
- **FR-015**: System MUST cancel in-progress runs when new commits are pushed to the same branch
- **FR-016**: System MUST communicate CI status via GitHub native PR status checks only (no external notifications)
- **FR-017**: System MUST verify that all public API items have documentation by building workspace documentation and failing on missing docs (enforces `missing_docs = deny` constitutional requirement)
- **FR-018**: System MUST execute performance-critical tests under release profile optimization to validate asymptotic performance contracts (enforces constitutional `--release` test gate)

### Key Entities

- **CI Pipeline**: The continuous integration workflow triggered on push/PR that runs quality gates and tests
- **CD Pipeline**: The continuous delivery workflow triggered on version tags that builds and publishes release artifacts
- **Proptest Regression**: Cached failing test seeds that ensure deterministic re-testing of edge cases
- **TestBackend**: In-memory terminal buffer used for headless TUI testing
- **Virtual PTY**: Pseudo-terminal allocated for integration tests requiring terminal capabilities
- **Release Artifact**: Packaged binary distribution for a specific platform target

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Code formatting violations are caught within 2 minutes of push
- **SC-002**: Static analysis (lint) violations are caught within 5 minutes of push
- **SC-003**: Dependency vulnerabilities are detected within 5 minutes of push
- **SC-004**: Test results for affected crates are available within 10 minutes of push
- **SC-005**: TUI tests run successfully in CI without terminal initialization errors
- **SC-006**: Proptest regression seeds are consistently re-tested across CI runs (zero flaky failures from cached seeds)
- **SC-007**: Release artifacts for all 5 platform targets are built and published within 15 minutes of version tag push
- **SC-008**: CI pipeline skips unnecessary crate tests when only one crate changes (at least 50% reduction in test time for isolated changes)
- **SC-009**: All release artifacts include valid SHA-256 checksums for integrity verification
- **SC-010**: Each job in the CI pipeline has a 30-minute hard timeout to prevent hung builds from consuming runner resources. Per-job timeout provides granular protection and allows other jobs to continue if one fails

## Clarifications

### Session 2026-09-03

- Q: If a cross-platform release build fails for one target but succeeds for others, what should happen? → A: Fail the entire release if any target fails (no partial releases)
- Q: How should CI failures be communicated to the development team? → A: GitHub native PR status checks only (no external notifications)
- Q: When proptest discovers a failing edge case on a cold cache, what should happen? → A: Fail immediately and write regression file
- Q: When only documentation or non-code files change, what should CI do? → A: Run formatting and linting checks only, skip tests
- Q: What is the maximum acceptable CI pipeline duration? → A: 30 minutes timeout

## Assumptions

- GitHub Actions is the CI/CD platform (already used in the project)
- The repository has access to standard GitHub Actions runners (ubuntu-latest, macos-13, macos-14, windows-latest)
- The existing CI workflow will be extended to add Swatinem/rust-cache for dependency caching (migration from current setup)
- The existing CI workflow will be extended to use `cross` tool for musl target compilation (migration from current setup)
- The workspace has deny.toml configured for security/license validation
- TUI tests are architected with TestBackend for unit tests (per Anti-Pattern 2 guidance)
- Proptest regression directory (target/proptest-regressions/) is cacheable
- The workspace follows a dependency topology where `leiden` core is a dependency of both `leiden-cli` and `leiden-tui` (core changes propagate to downstream crates)
- Current workspace test duration baseline (full workspace, debug build) is approximately 8-12 minutes on GitHub Actions ubuntu-latest runners (used for SC-008 measurement)

## Technical Tooling (referenced by Functional Requirements)

*Note: Specific tooling choices are documented here. Functional Requirements define capabilities; these are the planned implementations.*

- Code formatting enforcement: `cargo fmt --check` (FR-002)
- Static analysis: `cargo clippy --workspace --all-targets --all-features` with `-D warnings` (FR-003)
- Dependency audit: `cargo deny check` (FR-004)
- Test runner: cargo-nextest (FR-005)
- Property-based test regression cache: proptest native format in `target/proptest-regressions/`, cache key based on hash of `Cargo.lock` and test source files (FR-006)
- Virtual PTY allocation: `portable-pty` crate (primary) or `script` command (fallback) (FR-008)
- Release target triples: `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`, `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-pc-windows-msvc` (FR-010)
- Cross-compilation for musl targets: `cross` tool (FR-010)
- Release publishing: GitHub Releases via softprops/action-gh-release (FR-013)
