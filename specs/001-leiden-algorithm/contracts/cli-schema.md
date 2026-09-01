# CLI Schema Contract: `leiden` and `leiden-tui`

**Branch**: `[001-leiden-algorithm]` | **Date**: 2026-08-30

This document defines the input and output schemas for both binaries. It is the
authoritative reference for shell-pipeline integration.

---

## 1. `leiden` (non-interactive)

### 1.1 Synopsis

```text
leiden [OPTIONS] [<GRAPH_FILE>]
```

### 1.2 Arguments

| Flag | Type | Default | Description |
|---|---|---|---|
| `[<GRAPH_FILE>]` | path (positional) | `-` (when piped) | Path to the input graph file (edge-list or JSON), or `-` to read from stdin. Defaults to stdin if omitted when standard input is not a TTY. |
| `--gamma <F>` | f64 | `1.0` | Resolution parameter γ. Rejected if ≤ 0. |
| `--seed <U>` | u64 | `0` | Randomness seed for stochastic refinement. v1 is deterministic; the seed is accepted for forward compatibility. |
| `--iteration-cap <N>` | u32 | `10` | Maximum outer-loop iterations. Rejected if `< 1`. |
| `--format <FMT>` | enum | `json` | Output format: `json` or `text`. |
| `--log-file <PATH>` | path | (none) | If set, write structured tracing events to this file. |
| `--log-level <LVL>` | enum | `info` | Tracing level: `trace`, `debug`, `info`, `warn`, `error`. |
| `-h`, `--help` | flag | — | Print help and exit 0. |
| `-V`, `--version` | flag | — | Print version and exit 0. |

### 1.3 Input Formats

#### 1.3.1 Edge-list (primary)

Plain-text, one edge per line:

```text
# optional header
# nodes=<N>
<source_id><sep><target_id><sep><weight>
```

- `<sep>` is tab or comma. Tabs are the default; comma is auto-detected from
  the first non-header, non-comment line if no tab is found.
- `<weight>` is `f64`. Negative or non-finite weights are rejected with a
  line-referencing error.
- Lines beginning with `#` are comments and ignored.
- The optional `# nodes=<N>` header serves as an optional memory allocation hint;
  it is not a strict validation assertion and does not trigger errors if omitted or
  if the actual unique node count differs.
- Self-loops (`source == target`) are rejected.
- Empty lines are ignored.

#### 1.3.2 JSON adjacency (optional)

Accepted when the file extension is `.json` OR the first non-whitespace byte
is `{`:

**Dispatch precedence (authoritative; matches `spec.md` FR-011 and is locked by `tasks.md` T074a `dispatch_extension_wins_over_byte_sniff`)**:

1. **File extension wins first**. If the file extension is `.json`, the JSON adjacency parser is used unconditionally — even if the file's first non-whitespace byte is not `{`.
2. **Byte sniff only when extension is ambiguous**. When the extension is not `.json`, the first non-whitespace byte is sniffed: `{` selects the JSON adjacency parser.
3. **Default**. Otherwise, the edge-list parser is used.

Edge-list is the primary format; JSON adjacency is accepted only on explicit extension hint or unambiguous byte sniff. Tests: T074 (`dispatch_precedence_matches_cli_schema`), T074a (`dispatch_extension_wins_over_byte_sniff`).

```json
{
  "nodes": ["a", "b", "c", "d"],
  "edges": [["a", "b"], ["c", "d"], ["a", "c"]],
  "weights": [1.0, 1.0, 0.5]
}
```

- `weights` is optional; defaults to `1.0` per edge.
- Dangling node ids (in `edges` but not `nodes`) are rejected.
- `nodes` may be empty if no edges exist.

### 1.4 Output Formats

#### 1.4.1 JSON (default)

Written to **stdout**:

```json
{
  "gamma": 1.0,
  "seed": 0,
  "iterations": 3,
  "termination_reason": "converged",
  "quality": 0.4127,
  "threading": "SingleThreaded",
  "assignments": [
    {"node": "a", "community": 0},
    {"node": "b", "community": 0},
    {"node": "c", "community": 1},
    {"node": "d", "community": 1}
  ]
}
```

`termination_reason` is one of `"converged"`, `"iteration_cap"`, `"degenerate_input"`.
`assignments` is sorted by `node`.

#### 1.4.2 Text (`--format text`)

Written to **stdout**, tab-separated, sorted by node id:

```text
a	0
b	0
c	1
d	1
```

Other format strings are rejected with exit code 2 and a stderr message:
`unsupported output format '<x>'; expected 'json' or 'text'`.

### 1.5 Diagnostics (stderr)

- Progress and error lines emitted to **stderr** (stdout is reserved for the partition output). Authoritative examples are in §1.5.1; the canonical regex is `^(loaded graph|iteration <N>|terminated after <N> iterations|<error-class>: <message>)$` (anchored at start-of-line; the portion after the `:` separator in the `<error-class>: <message>` form is free-form text). Stdout and stderr are independent streams; consumers MUST read stdout only.
- A single progress line at start: `loaded graph: nodes=<n> edges=<m> total_weight=<w>`.
- One line per iteration: `iteration <i>: quality=<q>` (the `quality=<q>` suffix is the documented format; the regex in the §1.5.1 anchor below permits it).
- Final line: `terminated after <k> iterations: <reason>` where `<reason>` is `converged`, `iteration_cap`, or `degenerate_input`.
- Errors: `<error-class>: <message>` for parse errors (e.g. `malformed: <path>:<line>: invalid weight ...`, `io: <path>: <cause>`); no panic trace. When reading from standard input via stdin or `-`, `<path>` is `<stdin>`. Self-loops are reported as `malformed: <path>:<line>: self-loop on node '<id>': not permitted`, with the line number embedded in the `LeidenError::SelfLoop { line: Some(N), node }` payload. The library's `CsrGraph::from_edges` emits the same variant with `line: None` (no source line available) — see `data-model.md §1.11` and `spec.md` FR-008.

#### 1.5.1 Canonical Examples (authoritative)

The following stderr blocks are the canonical traces against which T094a asserts the regex. The authoritative regex permitting the optional `: quality=<q>` suffix on `iteration <N>` lines is `^(loaded graph|iteration \d+(: quality=-?\d+(\.\d+)?)?|terminated after \d+ iterations(: (converged|iteration_cap|degenerate_input))?|(malformed|io):.*)$`. Implementations MUST emit exactly these lines for the corresponding scenarios; deviations require a coordinated update of this section, T094a, and the implementation task.

**Successful run (`leiden --gamma 1.0 fixtures/karate.edg`):**
```
loaded graph: nodes=34 edges=78 total_weight=156.0
iteration 1: quality=0.4198
iteration 2: quality=0.4231
terminated after 2 iterations: converged
```

**Malformed input (negative weight on line 7 of `bad.edg`):**
```
loaded graph: nodes=10 edges=9 total_weight=12.0
malformed: bad.edg:7: invalid weight `-1.0`: must be finite and ≥ 0
```

Note: the `loaded graph:` line shown above is illustrative of an earlier successful load; on a parse failure mid-stream, the CLI does NOT emit a `loaded graph:` line — instead the first stderr line is the parse error itself (per `tasks.md` T086: self-loops and similar are validated at the parser boundary BEFORE `CsrGraph::from_edges` runs, so a malformed file emits no `loaded graph:` line).

**I/O error (missing path):**
```
io: fixtures/__missing__.edg: No such file or directory (os error 2)
```

Each line MUST match the §1.5 regex anchored at start-of-line; no panic-trace substrings (`panicked at`, `thread 'main'`) may appear on stderr in any scenario.

### 1.6 Exit Codes & Error Remediation Guidance

| Code | Error Class | Trigger | Remediation Guidance |
|---|---|---|---|
| `0` | Success | Normal execution | Partition output written to stdout. |
| `2` | Unsupported output format | Unknown format string in `--format` | Use `--format json` (default) or `--format text`. |
| `3` | Parameter validation | `gamma <= 0.0`, NaN, or `iteration-cap < 1` | Ensure `--gamma > 0.0` and `--iteration-cap >= 1`. |
| `4` | `ParseFieldCount` | Line has `< 2` or `> 3` columns | Check edge-list format: `<source><sep><target>[<sep><weight>]`. |
| `4` | `InvalidWeight` | Negative or non-finite weight value | Ensure all edge weights are finite and `>= 0.0`. |
| `4` | `SelfLoop` | Edge where `source == target` | Remove self-loops from the graph dataset before running Leiden. |
| `4` | `DanglingNode` | JSON edge references node missing from `nodes` list | Ensure all endpoints in `edges` are declared in `nodes`. |
| `4` | `EmptyGraph` | Input contains 0 nodes or 0 edges | Ensure graph file contains at least one node and edge. |
| `5` | `Io` | File not found, permission denied, or unreadable stream | Verify file path, read permissions, or pipe source. |
| `1` | `Internal` | Unexpected internal invariant violation | Report bug with reproduction graph. |

### 1.7 Example

```sh
$ leiden --gamma 1.0 --format text fixtures/two_cliques.edg
a	0
b	0
c	0
d	0
e	1
f	1
g	1
h	1
i	1
```

---

## 2. `leiden-tui` (interactive)

### 2.1 Synopsis

```text
leiden-tui [OPTIONS] [<GRAPH_FILE>]
```

If `<GRAPH_FILE>` is omitted, the TUI starts in `Idle` state and prompts the
user to choose a fixture from a built-in list or supply a path.

### 2.2 Arguments

| Flag | Type | Default | Description |
|---|---|---|---|
| `[<GRAPH_FILE>]` | path | — | Optional starting graph. |
| `--gamma <F>` | f64 | `1.0` | Initial γ. Editable in-app. |
| `--seed <U>` | u64 | `0` | Initial seed. Editable in-app. |
| `--iteration-cap <N>` | u32 | `10` | Initial cap. Editable in-app. |
| `--log-file <PATH>` | path | (none) | File for structured tracing. |
| `--log-level <LVL>` | enum | `info` | Tracing level. |

### 2.3 Key Bindings

| Key | Action |
|---|---|
| `q` / `Ctrl+C` | Quit |
| `r` | Restart with current parameters |
| `s` | Step (single iteration in paused mode) |
| `p` | Pause / resume auto-iteration |
| `g` | Toggle graph panel |
| `l` | Toggle log panel |
| `Tab` | Move focus between panels |
| `↑` / `↓` | Select community in the community panel |
| `?` | Toggle help overlay |

### 2.4 Panels

| Panel | Purpose |
|---|---|
| **Community list** | One row per community: id, size, internal-edge weight, total-degree. Sorted by size descending. |
| **Graph view** | Node–edge layout (BFS from highest-degree node); communities coloured by hash. |
| **Log pane** | Last 500 `tracing` events, colour-coded by level. |
| **Status bar** | Current state, iteration count, last quality, γ, seed. |

### 2.5 State Transitions (TUI)

```text
Idle ── r / file loaded ──► Running
Running ── converged/cap ──► Done
Running ── error ──► Error
Done ── r ──► Running (restart)
Error ── r ──► Idle
```

---

## 3. Pipeline Examples

```sh
# Detect communities and pretty-print JSON
leiden --gamma 1.5 graph.edg | jq '.quality, .termination_reason'

# Pipe from a generator
python3 gen_lfr.py --nodes 200 | leiden --format text > communities.tsv

# CI: assert a quality threshold
COMM=$(leiden graph.edg | jq '.quality')
test "$(echo "$COMM > 0.3" | bc)" = "1"
```