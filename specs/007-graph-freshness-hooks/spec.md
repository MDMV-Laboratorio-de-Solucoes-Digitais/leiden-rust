# Feature Specification: Knowledge Graph Freshness Hooks

**Feature Branch**: `007-graph-freshness-hooks`

**Created**: 2026-09-03

**Status**: Draft

**Input**: User description: "Git hooks — a pre-commit hook that warns (not regenerates) when the graph is stale relative to recent structural changes. Lightweight: compare graphify-out/ mtime against crates/ mtime, print a reminder if outdated. Doesn't block commits, just nudges. Full regeneration in a hook would be too slow and assume every developer has graphify installed. CI/CD — a check that fails if the graph is obviously stale (e.g., a crate was added/removed but graphify-out wasn't refreshed). This enforces Principle VIII without persisting the graph. The CI job runs graphify in a --check mode (if supported) or compares a content hash of the current structure against what the graph describes."

## Clarifications

### Session 2026-09-03

- Q: What counts as a "structural change" that triggers freshness detection? → A: Workspace membership only — adding, renaming, or removing crates in `Cargo.toml` and `crates/`. Internal modifications within an existing crate (modules, public API, source files) do not trigger it.
- Q: How should the hook detect a structural change — mtime comparison or diff inspection? → A: Inspect the staged diff — check whether staged changes touch `[workspace].members` in `Cargo.toml` or add/remove entries under `crates/`. Avoids mtime false positives.
- Q: How should CI detect structural divergence — compare members against graph-recorded names, or run graphify --check? → A: Compare current workspace members (`Cargo.toml` + `crates/` entries) against the crate names recorded in `graphify-out/` (e.g., from `graphify-out/graph.json`). Lightweight; does not require running graphify in CI.
- Q: How should the pre-commit hook be installed and enabled? → A: Configure via lefthook (already used in this project) — add a `pre-commit` hook entry to the existing lefthook manifest (`lefthook.yml`). No separate installation step; developers already using lefthook get it automatically.
- Q: What should the regeneration command be — direct graphify invocation or a wrapper? → A: A wrapper script (e.g., `scripts/regen-graph.sh` or `make graphify`) that encapsulates the exact graphify invocation. Single obvious command for the reminder message; versioned alongside the project.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Pre-commit freshness reminder (Priority: P1)

A developer commits a change that modifies the project's structure (adds a crate, removes a module, renames a crate). Before the commit lands, they see a friendly, non-blocking reminder that the knowledge graph appears stale relative to the structural change. The reminder tells them how to regenerate it. The commit is not blocked — the developer can choose to regenerate now, or later, or not at all if the change is trivial.

**Why this priority**: This is the primary touchpoint where staleness can be caught earliest — at commit time, when the structural change is fresh in the developer's mind and regeneration is cheapest.

**Independent Test**: Can be fully tested by making a structural change (e.g., touching a file in `crates/`), running `git commit`, and observing that a reminder is printed when `graphify-out/` is older than the structural change.

**Acceptance Scenarios**:

1. **Given** `graphify-out/` exists and is older than the most recent change under `crates/`, **When** a developer commits, **Then** a non-blocking reminder is printed telling them the graph may be stale and how to regenerate it.
2. **Given** `graphify-out/` is newer than all changes under `crates/`, **When** a developer commits, **Then** no reminder is printed.
3. **Given** `graphify-out/` does not exist, **When** a developer commits, **Then** no reminder is printed (staleness detection is best-effort; absence is not a failure).
4. **Given** a developer commits a non-structural change (e.g., a doc edit) while the graph is technically stale, **When** the commit proceeds, **Then** no reminder is printed (the hook compares against structural paths only).

---

### User Story 2 - CI enforcement on pull requests (Priority: P1)

A developer opens a pull request that adds, removes, or renames a crate (or otherwise changes workspace membership). The CI pipeline runs a freshness check. If the knowledge graph does not reflect the post-change structure, the check fails with a clear message telling the developer to regenerate the graph. This enforces Constitution Principle VIII ("MUST be refreshed when significant structural or cross-crate API boundaries are modified") without requiring CI to persist the regenerated graph.

**Why this priority**: CI is the backstop that catches what the pre-commit hook misses — PRs from developers who ignored the reminder, or structural changes that don't naturally trigger the hook.

**Independent Test**: Can be fully tested by opening a PR that adds a new crate (or modifies `[workspace].members` in `Cargo.toml`) without regenerating the graph, and observing that the CI freshness check fails.

**Acceptance Scenarios**:

1. **Given** a PR modifies `Cargo.toml` workspace membership or `crates/` structure, **When** the CI freshness check runs, **Then** it compares the current structure against what the graph describes and fails if they diverge.
2. **Given** a PR makes only documentation changes, **When** the CI freshness check runs, **Then** it passes (no structural change detected).
3. **Given** the graph accurately reflects the post-change structure, **When** the CI freshness check runs, **Then** it passes.
4. **Given** `graphify-out/` does not exist or is unreadable, **When** the CI freshness check runs, **Then** the check fails with a clear message (the graph is mandatory per Principle VIII).

---

### User Story 3 - Regeneration made trivial (Priority: P2)

A developer who sees the reminder (or whose CI check failed) can regenerate the knowledge graph with a single command — no need to remember graphify flags or invocation details. The project exposes a regeneration target/script that any developer can run.

**Why this priority**: Reminders and CI failures are only useful if the remedy is obvious and frictionless. If regeneration is hard, developers will ignore the nudges.

**Independent Test**: Can be fully tested by running the documented regeneration command and confirming that `graphify-out/` is updated to reflect the current structure.

**Acceptance Scenarios**:

1. **Given** the project root, **When** a developer runs the documented regeneration command, **Then** `graphify-out/` is refreshed with the current structure.
2. **Given** the pre-commit reminder is printed, **When** the developer follows the regeneration instruction, **Then** the next commit prints no reminder.

---

### Edge Cases

- **graphify-out/ absent**: the hook stays silent (best-effort); CI fails with a clear message.
- **graphify tool not installed locally**: the hook detects this silently and prints no reminder (avoids blocking developers who don't use graphify); CI has the tool installed.
- **First-time checkout with no graph**: the hook is silent; CI fails until someone regenerates.
- **Massive structural refactors** (e.g., workspace flatten): the reminder fires on the first commit after the change; regeneration is the developer's responsibility.
- **Non-structural changes**: the hook compares only against `crates/` and `Cargo.toml` (workspace membership) mtimes, not the entire tree.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST provide a pre-commit hook that inspects the staged diff for workspace membership changes (additions, renamings, or removals of crates in `[workspace].members` in `Cargo.toml` or entries under `crates/`) and prints a non-blocking reminder when `graphify-out/` predates the most recent such change. Internal modifications within an existing crate MUST NOT trigger the reminder.
- **FR-002**: The pre-commit hook MUST NOT block or abort the commit under any condition.
- **FR-003**: The pre-commit hook MUST remain silent when `graphify-out/` is newer than all structural paths, when `graphify-out/` does not exist, or when graphify is not installed.
- **FR-004**: The system MUST provide a CI check that fails when `graphify-out/` does not reflect the current workspace structure. Detection MUST compare current workspace members (`[workspace].members` in `Cargo.toml` plus `crates/` directory entries) against the crate names recorded in `graphify-out/` (e.g., from `graphify-out/graph.json`).
- **FR-005**: The CI check MUST NOT require running the graphify tool in CI. Divergence is detected solely by reading the recorded crate names from the existing `graphify-out/` and comparing them against the current workspace members.
- **FR-006**: The CI check MUST pass when no structural change is detected, regardless of graph freshness.
- **FR-007**: The system MUST provide a wrapper script (e.g., `scripts/regen-graph.sh` or `make graphify`) that encapsulates the graphify invocation and refreshes `graphify-out/` at the repository root. The reminder message MUST name this script.
- **FR-008**: The pre-commit hook MUST be configured via the project's existing lefthook manifest (`lefthook.yml`) — no separate installation step. The CI check MUST be additive and non-blocking on non-structural PRs.
- **FR-009**: The reminder message MUST include the regeneration command so the next action is obvious.

### Key Entities *(include if data involved)*

- **Knowledge graph** (`graphify-out/`): the codebase knowledge graph generated by graphify. Gitignored; local maintenance only. Its freshness is measured relative to structural source paths.
- **Structural paths**: the set of paths whose modification triggers freshness comparison — `crates/` (crate directories, membership only — not internal file changes) and `Cargo.toml` (workspace membership). Internal modifications within an existing crate (modules, public API, source files) are explicitly excluded.
- **Structural fingerprint**: a content-derived summary of the current workspace structure (crate names, member list) used by CI to detect divergence from the graph without regenerating it.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A developer who makes a structural change without regenerating the graph sees a non-blocking reminder at commit time in 100% of cases where `graphify-out/` predates the structural change.
- **SC-002**: A PR that modifies workspace membership or crate structure without regenerating the graph fails the CI freshness check; a PR with no structural changes passes it.
- **SC-003**: The regeneration command is documented and executable from the repository root in a single step.
- **SC-004**: Developers who do not have graphify installed are never blocked by the pre-commit hook.

## Assumptions

- `graphify-out/` is the canonical knowledge graph location and is gitignored (regeneration is a local maintenance action, not a committed artifact).
- The graphify tool is available in the CI environment but may not be installed on every developer's machine.
- Structural changes are defined as changes to `Cargo.toml` workspace membership or the `crates/` directory tree. Documentation-only and non-structural changes do not require graph regeneration.
- The pre-commit hook is best-effort: it may produce false negatives (miss a stale graph) but should not produce false positives (nag when the graph is fresh).
- Constitution Principle VIII (Knowledge Graph Context) governs when regeneration is mandatory; this feature enforces that principle at commit and PR time.
- The project uses lefthook as its git hook manager; the pre-commit hook is added to the existing `lefthook.yml` manifest rather than `.git/hooks/`.
