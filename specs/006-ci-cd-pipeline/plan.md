# Implementation Plan: CI/CD Pipeline for Leiden-Rust

**Branch**: `006-ci-cd-pipeline` | **Date**: 2026-09-03** | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/006-ci-cd-pipeline/spec.md`

**Note**: This template is filled in by the `/speckit-plan` command; its definition describes the execution workflow.

## Summary

This plan establishes a professional CI/CD pipeline for the `leiden-rust` multi-crate workspace. The pipeline addresses four technical constraints: (1) headless Ratatui testing via TestBackend and virtual PTY allocation, (2) deterministic property-based testing with proptest regression caching, (3) workspace-aware path filtering to avoid unnecessary recompilation, and (4) cross-platform release automation with SHA-256 checksum attestation. The implementation extends the existing `.github/workflows/ci.yml` with a new `docs` job for conditional documentation generation.

## Technical Context

**Language/Version**: Rust stable (edition 2024), toolchain pinned via `rust-toolchain.toml`, MSRV floor 1.88.0

**Primary Dependencies**:
- `ratatui = "0.30.2"` (TUI framework, requires rust_version 1.88.0)
- `crossterm` (terminal backend, used by Ratatui)
- `proptest` (property-based testing with regression caching)
- `cargo-nextest` (test runner with JUnit output)
- `cargo-deny` (security/license/duplicate dependency auditing)
- `criterion` (benchmarking - compile check only in CI)

**Storage**: N/A (CI pipeline, no persistent storage)

**Testing**: `cargo-nextest` for unit/integration/property tests; `TestBackend` for TUI unit tests; virtual PTY (`portable-pty` or `script`) for TUI integration tests

**Target Platform**: GitHub Actions runners: `ubuntu-latest`, `macos-13`, `macos-14`, `windows-latest`

**Project Type**: Multi-crate Cargo workspace (library + CLI + TUI desktop application)

**Performance Goals**:
- Formatting check: < 2 minutes (SC-001)
- Clippy lint: < 5 minutes (SC-002)
- Dependency audit: < 5 minutes (SC-003)
- Test results: < 10 minutes (SC-004)
- Full release build: < 15 minutes (SC-007)
- Pipeline timeout: 30 minutes (SC-010)

**Constraints**:
- `-D warnings` globally enforced via `RUSTFLAGS`
- Zero-panic production code (enforced by lint + panic trace grep)
- No benchmark execution on shared runners (compile check only)
- Proptest regression seeds must be cached for determinism
- TUI tests must not require real terminal hardware
- Partial releases are not allowed (all targets must succeed or release fails)

**Scale/Scope**: 3 crates (leiden, leiden-cli, leiden-tui), 5 platform targets for release, 1 GitHub Actions workflow (ci.yml) to be extended

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Compliance | Notes |
|-----------|------------|-------|
| I. Library-First & Domain Modeling | PASS | Pipeline enforces clean crate boundaries via path filtering |
| II. Strict Lint Compliance (NON-NEGOTIABLE) | PASS | `RUSTFLAGS: "-D warnings"` enforced; Clippy runs with workspace lints |
| III. Panic-Free Error Propagation | PASS | Panic trace grep test runs in CI; lints deny `unwrap_used`, `expect_used`, `panic` |
| IV. Documentation & Visibility Discipline | PASS | `cargo doc --workspace` runs; `missing_docs = deny` enforced |
| V. Test-First (NON-NEGOTIABLE) | PASS | CI runs unit, integration, property, and doc tests; bench compile check |
| VI. Observability & I/O Discipline | PASS | Pipeline uses structured GitHub Actions annotations, not print spam |
| VII. Dependency & Build Rigor | PASS | `cargo-deny` validates advisories, licenses, and duplicate versions |
| VIII. Knowledge Graph Context | PASS | `/graphify` queries used for architectural discovery |

**Gate Result**: PASS — No violations. The CI pipeline directly enforces multiple constitutional principles (II, III, IV, V, VII).

## Project Structure

### Documentation (this feature)

```text
specs/006-ci-cd-pipeline/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
leiden-rust/
├── .github/
│   └── workflows/
│       ├── ci.yml           # Extended: add docs job for conditional doc generation
│       └── release.yml      # Reference only (already exists from prior implementation)
├── crates/
│   ├── leiden/              # Algorithmic core - property tests run here
│   ├── leiden-cli/          # CLI binary - integration tests run here
│   └── leiden-tui/          # TUI binary - TestBackend + PTY tests run here
├── fixtures/                # Graph datasets for convergence regression tests
├── specs/                   # Spec-driven development contracts
├── deny.toml                # Security/license/duplicate dependency policies
├── clippy.toml              # Workspace lint configuration
├── rust-toolchain.toml      # Pinned compiler toolchain
└── Cargo.toml               # Workspace root with [workspace.lints] table
```

**Structure Decision**: The CI pipeline extends the existing `.github/workflows/ci.yml`. The new `docs` job integrates with the existing `changes` job outputs to determine doc scope (workspace-wide vs. crate-targeted). No new crates or modules are created — this is purely CI infrastructure.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

*No constitution violations — this section intentionally left blank.*

## Phase 0: Research

*See [research.md](research.md) for consolidated findings.*

Key research questions resolved:
1. How to implement conditional doc scope logic in GitHub Actions?
2. Best practices for caching proptest regressions in CI?
3. Existing patterns in the codebase for PTY allocation?

## Phase 1: Design

*See [data-model.md](data-model.md), [contracts/](contracts/), and [quickstart.md](quickstart.md) for design artifacts.*
