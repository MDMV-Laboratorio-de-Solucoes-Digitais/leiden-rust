# Git Hooks & Lefthook Research for Rust Projects

## 1. "On-Commit" Hook Terminology and Execution Order

### Correct Hook Name

The correct hook name for running checks **before a commit is finalized** is **`pre-commit`**. There is no git hook called "on-commit."

### Git Commit Hook Execution Order

When `git commit` is invoked, hooks run in this order:

| Order | Hook | Bypassed by `--no-verify` | Can Modify Tree | Can Modify Message | Can Abort Commit |
|-------|------|---------------------------|-----------------|--------------------|------------------|
| 1 | `pre-commit` | Yes | Yes (via `git add`) | No | Yes (non-zero exit) |
| 2 | `prepare-commit-msg` | **No** | Yes (via `git add`)* | Yes (edit message file) | Yes (non-zero exit) |
| 3 | `commit-msg` | Yes | No | Yes (edit message file) | Yes (non-zero exit) |
| 4 | `post-commit` | No | No | No | No (already committed) |

**Key details:**

- **`pre-commit`** — Invoked by `git-commit` before obtaining the proposed commit log message. Exiting with non-zero status causes `git commit` to abort before creating a commit. This is the primary hook for running checks (linting, formatting, testing). [Source: https://git-scm.com/docs/githooks#_pre_commit](https://git-scm.com/docs/githooks#_pre_commit)

- **`prepare-commit-msg`** — Invoked right after preparing the default log message, and before the editor is started. **Not suppressed by `--no-verify`.** The git documentation explicitly states: "it should not be used as a replacement for the pre-commit hook." [Source: https://git-scm.com/docs/githooks#_prepare_commit_msg](https://git-scm.com/docs/githooks#_prepare_commit_msg)

- **`commit-msg`** — Invoked after the message is finalized. Can be bypassed with `--no-verify`. Allowed to edit the message file in place. [Source: https://git-scm.com/docs/githooks#_commit_msg](https://git-scm.com/docs/githooks#_commit_msg)

- **`post-commit`** — Invoked after a commit is made. "This hook is meant primarily for notification, and cannot affect the outcome of `git commit`." [Source: https://git-scm.com/docs/githooks#_post_commit](https://git-scm.com/docs/githooks#_post_commit)

*See Section 2 for the nuanced answer on whether `prepare-commit-msg` can modify the tree.

### Recommendation

Use **`pre-commit`** as the primary hook for checks. It is the idiomatic, well-understood hook for aborting a commit when checks fail. Do not use `prepare-commit-msg` as a replacement.

---

## 2. `prepare-commit-msg` Limitation: Can It Modify the Commit Tree?

### Answer: Technically Yes, But With Critical Caveats

**The claim that "Git creates the commit's tree object BEFORE `prepare-commit-msg` runs" is FALSE for the normal `git commit` flow.**

Analyzing git's source code (`builtin/commit.c`):
- `prepare-commit-msg` hook runs at approximately line 1116-1118
- `write_index_as_tree()` (which creates the tree object from the index) runs at line 1706

Since line 1706 comes after line 1116, the tree is written **AFTER** `prepare-commit-msg` completes. This means `git add` during `prepare-commit-msg` **can** modify the index, and those changes **will** be included in the current commit's tree. [Source: https://deepwiki.com/git/git/3.1-commit-creation](https://deepwiki.com/git/git/3.1-commit-creation)

### Why This Is Still Problematic

Despite the technical ability to modify the tree, the git documentation explicitly warns:

> "The purpose of the hook is to edit the message file in place, and it is not suppressed by the `--no-verify` option. [...] It should not be used as a replacement for the pre-commit hook."
> — [https://git-scm.com/docs/githooks#_prepare_commit_msg](https://git-scm.com/docs/githooks#_prepare_commit_msg)

### The `--no-verify` Bypass Scenario

The `prepare-commit-msg` hook **IS** invoked even when `--no-verify` is used. This makes it a potential non-bypassable backstop. However:

1. **It can abort the commit** (non-zero exit status causes `git commit` to abort)
2. **It can modify the commit message** (e.g., append a "bypassed" footer)
3. **It can re-stage files** (via `git add`), and those files WILL be in the current commit

The [`precommit-verify`](https://lib.rs/crates/precommit-verify) Rust crate leverages this exact behavior: it uses `prepare-commit-msg` to append a `Verified: precommit-verify ✕` footer to the commit message when `--no-verify` bypasses the pre-commit checks, making the bypass visible in `git log`. [Source: https://lib.rs/crates/precommit-verify](https://lib.rs/crates/precommit-verify)

### Recommendation

- **Do NOT** use `prepare-commit-msg` as a replacement for `pre-commit` for running checks. The git maintainers explicitly discourage this.
- **DO** consider using `prepare-commit-msg` as a **detection mechanism** — it can record in the commit message whether checks were actually run, making `--no-verify` bypasses visible in the commit history.
- For a non-bypassable formatting backstop, the correct approach is to use `pre-commit` with `stage_fixed: true` (see Section 5) and enforce hooks via CI/server-side hooks, since `--no-verify` can always bypass client-side `pre-commit`.

---

## 3. Pre-Push Hook Best Practices for Rust Projects

### What to Run in `pre-push`

The `pre-push` hook is called by `git-push` and can prevent a push from taking place. It is the appropriate place for checks that are too slow for `pre-commit` but should run before code reaches the remote. [Source: https://git-scm.com/docs/githooks#_pre_push](https://git-scm.com/docs/githooks#_pre_push)

**Recommended checks for a Rust project:**

| Check | Command | Why It Belongs in pre-push |
|-------|---------|---------------------------|
| Full test suite | `cargo test` (including integration tests) | Slow; can take minutes on large projects |
| Clippy (strict) | `cargo clippy -- -D warnings` | Catches lints that might be too noisy for pre-commit |
| Cargo deny | `cargo deny check` | License/security auditing of dependencies |
| Documentation build | `cargo doc --no-deps` | Verifies docs compile without errors |
| Build (locked) | `cargo build --locked` | Ensures Cargo.lock is up to date |
| Formatting (all files) | `cargo fmt -- --check` | Can be done in pre-commit, but pre-push catches everything |

**Source for Rust best practices:** [https://rust.codeguides.io/git/best-practices/](https://rust.codeguides.io/git/best-practices/) recommends: "Pre-commit mirrors CI: `cargo fmt`, `clippy -D warnings`, fast `cargo test`." The pre-push hook extends this to slower checks.

### What Should Stay in `pre-commit` (Not pre-push)

- `cargo fmt -- --check` (fast, should block commits)
- Fast unit tests (`cargo test --lib`)
- File-level linting with `clippy` on changed files only

### Recommendation

Use `pre-push` for the full test suite, `cargo deny`, and `cargo doc`. These are checks that would slow down the commit cycle too much but should pass before code is shared with the team. Leverage lefthook's `parallel: true` to run these checks concurrently.

---

## 4. Other DX-Improving Hooks for Rust Projects

### `commit-msg`

**What it runs:** Validates commit message format (e.g., Conventional Commits, ticket ID presence).

**Why:** Enforces commit message conventions. For example, requires messages like `feat(api): add health endpoint [API-42]`.

**Source:** [https://git-scm.com/docs/githooks#_commit_msg](https://git-scm.com/docs/githooks#_commit_msg) — "The hook is allowed to edit the message file in place, and can be used to normalize the message into some project standard format."

**Lefthook example:**
```yaml
commit-msg:
  commands:
    lint:
      run: 'test $(grep -c "^Signed-off-by: " {1}) -lt 2'
```

### `post-commit`

**What it runs:** Notification triggers, CI pipeline kicks, or automatic changelog updates.

**Why:** "This hook is meant primarily for notification, and cannot affect the outcome of `git commit`." [Source: [https://git-scm.com/docs/githooks#_post_commit](https://git-scm.com/docs/githooks#_post_commit)]

**Use cases:**
- Trigger a build notification
- Update a local task tracker
- Run `cargo build --release` in the background for faster subsequent builds

### `post-merge`

**What it runs:** `cargo build`, dependency checks, or regeneration of build artifacts.

**Why:** "This hook is invoked by `git-merge`, which happens when a `git pull` is done on a local repository." [Source: [https://git-scm.com/docs/githooks#_post_merge](https://git-scm.com/docs/githooks#_post_merge)]

**Use cases for Rust:**
- Automatically run `cargo build` after pulling to pre-populate the build cache
- Check if `Cargo.lock` changed and notify the developer
- Re-run code generation (e.g., `build.rs` outputs)

### `post-checkout`

**What it runs:** `cargo build`, environment validation.

**Why:** "This hook can be used to perform repository validity checks, auto-display differences from the previous HEAD if different, or set working dir metadata properties." [Source: [https://git-scm.com/docs/githooks#_post_checkout](https://git-scm.com/docs/githooks#_post_checkout)]

**Use cases for Rust:**
- Warn if the branch change requires a different Rust toolchain (check `rust-toolchain.toml`)
- Pre-warm the build cache for the new branch

### `pre-rebase`

**What it runs:** Validation that the rebase is safe (e.g., not rebasing shared branches).

**Why:** "This hook is called by `git-rebase` and can be used to prevent a branch from getting rebased." [Source: [https://git-scm.com/docs/githooks#_pre_rebase](https://git-scm.com/docs/githooks#_pre_rebase)]

**Use cases:**
- Prevent rebasing commits that have already been pushed
- Warn when rebasing across `Cargo.lock` conflict boundaries

### `pre-applypatch` / `applypatch-msg`

**What it runs:** Validates patches applied via `git am`.

**Why:** Useful for projects that accept patches via email. [Source: [https://git-scm.com/docs/githooks#_applypatch_msg](https://git-scm.com/docs/githooks#_applypatch_msg)]

### Recommendation

For a Rust project, the highest-value hooks after `pre-commit` and `pre-push` are:
1. **`commit-msg`** — enforce Conventional Commits or ticket ID format
2. **`post-merge`** — auto-build after pull to keep the cache warm
3. **`pre-rebase`** — prevent dangerous rebases of shared history

---

## 5. Lefthook-Specific Features to Leverage

### `stage_fixed`

**What it does:** When set to `true`, lefthook automatically calls `git add` on files after running the command. Works **only** for the `pre-commit` hook. [Source: https://lefthook.dev/configuration/stage_fixed/](https://lefthook.dev/configuration/stage_fixed/)

**Use case:** Auto-formatters (e.g., `cargo fmt`) that modify files during pre-commit. The fixed files are automatically re-staged.

```yaml
pre-commit:
  commands:
    fmt:
      run: cargo fmt -- --check
      stage_fixed: true
```

**Important:** "If the `git add` call fails, the hook fails too. Otherwise the commit would silently go with the unfixed content."

### `parallel`

**What it does:** Runs commands and scripts concurrently. Default is `false` (sequential). [Source: https://lefthook.dev/configuration/parallel/](https://lefthook.dev/configuration/parallel/)

```yaml
pre-push:
  parallel: true
  commands:
    test:
      run: cargo test
    clippy:
      run: cargo clippy -- -D warnings
    doc:
      run: cargo doc --no-deps
```

### `skip`

**What it does:** Skips commands conditionally. Supports: `rebase`, `merge`, `merge-commit`, `ref: <branch>`, `run: <command>`. [Source: https://lefthook.dev/configuration/skip/](https://lefthook.dev/configuration/skip/)

```yaml
pre-commit:
  commands:
    test:
      skip:
        - merge
        - rebase
      run: cargo test
```

**Skip on a specific branch:**
```yaml
pre-commit:
  skip:
    - ref: main
```

**Skip using environment variable check:**
```yaml
pre-commit:
  skip:
    - run: test "${NO_HOOK}" -eq 1
```

### `only`

**What it does:** The opposite of `skip` — only execute when conditions are met. `skip` takes precedence over `only`. [Source: https://lefthook.dev/configuration/only/](https://lefthook.dev/configuration/only/)

```yaml
pre-commit:
  commands:
    lint-on-rebase:
      only: rebase
      run: cargo check
```

### `files` Filtering

**What it does:** Custom command that returns files to be referenced in `{files}` template. If the result is empty, execution is skipped. Can be set at hook-level or job-level. [Source: https://lefthook.dev/configuration/files/](https://lefthook.dev/configuration/files/) and [https://lefthook.dev/configuration/files-global/](https://lefthook.dev/configuration/files-global/)

```yaml
pre-commit:
  files: git diff --name-only master
  commands:
    lint:
      run: cargo clippy -- {files}
```

### File Templates in `run`

Available placeholders:
- `{files}` — custom `files` command result
- `{staged_files}` — staged files being committed
- `{push_files}` — files committed but not pushed
- `{all_files}` — all files tracked by git
- `{cmd}` — shorthand for the command from `lefthook.yml`
- `{0}` — all git hook arguments
- `{1}`, `{2}`, etc. — individual git hook arguments

[Source: https://lefthook.dev/configuration/run/](https://lefthook.dev/configuration/run/)

### `glob` and `exclude`

**What they do:** Filter files by glob pattern or exclude specific patterns. [Source: https://lefthook.dev/configuration/glob/](https://lefthook.dev/configuration/glob/) and [https://lefthook.dev/configuration/exclude/](https://lefthook.dev/configuration/exclude/)

```yaml
pre-commit:
  commands:
    clippy:
      glob: "*.rs"
      exclude:
        - "target/**"
        - "generated/**"
      run: cargo clippy -- {staged_files}
```

### `piped`

**What it does:** Stops running commands if one of them fails. Cannot be combined with `parallel: true`. [Source: https://lefthook.dev/configuration/piped/](https://lefthook.dev/configuration/piped/)

### `interactive`

**What it does:** Allows commands to receive stdin from the terminal. Useful for commands that prompt the user. [Source: https://lefthook.dev/configuration/interactive/](https://lefthook.dev/configuration/interactive/)

### `env`

**What it does:** Sets environment variables for commands. Useful for extending `$PATH` in GUI environments. [Source: https://lefthook.dev/configuration/env/](https://lefthook.dev/configuration/env/)

### `no_tty`

**What it does:** Hides spinner and interactive output. [Source: https://lefthook.dev/configuration/no_tty/](https://lefthook.dev/configuration/no_tty/)

### Handling the `--no-verify` Bypass

**How lefthook handles `--no-verify`:**

Git's `--no-verify` flag bypasses the `pre-commit` and `commit-msg` hooks at the git level — lefthook never gets invoked for those hooks. However:

1. **`prepare-commit-msg` is NOT bypassed** by `--no-verify`. Lefthook will still run `prepare-commit-msg` commands even when `--no-verify` is used. [Source: https://git-scm.com/docs/githooks#_prepare_commit_msg](https://git-scm.com/docs/githooks#_prepare_commit_msg)

2. **`LEFTHOOK=0` environment variable** — Lefthook can be skipped entirely by setting `LEFTHOOK=0`:
   ```
   LEFTHOOK=0 git commit
   ```
   [Source: https://lefthook.dev/usage/](https://lefthook.dev/usage/)

3. **Per-command `skip`** — Individual commands can be skipped via the `skip` option (see above).

**Recommendation for preventing bypasses:**

Since `--no-verify` is a client-side control, it can always be bypassed by a determined developer. To make checks truly non-bypassable:
- Use **server-side hooks** (e.g., `pre-receive` on the remote) to reject pushes that don't meet standards
- Use **CI/CD enforcement** — run the same checks in CI and block merging on failure
- Use `prepare-commit-msg` as a **detection mechanism** (not prevention) — append a footer indicating checks were bypassed, as done by [`precommit-verify`](https://lib.rs/crates/precommit-verify)

---

## Summary of Recommendations

| Hook | What to Run | Why |
|------|-------------|-----|
| `pre-commit` | `cargo fmt -- --check`, `cargo clippy` (changed files), fast unit tests | Fast feedback, auto-fix with `stage_fixed` |
| `prepare-commit-msg` | Detection footer (e.g., `precommit-verify`) | Non-bypassable visibility into `--no-verify` usage |
| `commit-msg` | Conventional commit / ticket ID validation | Enforce message standards |
| `pre-push` | Full `cargo test`, `cargo deny check`, `cargo doc` | Slow checks before sharing code |
| `post-merge` | `cargo build` (cache warming) | Keep build cache fresh after pull |
| `pre-rebase` | Safety checks | Prevent dangerous rebases of shared history |

**Lefthook features to leverage:**
- `parallel: true` on `pre-push` for concurrent test/doc/clippy runs
- `stage_fixed: true` on `pre-commit` for auto-formatting
- `skip: [merge, rebase]` on slow commands that are irrelevant during merge/rebase
- `glob: "*.rs"` to limit Rust-specific commands to Rust files
- `piped: true` for sequential setup hooks where later steps depend on earlier ones
