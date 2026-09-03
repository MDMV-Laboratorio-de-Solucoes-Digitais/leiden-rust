# Data Model: CI/CD Pipeline for Leiden-Rust

**Feature**: CI/CD Pipeline (006-ci-cd-pipeline)
**Date**: 2026-09-03

This document describes the key entities and their relationships in the CI/CD pipeline. Since this is a CI infrastructure feature (not a code feature), the "entities" are workflow job outputs and configuration values rather than domain objects.

---

## Entity: DocScope

**Description**: Determines the scope of documentation generation based on changed files.

**Fields**:
| Field | Type | Description |
|-------|------|-------------|
| `scope` | `enum {workspace, crate, skip}` | The computed doc build scope |
| `crate` | `Option<String>` | The single crate name when `scope == crate` |

**State Transitions**:
```
[paths-filter outputs] → [counting step] → DocScope
                                          ├── workspace (config changed OR 2+ crates)
                                          ├── crate (exactly 1 crates)
                                          └── skip (no code changes)
```

**Validation Rules**:
- `scope == crate` → `crate` field MUST be set to a valid workspace member name
- `scope == workspace` → `crate` field is None
- `scope == skip` → `crate` field is None
- `workspace-config` change always forces `scope == workspace` regardless of crate count

---

## Entity: ChangeDetection

**Description**: Output of the path-filtering step that identifies which subsystems changed.

**Fields**:
| Field | Type | Description |
|-------|------|-------------|
| `core` | `bool` | True if `crates/leiden/**` or `fixtures/**` changed |
| `cli` | `bool` | True if `crates/leiden-cli/**` changed |
| `tui` | `bool` | True if `crates/leiden-tui/**` changed |
| `meta` | `bool` | True if workspace config or CI config changed |
| `any_code` | `bool` | True if any code file changed (composite of core/cli/tui/meta) |

**Relationships**:
- `ChangeDetection` → `DocScope`: The counting step consumes ChangeDetection outputs to compute DocScope
- `ChangeDetection` → Job conditions: Individual test jobs use these booleans for conditional execution

---

## Entity: ProptestRegressionCache

**Description**: Configuration for caching proptest regression seeds across CI runs.

**Fields**:
| Field | Type | Description |
|-------|------|-------------|
| `cache_paths` | `Vec<Path>` | Directories to cache: `target/proptest-regressions/`, `crates/*/target/proptest-regressions/` |
| `primary_key` | `String` | `proptest-regressions-${{ runner.os }}-${{ hashFiles('crates/leiden/tests/*.rs') }}` |
| `restore_keys` | `Vec<String>` | Fallback: `proptest-regressions-${{ runner.os }}-` |

**State Transitions**:
```
[cache miss] → [proptest runs fresh] → [on failure: writes seed file] → [post-step upload]
                                                                    ↓
[cache hit] ← [next CI run] ← [seed replayed first] ← [deterministic re-test]
```

**Validation Rules**:
- Cache hit on primary key → full seed replay (deterministic)
- Cache hit on restore-keys only → partial seed replay (older seeds)
- Cache miss → cold start, no replay, proptest runs full random exploration

---

## Entity: ReleaseTarget

**Description**: A platform target for cross-compiled release binaries.

**Fields**:
| Field | Type | Description |
|-------|------|-------------|
| `os` | `String` | GitHub Actions runner OS (e.g., `ubuntu-latest`, `macos-14`, `windows-latest`) |
| `target` | `String` | Rust target triple (e.g., `x86_64-unknown-linux-musl`) |
| `use_cross` | `bool` | Whether to use `cross` tool for compilation |
| `archive_ext` | `enum {tar.gz, zip}` | Archive format for this target |
| `bin_ext` | `String` | Binary extension (`.exe` for Windows, `""` otherwise) |

**Known Targets** (from release.yml matrix):
| OS | Target | Cross? | Archive | Bin Ext |
|----|--------|--------|---------|---------|
| ubuntu-latest | x86_64-unknown-linux-musl | Yes | tar.gz | "" |
| ubuntu-latest | aarch64-unknown-linux-musl | Yes | tar.gz | "" |
| macos-14 | aarch64-apple-darwin | No | tar.gz | "" |
| macos-13 | x86_64-apple-darwin | No | tar.gz | "" |
| windows-latest | x86_64-pc-windows-msvc | No | zip | .exe |

---

## Entity: ReleaseArtifact

**Description**: A packaged binary distribution for a specific platform target.

**Fields**:
| Field | Type | Description |
|-------|------|-------------|
| `target` | `ReleaseTarget` | The platform target this artifact was built for |
| `package_name` | `String` | `leiden-${TAG_NAME}-${TARGET}` |
| `archive_path` | `Path` | Path to the tar.gz or zip file |
| `checksum_path` | `Path` | Path to the `.sha256` checksum file |
| `binaries` | `Vec<Path>` | Paths to the stripped binaries (leiden-cli, leiden-tui) |
| `includes_readme` | `bool` | Whether README.md was included |
| `includes_license` | `bool` | Whether LICENSE was included |

**Relationships**:
- `ReleaseArtifact` → `ReleaseTarget`: Each artifact is associated with exactly one target
- `ReleaseArtifact` → `SHA256SUMS.txt`: All artifact checksums are aggregated into a single manifest

**Validation Rules**:
- `archive_path` MUST exist and be non-empty
- `checksum_path` MUST exist and contain a valid SHA-256 hash
- Both `leiden-cli` and `leiden-tui` binaries MUST be present in the archive
- Binaries on Unix targets MUST have debug symbols stripped

---

## Entity: SHA256SUMS

**Description**: Aggregated checksum manifest for all release artifacts.

**Fields**:
| Field | Type | Description |
|-------|------|-------------|
| `file_path` | `Path` | Always `release-assets/SHA256SUMS.txt` |
| `entries` | `Vec<(String, String)>` | Pairs of (hash, filename) |

**State Translations**:
```
[per-target sha256sum] → [download all artifacts] → [concatenate *.sha256] → SHA256SUMS.txt
```

---

## Entity: PipelineTimeout

**Description**: Timeout configuration to prevent hung builds from consuming runner resources.

**Fields**:
| Field | Type | Description |
|-------|------|-------------|
| `job_timeout_minutes` | `u32` | 30 minutes per job (SC-010) |

**Validation Rules**:
- Any job exceeding this timeout is forcibly terminated by GitHub Actions
- This is a hard limit, not a warning threshold

---

## Entity Relationships Diagram

```
┌─────────────────┐
│ ChangeDetection │─────────────┐
│ (paths-filter)  │             │
└─────────────────┘             │
        │                       │
        ▼                       ▼
┌─────────────────┐     ┌──────────────┐
│   DocScope      │     │  Test Jobs   │
│ (counting step) │     │ (conditional)│
└─────────────────┘     └──────────────┘
        │
        ▼
┌─────────────────┐
│   Docs Job      │
│ (workspace/crate│
│  /skip)         │
└─────────────────┘

┌─────────────────────┐
│ ProptestRegression  │
│     Cache           │
│ (actions/cache)     │
└─────────────────────┘

┌─────────────────┐     ┌─────────────────┐
│  ReleaseTarget  │────▶│ ReleaseArtifact │
│  (matrix)       │     │ (per-platform)  │
└─────────────────┘     └─────────────────┘
                               │
                               ▼
                        ┌─────────────────┐
                        │  SHA256SUMS     │
                        │  (aggregated)   │
                        └─────────────────┘
```
