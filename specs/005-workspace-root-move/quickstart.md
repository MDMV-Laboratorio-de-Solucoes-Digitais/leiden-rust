# Quickstart: Validate the Workspace Root Move

**Feature**: 005-workspace-root-move

Run these scenarios after implementing PR 1 (and again after PR 2). They
prove the flattened layout end-to-end. Reference details:
[repository-layout.md](./contracts/repository-layout.md) ·
[ci-and-commands.md](./contracts/ci-and-commands.md) ·
[data-model.md](./data-model.md).

## Prerequisites

- Rust stable toolchain honoring `rust-toolchain.toml` (MSRV ≥ 1.88.0)
- `cargo-nextest` (alias `ct`), `cargo-deny` installed
- PR 1 branch checked out (`005-workspace-root-move`)

## Scenario 1 — Root inventory (User Story 1, acceptance 2 & 3)

```sh
ls Cargo.toml Cargo.lock rust-toolchain.toml clippy.toml deny.toml proptest.toml README.md \
   crates/leiden crates/leiden-cli crates/leiden-tui fixtures AGENTS.md specs .specify
test ! -e leiden && echo "FR-008 OK: no nested leiden/"
```

**Expected**: every path exists; no `leiden/` directory at the root.

## Scenario 2 — Full verification suite from the root (FR-004, SC-004, SC-001)

Run entirely from the repository root, no `cd`:

```sh
cargo check --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
ct --workspace
ct --workspace --release
cargo doc --workspace --no-deps
cargo deny --config deny.toml check
```

**Expected**: all exit 0 with the same pass/fail profile as `dev` before the
move. If `ct` is unavailable, substitute `cargo nextest run`.

## Scenario 3 — Fixture-path integrity (the one edited test)

```sh
cargo test -p leiden --test observability_checklist
cargo test -p leiden-cli --test cli_round_trip
cargo test -p leiden --test sc002_fixture_cardinality_phase1
```

**Expected**: the checklist test finds
`specs/001-leiden-algorithm/checklists/observability.md` at the new
root-relative location; fixture-driven tests load from `fixtures/` without
edits (uniform-shift resolution still valid).

## Scenario 4 — Rename purity & history integrity (User Story 2, SC-002)

```sh
git show --stat -M --format= HEAD        # restructuring commit
git show -M --name-status --format= HEAD | grep -c '^R'   # rename count
git log --follow --oneline README.md | tail -5
```

**Expected**: 139 `R` (rename) entries for the moved files; zero `A`/`D`
pairs among source files (the only `D` is `leiden/.gitignore`, paired with
the deliberate root `.gitignore` consolidation); `git log --follow` reaches
pre-move commits for a moved file.

## Scenario 5 — Ignore rules at the new location (FR-005)

```sh
cargo build -p leiden-cli
git status --porcelain | grep -E '^\?\? (target|crates)' || echo "FR-005 OK: artifacts ignored"
git check-ignore target crates/leiden/target && echo "ignore patterns active"
```

**Expected**: no untracked build artifacts appear; `git check-ignore` confirms
`/target/` and `/crates/*/target/` match at the root.

## Scenario 6 — Zero stale references (User Story 3, SC-003 — after PR 2)

```sh
grep -rn "working-directory: leiden" .github/ && echo "STALE" || echo "CI clean"
grep -rln "\.\./specs\|\.\./\.specify" README.md && echo "STALE" || echo "README clean"
grep -rn "leiden/leiden" --include='*.md' --include='*.yml' --include='*.toml' . \
  | grep -v specs/00[1-4]- | grep -v graphify-out || echo "SC-003 OK: no live old-prefix references"
```

**Expected**: CI and README clean; no live reference to the old nested prefix
outside archived spec snapshots (`specs/001…004`) and git history.

## Scenario 7 — Fresh-clone equivalence (User Story 1 independent test)

```sh
git clone <repo-url> /tmp/leiden-fresh && cd /tmp/leiden-fresh
cargo check --workspace && cargo nextest run --workspace
```

**Expected**: builds and passes with zero manual path fixes — the strongest
proof the move is complete and self-consistent.
