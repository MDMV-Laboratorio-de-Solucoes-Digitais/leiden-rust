# Implementation Plan: Lefthook Pre-commit Hooks

**Branch**: `008-lefthook-precommit-hooks` | **Date**: 2026-09-03 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/008-lefthook-precommit-hooks/spec.md`

## Summary

Configure lefthook to run fast quality checks on commit (fmt + clippy + check, <5s), heavier validation on push (nextest, deny, audit, doc, llvm-cov), enforce Conventional Commits via cocogitto in commit-msg, and warm the build cache post-merge/post-checkout. The system uses `prepare-commit-msg` as a secondary non-bypassable formatting backstop for `--no-verify` scenarios, while acknowledging `LEFTHOOK=0` as a known bypass vector with CI/CD as the ultimate enforcement gate.

## Technical Context

**Language/Version**: Rust stable edition 2024 (workspace lints configured per `rust-code-rigor.md`)

**Primary Dependencies**: lefthook (binary, git hook manager), cocogitto (Conventional Commits), cargo-nextest (test runner), cargo-deny (license/security), cargo-audit (vulnerabilities), cargo-llvm-cov (code coverage)

**Storage**: N/A — configuration files only (`lefthook.yml`, `.lefthook/`)

**Testing**: Manual validation via git operations (commit, push, merge, checkout); hooks run as git lifecycle commands

**Target Platform**: Linux/macOS developer machines; lefthook provides cross-platform git hook management

**Project Type**: Infrastructure/DevTooling configuration (no library or binary crates created)

**Performance Goals**: Pre-commit hooks complete in <5 seconds for typical commits (1 file, up to 10 files/500 lines benchmark)

**Constraints**: Must not compromise DX; must work cross-platform; must not block commits for non-Rust changes; must run checks in parallel to meet performance target

**Scale/Scope**: Single `lefthook.yml` configuration file at repo root; affects all developers using the repository

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Library-First & Domain Modeling | **N/A** | No library code; infrastructure configuration only |
| II. Strict Lint Compliance | **PASS** | No Rust code written; hooks enforce existing lint config |
| III. Panic-Free Error Propagation | **N/A** | No Rust code written |
| IV. Documentation & Visibility Discipline | **N/A** | No public API; spec documents behavior |
| V. Test-First | **PASS** | quickstart.md scenarios 1-12 serve as the test-first validation suite for this infrastructure; each hook is validated via git operations before merge |
| VI. Observability & I/O Discipline | **PASS** | Hook output provides clear success/failure feedback per FR-006 |
| VII. Dependency & Build Rigor | **PASS** | lefthook is a binary tool, not a Rust dependency; no Cargo.toml changes |
| VIII. Knowledge Graph Context | **PASS** | Research conducted via `/graphify` queries where applicable |

**Development Workflow Alignment**:
- Spec Kit flow: This plan is part of the spec workflow
- CI pipeline: Pre-push hooks run heavier checks too slow for pre-commit (nextest, deny, audit, doc, llvm-cov)
- Atomic commits: commit-msg hook enforces Conventional Commit format
- Review: Plan reviewed via spec workflow

**Gate Result**: PASS — no violations. Feature is infrastructure configuration that supports constitution principles without violating any.

## Project Structure

### Documentation (this feature)

```text
specs/008-lefthook-precommit-hooks/
├── spec.md                # Feature specification
├── plan.md                # This file
├── research.md            # Phase 0 output
├── data-model.md          # Phase 1 output
├── quickstart.md          # Phase 1 output
├── checklists/
│   └── requirements.md    # Spec quality checklist
└── research-hooks.md      # Research from clarify phase
```

### Source Code (repository root)

```text
lefthook.yml               # Main lefthook configuration (NEW)
.lefthook/                 # Lefthook scripts directory (if needed)
```

**Structure Decision**: Single `lefthook.yml` at repo root. No crate or source code changes. This is pure infrastructure configuration.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations — this section intentionally left blank.

---

## Phase 0: Research & Unknowns

### Unknowns Resolved

| Unknown | Resolution | Source |
|---------|------------|--------|
| `prepare-commit-msg` tree modification | Git writes tree AFTER hook runs (line 1706 vs 1116 in `builtin/commit.c`); re-staged files ARE included in current commit | Git source code analysis |
| `--no-verify` bypass behavior | Bypasses `pre-commit` and `commit-msg` but NOT `prepare-commit-msg`; `LEFTHOOK=0` bypasses all lefthook hooks | Git docs, lefthook docs |
| Pre-push check selection | Run heavier checks (nextest, deny, audit, doc, llvm-cov) that are too slow for pre-commit | Rust CI best practices |
| Conventional Commits enforcement | Use cocogitto in `commit-msg` hook | cocogitto docs |
| Cache warming approach | Background `cargo build` in `post-merge` and `post-checkout` | Git hooks best practices |

### Research Output

See [research.md](./research.md) for consolidated findings.

---

## Phase 1: Design & Contracts

### Data Model

See [data-model.md](./data-model.md) for the lefthook configuration structure.

### Contracts

**N/A** — This feature is internal infrastructure configuration with no external interfaces, public APIs, or user-facing contracts. The "contract" is the `lefthook.yml` configuration schema, documented in the data model.

### Quickstart Validation

See [quickstart.md](./quickstart.md) for validation scenarios.

---

## Implementation Approach

The implementation is a single `lefthook.yml` file with the following hook structure:

```yaml
pre-commit:
  parallel: true
  commands:
    fmt:
      glob: "*.rs"
      run: cargo fmt || true
      stage_fixed: true
    clippy:
      glob: "*.rs"
      run: cargo clippy --workspace --all-targets -- -D warnings
    check:
      glob: "*.rs"
      run: cargo check --workspace

prepare-commit-msg:
  commands:
    # NOTE: Uses `cargo fmt --all` (workspace-wide) rather than staged-files-only.
    # This is intentional — prepare-commit-msg is a non-bypassable backstop for
    # --no-verify scenarios (FR-003), so it formats the entire workspace as a
    # safety net. The pre-commit hook (FR-001) formats only staged files.
    fmt-backstop:
      # NOTE: stage_fixed:true only works for pre-commit in lefthook.
      # prepare-commit-msg must explicitly re-stage via `git add -u`.
      run: cargo fmt --all && git add -u

commit-msg:
  commands:
    conventional:
      run: cog verify --file {1}

pre-push:
  parallel: true
  commands:
    test:
      run: cargo nextest run
    deny:
      run: cargo deny check
    audit:
      run: cargo audit
    doc:
      run: cargo doc --workspace --no-deps
    coverage:
      run: cargo llvm-cov

post-merge:
  commands:
    cache-warm:
      run: nohup cargo build > /dev/null 2>&1 &

post-checkout:
  commands:
    cache-warm:
      run: nohup cargo build > /dev/null 2>&1 &
```

**Note**: Final implementation will be refined during `/speckit-tasks` and `/speckit-implement` based on this plan.

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Pre-push too slow for frequent pushers | Document `LEFTHOOK=0` as escape hatch; CI is ultimate gate |
| `cocogitto` not installed | Document as required tool; future dev setup spec handles installation |
| Cache-warming build conflicts with ongoing work | Use `nohup cargo build > /dev/null 2>&1 &` for true background execution; non-blocking. Concurrent builds from rapid merge/checkout overlap safely (redundant work, not corruption). |
| `prepare-commit-msg` formatting surprises | Document behavior clearly; `stage_fixed: true` is explicit |

---

## Next Steps

1. Review this plan
2. Run `/speckit-tasks` to generate task breakdown
3. Run `/speckit-implement` to create the `lefthook.yml`
