# Contract: Repository Layout (Old → New File Map)

**Feature**: 005-workspace-root-move

This is the authoritative mapping every contributor, agent, and tool must
assume after PR 1. "Old" paths are relative to the repository root before the
change; "new" paths are the locations after it.

## Moved (139 tracked files — all via `git mv`, content unchanged)

| Old path | New path | Notes |
|---|---|---|
| `leiden/Cargo.toml` | `Cargo.toml` | workspace manifest; **zero content edits** (members stay `crates/*`) |
| `leiden/Cargo.lock` | `Cargo.lock` | zero content edits |
| `leiden/rust-toolchain.toml` | `rust-toolchain.toml` | zero content edits |
| `leiden/clippy.toml` | `clippy.toml` | zero content edits |
| `leiden/deny.toml` | `deny.toml` | zero content edits |
| `leiden/proptest.toml` | `proptest.toml` | zero content edits |
| `leiden/README.md` | `README.md` | content edits deferred to PR 2 (`../specs`, `../.specify` links) |
| `leiden/crates/leiden/**` | `crates/leiden/**` | library crate, unchanged internals |
| `leiden/crates/leiden-cli/**` | `crates/leiden-cli/**` | CLI crate, unchanged internals |
| `leiden/crates/leiden-tui/**` | `crates/leiden-tui/**` | TUI crate, unchanged internals |
| `leiden/fixtures/**` (27 files) | `fixtures/**` | unchanged |

## Edited in PR 1 (minimal functional fixes — nothing else)

| File | Change |
|---|---|
| `.github/workflows/ci.yml` | delete the six `working-directory: leiden` lines |
| `.gitignore` (root) | keep as-is (already supersedes the nested copy; pending `graphify-out/`/`.opencode/` additions ship here) |
| `leiden/.gitignore` | **deleted** (`git rm`) — strict subset of the root copy |
| `crates/leiden/tests/observability_checklist.rs` | one line: `join("../specs/001-leiden-algorithm/checklists/observability.md")` → `join("specs/001-leiden-algorithm/checklists/observability.md")` |

## Untouched (stay at root)

`AGENTS.md`, `design-system.md` (links fixed in PR 2),
`guide-to-strict-rust.md`, `rust-code-rigor.md`, `specs/`, `.specify/`,
`.github/` (apart from ci.yml above), `graphify-out/` (gitignored, regenerated post-move).

## Removed (untracked, local cleanup — no git content)

`leiden/target/` (disposable build cache), `leiden/.commandcode/`
(session-local config), then the empty `leiden/` directory itself (FR-008).

## Post-move invariants

1. `cargo metadata --no-deps` from the root resolves exactly the three members.
2. Every command in [ci-and-commands.md](./ci-and-commands.md) runs from the root with no `cd` (SC-001).
3. `test ! -d leiden` succeeds at the repository root.
4. `git log --follow <any moved file>` reaches pre-move history (FR-002).
