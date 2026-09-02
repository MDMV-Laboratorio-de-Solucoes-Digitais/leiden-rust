# Contract: TUI Visual Explanation Interface

**Feature**: TUI Visual Explanation (`specs/004-tui-visual-explanation`)  
**Contract Version**: 1.2.0  
**Status**: Ratified  
**Aligned With**: `design-system.md`, `.specify/memory/constitution.md` (v1.1.0)  

---

## 1. UI Layout & Viewport Contract

### 1.1 Two-Stage Layout Splitting

The TUI root rendering window is partitioned vertically in two deterministic stages:

1. **Stage 1 (Root Vertical Split)**:
   - **Main Content Area**: `Constraint::Min(23)` (occupies rows 1 to `Height - 1`)
   - **Status & Playback Bar**: `Constraint::Length(1)` (occupies fixed bottom row `Height`)

2. **Stage 2 (Main Content Area Split)**:
   - **Explanation Panel**: `Constraint::Percentage(35)` (upper sub-area, min 8 rows at $80 \times 24$)
   - **Graph Visualization Canvas**: `Constraint::Percentage(65)` (lower sub-area, min 15 rows at $80 \times 24$)

```text
╭────────────────────────────────────────────────────────────╮
│ ╭─ EXPLANATION PANEL (35% of Main Area) ─────────────────╮ │
│ │  STEP 1 OF 3: FINDING BEST FRIEND CIRCLES              │ │
│ │                                                        │ │
│ │  Each person looks around at their closest friends     │ │
│ │  and moves into the club where they have the most      │ │
│ │  in common.                                            │ │
│ │                                                        │ │
│ │  Phase: Local Moving   Communities: 4   Progress: 45%  │ │
│ ╰────────────────────────────────────────────────────────╯ │
│ ╭─ GRAPH VISUALIZATION (65% of Main Area) ───────────────╮ │
│ │                                                        │ │
│ │          ●───●                  ●───●                  │ │
│ │         /│   │\                /│   │\                 │ │
│ │        ● └───┘ ●              ● └───┘ ●                │ │
│ │          (Club 0)               (Club 1)               │ │
│ │                                                        │ │
│ │  Dataset: [Karate Club] (Active) · 34 nodes · 78 edges │ │
│ ╰────────────────────────────────────────────────────────╯ │
│ ╭─ STATUS & PLAYBACK BAR (Fixed 1 row) ──────────────────╮ │
│ │ ● Playing  [██████░░░░] 60%  Mode: Phase  Space:Pause  │ │
│ ╰────────────────────────────────────────────────────────╯ │
╰────────────────────────────────────────────────────────────╯
```

---

## 2. Keyboard & Playback Contract

### 2.1 Keybinding Matrix

| Key | Action | Scope / Precondition | UX State Transition |
|---|---|---|---|
| `Space` | Toggle Play / Pause auto-stepping | Global | Switches `is_playing` flag; updates status bar badge |
| `n` / `Right Arrow` | Advance one step forward | Paused or Playing | If playing, auto-pauses then advances exactly 1 step |
| `t` | Toggle Granularity Mode | Global | Toggles `PhaseLevel` (`Mode: Phase`) ↔ `MicroStep` (`Mode: Micro`) |
| `1` | Load Preset: Zachary's Karate Club | Global | Auto-pauses, switches graph dataset, resets to Step 1 Initial State |
| `2` | Load Preset: Two Cliques | Global | Auto-pauses, switches graph dataset, resets to Step 1 Initial State |
| `3` | Load Preset: Random Mess | Global | Auto-pauses, switches graph dataset, resets to Step 1 Initial State |
| `r` | Restart explanation run | Global | Resets simulation physics and state machine back to Step 1 |
| `?` | Toggle keybinding help modal | Global | Opens centered $50 \times 14$ help overlay; dismisses with `?`, `Esc`, `Space` |
| `q` / `Ctrl+C` | Clean exit | Global | Restores terminal raw mode & alternate screen buffer immediately |

### 2.2 Preset Switching Lifecycle Policy

1. **Reset Invariant**: Switching presets via keys `1`, `2`, `3` ALWAYS resets the explanation state machine to `Phase::InitialState` (Step 1 of N).
2. **Auto-Pause Policy**: If auto-play is active when a preset key is pressed, playback is automatically paused so the user can inspect the unclustered starting topology.
3. **Active Preset Highlight**: The active preset title is rendered in the canvas footer with an `(Active)` badge styled in `ACCENT_PRIMARY`.
4. **Granularity Mode Preservation**: The user's active `GranularityMode` (`PhaseLevel` vs `MicroStep`) is strictly preserved across preset switches.

---

## 3. Canvas Rendering & Physics Contract

### 3.1 Node & Edge Visual Representation

1. **Nodes**:
   - Rendered using Unicode discs `●` (`U+25CF`) via `ratatui::widgets::canvas::Canvas`.
   - **Initial State**: Monochromatic `FG_2` (`#737a89`).
   - **Active/Completed Clustering**: Categorical color `COMMUNITY_COLORS[comm_id % 12]`.
   - **Node ID Labels**: Displayed adjacent at $(x + 0.015, y)$ if and only if total nodes $N \le 40$. Suppressed when $N > 40$ to avoid clutter.
2. **Edges**:
   - Rendered as continuous lines on the canvas using `ratatui::widgets::canvas::Line`.
   - **Intra-community edges** (same community): Colored with `COMMUNITY_COLORS[comm_id % 12]`.
   - **Inter-community edges** (disjoint communities): Dimmed with `FG_3` (`#565f71`).

### 3.2 Force Simulation Dynamics & Numerical Stability

1. **Virtual Coordinate Space**: All node coordinates are computed in normalized $[0.05, 0.95] \times [0.05, 0.95]$ virtual unit space, clamped to prevent clipping panel borders.
2. **Deterministic Seeding**: Initial unclustered layouts are initialized using a deterministic seed based on node index / CRC32 hash to guarantee 100% reproducible animations.
3. **Soft Collision & Zero-Division Safety**:
   - **Pairwise Repulsion**: $F_{rep}(u, v) = \frac{k_{rep}}{\max(d(u, v)^2, \epsilon^2)}$ with softening factor $\epsilon = 0.03$.
   - **Zero-Division Guarded Separation**: $d_{min} = 0.04$. If Euclidean distance $d(u, v) < d_{min}$, a separation displacement vector $\vec{\delta} = \frac{\vec{u} - \vec{v}}{\max(d(u,v), \epsilon)} \cdot (d_{min} - d) \times 0.5$ is applied, strictly preventing $0/0 \to \text{NaN}$.
4. **Convergence Damping**: Velocity damping factor is fixed at $\alpha = 0.85$, with a maximum tick budget of 25 relaxation steps per phase jump.
5. **Zero Allocation Physics Tick**: All spatial node structures and velocity vectors are pre-allocated at initialization in flat pre-sized buffers; `tick()` executes with zero heap reallocations.
6. **Dataset Scale & Graceful Degradation Bounds**:
   - Standard 2D force relaxation is active for graphs up to $N \le 200$ nodes and $E \le 1000$ edges.
   - For graphs with $N > 200$, the layout transitions to a simplified radial cluster centroid projection to preserve $\le 16\text{ms}$ frame render times.

---

## 4. Minimum Dimension, Resize & Signal Cleanup Contract

### 4.1 Viewport Dimension Guard

If `TerminalDimensionGuard::is_valid(width, height)` evaluates to `false` ($< 80 \times 24$):

1. Normal widget layout and physics ticks are immediately suspended (`is_paused_by_resize = true`).
2. The UI renders a centered $46 \times 7$ modal dialog blocking all non-exit interaction:
   ```text
   ╭─ TERMINAL TOO SMALL ──────────────────────╮
   │ Current size: {width} × {height}          │
   │ Minimum required: 80 × 24                 │
   │ Please resize your terminal window.       │
   ╰───────────────────────────────────────────╯
   ```

### 4.2 Restoration & CPU Throttling Policy

1. **Restoration**: When terminal geometry is expanded back to $\ge 80 \times 24$, coordinates re-normalize instantly to the new canvas `Rect` without loss of physics state or playback progress.
2. **CPU Throttling**: When playback is paused or the TUI is idle, physics simulation ticks are halted immediately, and the event poll blocks for up to 200ms, maintaining CPU utilization $< 0.1\%$.
3. **Signal & Panic Cleanup**: A dedicated panic hook and signal handler for `SIGINT`, `SIGTERM`, and `SIGHUP` guarantees invocation of `crossterm::terminal::disable_raw_mode()` and `crossterm::execute!(stdout(), LeaveAlternateScreen, Show)`.

---

## 5. Design System Token Mapping & Terminal Compatibility

### 5.1 Color Token Table & Contrast Metrics

| Token Name | Hex Code | Contrast on `BG_0` | Contrast on `BG_1` | ANSI 256 Fallback | Semantic UI Role |
|---|---|---|---|---|---|
| `BG_0` | `#1a1b26` | Base | — | 234 | Root canvas background |
| `BG_1` | `#1f202e` | — | Base | 235 | Panel interior background |
| `FG_0` | `#c0caf5` | 11.1:1 (AAA) | 10.3:1 (AAA) | 189 | Step Headlines, Modal titles |
| `FG_1` | `#a9b1d6` | 8.1:1 (AAA) | 7.4:1 (AAA) | 146 | Body analogy text, active labels |
| `FG_2` | `#7d8594` | 4.8:1 (AA) | 4.5:1 (AA) | 243 | Muted metadata, unassigned nodes |
| `FG_3` | `#565f71` | 3.1:1 (Dim) | 2.8:1 (Dim) | 240 | Unfocused borders, inter-community edges |
| `ACCENT_PRIMARY` | `#7aa2f7` | 6.8:1 (AAA) | 6.3:1 (AA) | 111 | Focused block borders, active preset badge |
| `ACCENT_INFO` | `#73daca` | 10.4:1 (AAA) | 9.6:1 (AAA) | 116 | Live stat badges, `Mode: Micro` label |
| `ACCENT_SUCCESS` | `#9ece6a` | 8.9:1 (AAA) | 8.2:1 (AAA) | 150 | Completion indicator (`✔ Finished`) |
| `ACCENT_WARNING` | `#e0af68` | 9.5:1 (AAA) | 8.8:1 (AAA) | 179 | Paused state badge, resize warning title |
| `COMMUNITY_COLORS` | 12 CIELAB | $\ge 4.5:1$ (AA) | $\ge 4.5:1$ (AA) | 12-color table | 12 distinct colors (pairwise $\Delta E^* \ge 25.0$) |

### 5.2 Terminal Emulator Compatibility Discipline

1. **No `Modifier::ITALIC`**: Italic styling is strictly prohibited across all widgets to prevent reverse-video or blinking artifacts on legacy Linux TTYs and tmux sessions.
2. **Rounded Borders & ASCII Fallback**: All panel borders default to `BorderType::Rounded` (`╭╮╰╯`). In non-UTF-8 terminal environments (`LANG=C` or ASCII), rendering automatically falls back to `BorderType::Plain` (`+--+`, `|  |`).
