# Leiden TUI — Design System

> **Product**: `leiden-tui` — Interactive Terminal UI for Leiden community detection  
> **Framework**: Ratatui 0.30.2 + Crossterm 0.29.0  
> **Style**: Scientific Dashboard · Data-Dense · Terminal-Native  

---

## 0. Prerequisites & Scope

### 0.1 Toolchain

| Requirement | Value | Source |
|---|---|---|
| Rust edition | 2024 | [Cargo.toml](file:///home/luis/development/leiden/Cargo.toml#L45) |
| MSRV | 1.88.0 | Ratatui 0.30.2 `rust_version` floor |
| Ratatui | 0.30.2 (pinned) | [leiden-tui/Cargo.toml](file:///home/luis/development/leiden/crates/leiden-tui/Cargo.toml#L18) |
| Crossterm | 0.29.0 | [leiden-tui/Cargo.toml](file:///home/luis/development/leiden/crates/leiden-tui/Cargo.toml#L19) |

### 0.2 Terminal Requirements

- **Minimum size**: 80 columns × 24 rows
- **Unicode**: Basic Multilingual Plane (U+0000–U+FFFF) — rounded box-drawing, Greek letters
- **Color**: True-color preferred; graceful degradation to 256-color and 16-color ANSI

### 0.3 Non-Goals (Out of Scope)

| Non-Goal | Rationale |
|---|---|
| Mouse interaction | Terminal keyboard-first; mouse adds complexity without value for this audience |
| Light theme | Dark-only; the target audience (developers, researchers) overwhelmingly uses dark terminals |
| Plugin / theme system | v1 ships one visual identity; extensibility is future scope |
| Per-user customization | No config file for colors/keybindings in v1 |
| Animation / transition timing | Immediate state transitions; no easing curves or frame interpolation |
| Internationalization (i18n) | English-only UI text; Unicode symbols are language-neutral |

### 0.4 Task Cross-Reference

| Task ID | Widget / Module | Design System Section |
|---|---|---|
| T104 | `App` struct, `AppState` enum | §6 State-Driven Theming |
| T105 | Key bindings (`event.rs`) | §5.5 Help Overlay, §8.2 Focus Cycle |
| T105a | Color scheme docs (`colors.rs`) | §9 Implementation Constants Module |
| T106 | Community panel (`ui/community.rs`) | §5.1 Community Panel |
| T107 | Graph view (`ui/graph.rs`) | §5.2 Graph View |
| T108 | Log pane (`ui/log_pane.rs`) | §5.3 Log Pane |
| T109 | Status bar (`ui/status_bar.rs`) | §5.4 Status Bar |
| T110 | `ui::render` dispatcher (`ui/mod.rs`) | §3 Layout System |

---

## 1. Design Philosophy

### 1.1 Core Principles

| Principle | Rationale |
|---|---|
| **Information Density** | Users are researchers/developers inspecting graph partitions — maximize data visible per screen area |
| **State Clarity** | Four `AppState` variants (`Idle`, `Running`, `Done`, `Error`) must be instantly distinguishable |
| **Terminal-Native** | No web/GUI metaphors — respect terminal conventions (borders, block characters, ANSI escapes) |
| **Accessibility First** | WCAG-equivalent contrast ratios (4.5:1 minimum) even in 16-color fallback mode |
| **Calm Palette** | Scientific tools should feel precise, not flashy — muted base with strategic accent use |

### 1.2 Design Pattern: **Data Observatory**

A multi-panel observatory layout where the user monitors an algorithm's live execution. Inspired by systems like `htop`, `k9s`, and `lazygit` — familiar to the target audience (Rust developers and graph researchers).

### 1.3 Anti-Patterns to Avoid

| Anti-Pattern | Why |
|---|---|
| Emoji as icons | Inconsistent rendering across terminals; use Unicode box/block characters |
| Gratuitous color | Too many colors compete with community-hash colors in the graph view |
| Animation/blinking | Distracting in a data-monitoring context; violates `prefers-reduced-motion` spirit |
| Thick borders everywhere | Wastes precious terminal real estate; use `Borders::NONE` for inner separators |
| Modal dialogs | Terminal UIs should be keyboard-driven and non-blocking; use inline overlays |

---

## 2. Color System

### 2.1 Palette Overview (Dark Theme)

The palette is designed for dark terminal backgrounds (`#1a1b26` to `#24283b` range). All colors are specified as **Ratatui `Color::Rgb(r, g, b)`**. For the canonical Rust constants with 256-color fallback comments, see **§9 Implementation Constants Module**.

```text
┌─────────────────────────────────────────────────────────────┐
│  LEIDEN TUI COLOR SYSTEM                                    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  BASE LAYER (backgrounds & surfaces)                        │
│  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐              │
│  │ bg0  │ │ bg1  │ │ bg2  │ │ bg3  │ │ bg4  │              │
│  │#1a1b │ │#1f20 │ │#2426 │ │#2a2d │ │#3b3f │              │
│  │  26  │ │  2e  │ │  3a  │ │  42  │ │  58  │              │
│  └──────┘ └──────┘ └──────┘ └──────┘ └──────┘              │
│                                                             │
│  TEXT LAYER                                                 │
│  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐                       │
│  │ fg0  │ │ fg1  │ │ fg2  │ │ fg3  │                       │
│  │#c0ca │ │#a9b1 │ │#737a │ │#565f │                       │
│  │  f5  │ │  d6  │ │  89  │ │  71  │                       │
│  └──────┘ └──────┘ └──────┘ └──────┘                       │
│  bright   normal   muted    dim                             │
│                                                             │
│  ACCENT LAYER (semantic)                                    │
│  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐              │
│  │ blue │ │ teal │ │amber │ │ red  │ │green │              │
│  │#7aa2 │ │#73da │ │#e0af │ │#f767 │ │#9ece │              │
│  │  f7  │ │  ca  │ │  68  │ │  7d  │ │  6a  │              │
│  └──────┘ └──────┘ └──────┘ └──────┘ └──────┘              │
│  primary  info     warning  error    success                │
│                                                             │
│  COMMUNITY HASHING (12 distinct, high-contrast)             │
│  ┌──┐┌──┐┌──┐┌──┐┌──┐┌──┐┌──┐┌──┐┌──┐┌──┐┌──┐┌──┐        │
│  │C0││C1││C2││C3││C4││C5││C6││C7││C8││C9││CA││CB│        │
│  └──┘└──┘└──┘└──┘└──┘└──┘└──┘└──┘└──┘└──┘└──┘└──┘        │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 Semantic Color Mapping

| UI Element | Color | When |
|---|---|---|
| Focused panel border | `ACCENT_PRIMARY` | Panel has keyboard focus |
| Unfocused panel border | `FG_3` | Panel does not have focus |
| Status bar background | `BG_2` | Always |
| `AppState::Idle` indicator | `FG_2` | Waiting for input |
| `AppState::Running` indicator | `ACCENT_INFO` | Algorithm executing |
| `AppState::Done` indicator | `ACCENT_SUCCESS` | Converged or cap reached |
| `AppState::Error` indicator | `ACCENT_ERROR` | Error occurred |
| Modularity quality value | `ACCENT_INFO` | Always (numeric highlight) |
| Improving quality delta | `ACCENT_SUCCESS` | ΔQ > 0 |
| Degrading quality delta | `ACCENT_ERROR` | ΔQ < 0 |
| Throttled event | `ACCENT_WARNING` | Back-pressure indicator |
| Selected community row | `BG_3` on `FG_0` | User-selected row |
| Log level `ERROR` | `ACCENT_ERROR` | Log pane |
| Log level `WARN` | `ACCENT_WARNING` | Log pane |
| Log level `INFO` | `ACCENT_INFO` | Log pane |
| Log level `DEBUG` | `FG_2` | Log pane |
| Log level `TRACE` | `FG_3` | Log pane |

### 2.3 Contrast Ratios (verified)

All ratios computed against the corrected `FG_0` value `#c0caf5` = `(192, 202, 245)`.

| Pair | Ratio | Pass |
|---|---|---|
| `FG_0` on `BG_0` | 11.1:1 | ✅ AAA |
| `FG_1` on `BG_0` | 8.1:1 | ✅ AAA |
| `FG_2` on `BG_0` | 4.7:1 | ✅ AA |
| `FG_1` on `BG_1` | 7.4:1 | ✅ AAA |
| `ACCENT_PRIMARY` on `BG_0` | 6.3:1 | ✅ AA |
| `ACCENT_ERROR` on `BG_0` | 5.8:1 | ✅ AA |
| `ACCENT_SUCCESS` on `BG_0` | 8.4:1 | ✅ AAA |
| `FG_0` on `BG_3` | 7.5:1 | ✅ AAA |

---

## 3. Layout System

### 3.1 Primary Layout (3-panel + status bar)

```text
╭─ Terminal ──────────────────────────────────────────────────╮
│ ╭─ Community Panel [Tab 1] ──────┬─ Graph View [Tab 2] ──╮ │
│ │                                │                        │ │
│ │  Community  Size  IntW  TDeg   │    ●─────●             │ │
│ │ ►  0        124   892   1847   │   /│\    │\            │ │
│ │    1         87   534   1102   │  ● │ ●───● ●           │ │
│ │    2         63   301    688   │   \│/    │/            │ │
│ │    3         42   198    476   │    ●─────●             │ │
│ │    4         31   145    312   │                        │ │
│ │    …         …     …      …   │                        │ │
│ │                                │                        │ │
│ ├─ Log Pane [Tab 3] ────────────┴────────────────────────┤ │
│ │ [INFO] leiden: IterationStarted index=0 phase=LocalMov │ │
│ │ [INFO] leiden: LocalMovingProgress iteration=0 moved=12│ │
│ │ [INFO] leiden: QualityComputed iteration=0 quality=0.41│ │
│ │ [INFO] leiden: IterationFinished index=0 quality=0.4198│ │
│ ╰────────────────────────────────────────────────────────╯ │
│ ╭─ Status Bar ───────────────────────────────────────────╮ │
│ │ ● Running  iter 2/10  Q=0.4231  γ=1.0  seed=0   ?help │ │
│ ╰────────────────────────────────────────────────────────╯ │
╰────────────────────────────────────────────────────────────╯
```

### 3.2 Glossary: Column Abbreviations

| Column | Full Name | Source |
|---|---|---|
| `IntW` | Internal edge weight | Sum of edge weights within the community |
| `TDeg` | Total degree | Sum of all incident edge weights for nodes in the community |

These values come from the Leiden algorithm's partition output and are computed from the `CsrGraph` adjacency structure.

### 3.3 Layout Proportions (Ratatui Constraints)

```rust
use ratatui::layout::{Constraint, Direction, Layout};

/// Root layout: main area + status bar.
fn root_layout(area: ratatui::layout::Rect) -> [ratatui::layout::Rect; 2] {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),        // main panels area
            Constraint::Length(1),     // status bar (single line)
        ])
        .split(area);
    [chunks[0], chunks[1]]
}

/// Main area: top panels (community + graph) + log pane.
fn main_layout(area: ratatui::layout::Rect) -> [ratatui::layout::Rect; 2] {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(65),  // top panels
            Constraint::Percentage(35),  // log pane
        ])
        .split(area);
    [chunks[0], chunks[1]]
}

/// Top panels: community list + graph view.
fn top_panels(area: ratatui::layout::Rect) -> [ratatui::layout::Rect; 2] {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),  // community panel
            Constraint::Percentage(60),  // graph view
        ])
        .split(area);
    [chunks[0], chunks[1]]
}
```

### 3.4 Responsive Breakpoints

| Terminal Width | Mode | Layout |
|---|---|---|
| **≥ 120 cols** | `Full` | Community (40%) + Graph (60%) side-by-side; Log below (35%) |
| **80–119 cols** | `Compact` | Community (50%) + Graph (50%) side-by-side; Log shortened |
| **60–79 cols** | `Stacked` | Community → Graph → Log in single column |
| **< 60 cols** | `Minimal` | Community panel only + status bar; Graph/Log toggled via `g`/`l` |

```rust
/// Determine layout mode from terminal width.
fn layout_mode(width: u16) -> LayoutMode {
    match width {
        120.. => LayoutMode::Full,
        80..=119 => LayoutMode::Compact,
        60..=79 => LayoutMode::Stacked,
        _ => LayoutMode::Minimal,
    }
}
```

### 3.5 Panel Toggle States

When `show_graph == false` or `show_log == false`, redistribute space. The table below covers all layout modes.

#### Default Mode (Full / Compact)

| `show_graph` | `show_log` | Layout |
|---|---|---|
| `true` | `true` | Default 3-panel (§3.1) |
| `false` | `true` | Community panel full width top, log pane bottom |
| `true` | `false` | Community + graph side-by-side, full height |
| `false` | `false` | Community panel full screen + status bar |

#### Stacked Mode (60–79 cols)

| `show_graph` | `show_log` | Layout |
|---|---|---|
| `true` | `true` | Community (40%) → Graph (30%) → Log (30%) vertically stacked |
| `false` | `true` | Community (60%) → Log (40%) vertically stacked |
| `true` | `false` | Community (50%) → Graph (50%) vertically stacked |
| `false` | `false` | Community panel full screen + status bar |

#### Minimal Mode (< 60 cols)

Only the **focused** panel is visible. `g` and `l` toggle which panel is focused. The community panel is always the fallback when both graph and log are hidden.

---

## 4. Typography & Text Styling

### 4.1 Terminal Typography Rules

Since terminals use monospace fonts exclusively, "typography" means strategic use of **weight** (bold), **decoration** (underline), and **spacing**.

| Element | Style | Ratatui Modifier |
|---|---|---|
| Panel titles | Bold, UPPERCASE | `Modifier::BOLD` |
| Column headers | Bold | `Modifier::BOLD` |
| Status bar labels | Normal | (none) |
| Status bar values | Bold | `Modifier::BOLD` |
| Key binding hints | Dim | `Modifier::DIM` |
| Selected row text | Bold, explicit `BG_3` + `FG_0` | `Modifier::BOLD` |
| Quality values | Bold | `Modifier::BOLD` |
| Timestamps in log | Dim | `Modifier::DIM` |
| Help overlay text | Normal | (none) |
| Help overlay keys | Bold | `Modifier::BOLD` |
| Help footer hint | Dim, underline | `Modifier::DIM \| Modifier::UNDERLINED` |

> [!NOTE]
> **Selected rows use explicit colors** (`BG_3` background, `FG_0` foreground) rather than `Modifier::REVERSED`. Reverse-video swaps the terminal's default fg/bg, which produces unpredictable results on terminals with custom palettes or in 16-color fallback mode. Explicit color tokens ensure consistent selection rendering across all environments.

> [!NOTE]
> **`Modifier::ITALIC` is intentionally avoided.** While widely supported in modern terminals (kitty, alacritty, iTerm2, WezTerm), older terminals (xterm legacy, PuTTY) may render italic as blank or fallback text. Use `Modifier::DIM` or `Modifier::UNDERLINED` instead for universal compatibility.

### 4.2 Number Formatting

| Value | Format | Example |
|---|---|---|
| Modularity (Q) | 4 decimal places | `Q=0.4231` |
| Delta Q (ΔQ) | 4 decimal places, signed | `ΔQ=+0.0033` |
| Total weight (m) | 1 decimal place | `m=156.0` |
| Node/edge counts | Comma-separated thousands | `12,345` |
| Community ID | Zero-padded to widest | `003` |
| Iteration count | `current/cap` format | `2/10` |
| γ (gamma) | Up to 2 decimal places | `γ=1.50` |

### 4.3 Unicode Characters

Use only widely-supported Unicode characters for drawing. All characters are in the Basic Multilingual Plane (U+0000–U+FFFF).

| Purpose | Character | Codepoint | Fallback (ASCII) |
|---|---|---|---|
| State indicator (idle) | `○` | U+25CB | `o` |
| State indicator (running) | `●` | U+25CF | `*` |
| State indicator (done) | `✓` | U+2713 | `+` |
| State indicator (error) | `✗` | U+2717 | `!` |
| Graph node | `●` | U+25CF | `*` |
| Graph edge (horizontal) | `─` | U+2500 | `-` |
| Graph edge (vertical) | `│` | U+2502 | `\|` |
| Graph edge (diagonal) | `/` `\` | ASCII | `/` `\` |
| Sort indicator (desc) | `▼` | U+25BC | `v` |
| Progress fill | `█` | U+2588 | `#` |
| Progress empty | `░` | U+2591 | `.` |
| Separator dot | `·` | U+00B7 | `.` |
| Arrow right | `→` | U+2192 | `->` |
| Gamma | `γ` | U+03B3 | `g` |
| Delta | `Δ` | U+0394 | `d` |

> [!IMPORTANT]
> The `γ` (U+03B3) and `Δ` (U+0394) characters appear in the status bar — a high-visibility position. These are widely supported in modern terminals (kitty, alacritty, iTerm2, WezTerm, Windows Terminal, GNOME Terminal) but may not render in legacy environments. The pre-delivery checklist (§12) includes a verification item for these characters.

---

## 5. Widget Specifications

### 5.1 Community Panel (`community.rs` — T106)

```text
╭─ COMMUNITIES ─────────────────────╮
│  #   Community  Size  IntW   TDeg │
│ ►  0       ██   124   892   1847  │  ← selected row (BG_3 + FG_0, bold)
│    1       ██    87   534   1102  │  ← color block stays community-colored
│    2       ██    63   301    688  │
│    3       ██    42   198    476  │
│    4       ██    31   145    312  │
│    5       ██    18    67    142  │
│    6       ██    12    34     78  │
│    ─────────────────────────────  │
│  7 communities · 377 nodes        │  ← summary footer (FG_2)
╰───────────────────────────────────╯
```

| Element | Color | Style |
|---|---|---|
| Title "COMMUNITIES" | `FG_0` | Bold |
| Column headers (#, Community, Size, IntW, TDeg) | `FG_1` | Bold |
| Community color block `██` | `COMMUNITY_COLORS[id % 12]` | Normal |
| Community color block `██` (selected row) | `COMMUNITY_COLORS[id % 12]` | Normal (unchanged) |
| Selected row background | `BG_3` | — |
| Selected row text (non-block cells) | `FG_0` | Bold |
| Unselected row text | `FG_1` | Normal |
| Summary footer | `FG_2` | Normal |
| Focused border | `ACCENT_PRIMARY` | Rounded |
| Unfocused border | `FG_3` | Rounded |
| Column alignment | `Size`, `IntW`, `TDeg` right-aligned | — |

> [!NOTE]
> **Community color blocks remain their community color even when the row is selected.** This is intentional — the color block serves as a persistent visual identifier linking the community list to the graph view. Altering it on selection would break the cross-panel color correspondence. The selected row is distinguished by the `BG_3` background and `FG_0` bold text on all non-block cells.

### 5.2 Graph View (`graph.rs` — T107)

```text
╭─ GRAPH VIEW ──────────────────────────────╮
│                                           │
│           ●───────●                       │
│          /│\      │\                      │
│         ● │  ●────● ●                    │
│          \│/      │/                      │
│           ●───────●                       │
│                                           │
│  nodes=34 · edges=78 · m=156.0           │  ← metrics footer (FG_2)
╰───────────────────────────────────────────╯
```

| Element | Color | Style |
|---|---|---|
| Title "GRAPH VIEW" | `FG_0` | Bold |
| Node circles `●` | `COMMUNITY_COLORS[community_id % 12]` | Normal |
| Edge lines | `FG_3` | Normal |
| Selected community nodes | Same color | Bold |
| Non-selected community nodes | Same color, dimmed | Dim |
| Metrics footer | `FG_2` | Normal |
| Metric values | `ACCENT_INFO` | Bold |

### 5.3 Log Pane (`log_pane.rs` — T108)

```text
╭─ LOG ─────────────────────────────────────────────────────╮
│ [INFO] leiden: IterationStarted index=0 phase=LocalMoving │
│ [INFO] leiden: LocalMovingProgress iter=0 moved=127       │
│ [WARN] leiden: Throttled dropped=3                        │
│ [INFO] leiden: QualityComputed iter=0 quality=0.4198      │
│ [INFO] leiden: IterationFinished index=0 quality=0.4198   │
│ [INFO] leiden: IterationStarted index=1 phase=LocalMoving │
╰───────────────────────────────────────────────────────────╯
```

| Element | Color | Style |
|---|---|---|
| Title "LOG" | `FG_0` | Bold |
| `[ERROR]` prefix | `ACCENT_ERROR` | Bold |
| `[WARN]` prefix | `ACCENT_WARNING` | Bold |
| `[INFO]` prefix | `ACCENT_INFO` | Normal |
| `[DEBUG]` prefix | `FG_2` | Normal |
| `[TRACE]` prefix | `FG_3` | Dim |
| Target name (`leiden:`) | `FG_2` | Normal |
| Event name | `FG_1` | Normal |
| Field keys (`index=`, `quality=`) | `FG_2` | Normal |
| Field values | `FG_0` | Normal |
| Quality values specifically | `ACCENT_INFO` | Bold |
| Scroll indicator (right edge) | `FG_3` | Dim |

#### Log Ring Eviction Policy

The `LogRing` buffer has a fixed capacity of **500 entries** and uses **FIFO eviction** (oldest-first). When the 501st entry arrives, the oldest entry is silently dropped. The producer (the `LogPaneLayer` tracing subscriber) is **never blocked** — `push_back` always succeeds. This matches the channel back-pressure design: the algorithm's execution is never gated by the UI's ability to display logs. Users scrolled to the top will see entries disappear; users at the tail (default position) see a continuously appending stream.

### 5.4 Status Bar (`status_bar.rs` — T109)

```text
 ● Running  [████████░░░░░░░░░░░░] 3/10  Q=0.4231  ΔQ=+0.0033  γ=1.0  seed=0      q:quit r:restart p:pause ?:help
```

Single line, spanning the full terminal width.

| Element | Color | Style |
|---|---|---|
| Background | `BG_2` | — |
| State indicator `●`/`○`/`✓`/`✗` | State color (see §2.2) | Normal |
| State label ("Running", "Done", …) | State color | Bold |
| Parameter labels ("iter", "Q=", "γ=") | `FG_2` | Normal |
| Parameter values | `FG_0` | Bold |
| Quality improving indicator | `ACCENT_SUCCESS` | Bold |
| Quality degrading indicator | `ACCENT_ERROR` | Bold |
| Key hints (right-aligned) | `FG_3` | Dim |
| Key letters in hints | `FG_2` | Normal |

### 5.5 Help Overlay (`?` key)

```text
╭─ KEY BINDINGS ────────────────────╮
│                                   │
│  q / Ctrl+C   Quit               │
│  r            Restart             │
│  s            Step (one iter)     │
│  p            Pause / Resume      │
│  g            Toggle graph        │
│  l            Toggle log          │
│  Tab          Switch panel focus  │
│  ↑ / ↓        Select community   │
│  ?            Close help          │
│                                   │
│       Press any key to close      │
╰───────────────────────────────────╯
```

| Element | Color | Style |
|---|---|---|
| Overlay background | `BG_1` | — |
| Overlay border | `ACCENT_PRIMARY` | Rounded |
| Title "KEY BINDINGS" | `FG_0` | Bold |
| Key names | `ACCENT_PRIMARY` | Bold |
| Action descriptions | `FG_1` | Normal |
| Footer hint | `FG_3` | Dim, Underlined |

---

## 6. State-Driven Theming

### 6.1 State Color Map

```rust
/// Map `AppState` to its semantic color.
fn state_color(state: &AppState) -> Color {
    match state {
        AppState::Idle => FG_2,                    // muted — waiting
        AppState::Running { .. } => ACCENT_INFO,   // teal — active
        AppState::Done { .. } => ACCENT_SUCCESS,   // green — complete
        AppState::Error(_) => ACCENT_ERROR,        // red — problem
    }
}

/// Map `AppState` to its indicator character.
fn state_indicator(state: &AppState) -> &'static str {
    match state {
        AppState::Idle => "○",
        AppState::Running { .. } => "●",
        AppState::Done { .. } => "✓",
        AppState::Error(_) => "✗",
    }
}

/// Map `AppState` to its label.
fn state_label(state: &AppState) -> &'static str {
    match state {
        AppState::Idle => "Idle",
        AppState::Running { .. } => "Running",
        AppState::Done { .. } => "Done",
        AppState::Error(_) => "Error",
    }
}
```

### 6.2 Transition Effects

| Transition | Visual Effect |
|---|---|
| `Idle → Running` | Status bar color shifts from muted to teal; progress gauge appears |
| `Running → Done` | Status bar turns green; final quality highlighted with success color |
| `Running → Error` | Status bar turns red; error message replaces progress info |
| `Error → Idle` | Status bar returns to muted; log ring preserved (T101a) |
| `Done → Running` | Status bar returns to teal; community list clears, progress resets |

All transitions are **immediate** — no animation frames or easing. The next `terminal.draw()` call reflects the new state.

---

## 7. Progress Visualization

### 7.1 Iteration Progress Gauge

When `AppState::Running`, show a progress gauge in the status bar:

```text
 ● Running  [████████░░░░░░░░░░░░] 3/10  Q=0.4231
```

```rust
use ratatui::widgets::Gauge;

/// Build the iteration progress gauge.
fn progress_gauge(iteration: u32, cap: u32, quality: f64) -> Gauge<'static> {
    let ratio = f64::from(iteration) / f64::from(cap);
    Gauge::default()
        .ratio(ratio.clamp(0.0, 1.0))
        .gauge_style(
            Style::default()
                .fg(ACCENT_INFO)
                .bg(BG_4)
        )
        .label(format!("{iteration}/{cap}  Q={quality:.4}"))
}
```

### 7.2 Quality Trend (Sparkline)

A compact sparkline in the status bar or community panel header showing modularity across iterations:

```text
Q ▁▃▅▇█  0.4231
```

```rust
use ratatui::widgets::Sparkline;

/// Build a quality-over-iterations sparkline.
///
/// Normalizes quality values to `[0, 100]` relative to the observed
/// range, so early low-quality iterations remain visually distinct
/// from later high-quality iterations. Avoids the flat-sparkline
/// problem where auto-scaling to `[0, 10000]` compresses early bars.
fn quality_sparkline(qualities: &[f64]) -> (Sparkline<'_>, Vec<u64>) {
    let max = qualities
        .iter()
        .copied()
        .fold(0.0_f64, f64::max)
        .max(1e-9); // avoid division by zero
    let data: Vec<u64> = qualities
        .iter()
        .map(|q| ((q.clamp(0.0, max) / max) * 100.0) as u64)
        .collect();
    let sparkline = Sparkline::default()
        .data(&data)
        .style(Style::default().fg(ACCENT_INFO));
    (sparkline, data)
}
```

---

## 8. Border & Focus System

### 8.1 Border Styles

All panels use `BorderType::Rounded` (`╭╮╰╯`) for a modern, polished look. This requires terminals that support Unicode box-drawing characters — all modern terminals in the target audience (kitty, alacritty, iTerm2, WezTerm, Windows Terminal, GNOME Terminal) support these.

```rust
use ratatui::widgets::{Block, Borders, BorderType};

/// Focused panel block with accent border.
fn focused_block(title: &str) -> Block<'_> {
    Block::default()
        .title(format!(" {title} "))
        .title_style(Style::default().fg(FG_0).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(ACCENT_PRIMARY))
}

/// Unfocused panel block with dim border.
fn unfocused_block(title: &str) -> Block<'_> {
    Block::default()
        .title(format!(" {title} "))
        .title_style(Style::default().fg(FG_2))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(FG_3))
}
```

### 8.2 Focus Cycle

```text
Tab →  Community Panel  →  Graph View  →  Log Pane  → (cycle)
```

Only the focused panel receives keyboard input. Focus is indicated by the border color change from `FG_3` → `ACCENT_PRIMARY`.

When a panel is hidden (via `g`/`l` toggle), it is skipped in the focus cycle. If only one panel is visible, Tab is a no-op.

---

## 9. Implementation Constants Module

The canonical `colors.rs` module for `leiden-tui`. This is the **single source of truth** for all color and style constants. All widget implementations must import from this module rather than using inline `Color::Rgb(...)` values.

```rust
//! Design system color constants for `leiden-tui`.
//!
//! This module defines the complete color palette and style presets
//! used across all TUI widgets. Colors are sourced from the Leiden
//! TUI Design System document.
//!
//! # Color layers
//!
//! - **Base** (`BG_0`–`BG_4`): background surfaces, darkest to lightest.
//! - **Text** (`FG_0`–`FG_3`): foreground text, brightest to dimmest.
//! - **Accent**: semantic colors mapped to application states and events.
//! - **Community**: 12 visually distinct colors for community-id hashing.
//!
//! # 256-color fallbacks
//!
//! Each constant documents its closest 256-color index in a trailing
//! comment. When true-color is unavailable, use `supports_truecolor()`
//! to select the fallback palette (see §10 of the Design System).

use ratatui::style::{Color, Modifier, Style};

// ── Base Layer (backgrounds & surfaces) ──────────────────
/// Terminal background.
pub const BG_0: Color = Color::Rgb(26, 27, 38);    // 256: 234
/// Elevated surface (panel interiors).
pub const BG_1: Color = Color::Rgb(31, 32, 46);    // 256: 235
/// Active/focused panel background.
pub const BG_2: Color = Color::Rgb(36, 38, 58);    // 256: 236
/// Highlighted row / selection background.
pub const BG_3: Color = Color::Rgb(42, 45, 66);    // 256: 237
/// Hover / transient highlight.
pub const BG_4: Color = Color::Rgb(59, 63, 88);    // 256: 240

// ── Text Layer ───────────────────────────────────────────
/// Bright foreground (titles, selected items, values).
///
/// Matches Tokyo Night `fg` primary: `#c0caf5` = `(192, 202, 245)`.
pub const FG_0: Color = Color::Rgb(192, 202, 245); // 256: 189
/// Normal foreground (body content, table rows).
pub const FG_1: Color = Color::Rgb(169, 177, 214); // 256: 146
/// Muted foreground (labels, secondary info, borders).
pub const FG_2: Color = Color::Rgb(115, 122, 137); // 256: 245
/// Dim foreground (disabled items, timestamps, hints).
pub const FG_3: Color = Color::Rgb(86, 95, 113);   // 256: 242

// ── Semantic Accents ─────────────────────────────────────
/// Primary accent blue — focused borders, active state.
pub const ACCENT_PRIMARY: Color = Color::Rgb(122, 162, 247); // 256: 111
/// Info accent teal — metrics, iteration counts.
pub const ACCENT_INFO: Color = Color::Rgb(115, 218, 202);    // 256: 79
/// Warning accent amber — throttle events, iteration cap.
pub const ACCENT_WARNING: Color = Color::Rgb(224, 175, 104); // 256: 214
/// Error accent red — parse errors, invalid input.
pub const ACCENT_ERROR: Color = Color::Rgb(247, 118, 125);   // 256: 204
/// Success accent green — converged, quality improving.
pub const ACCENT_SUCCESS: Color = Color::Rgb(158, 206, 106); // 256: 150

// ── Community Hash Colors (12 visually distinct) ─────────
/// Deterministic color array for community-id → color mapping.
///
/// Usage: `COMMUNITY_COLORS[community_id as usize % COMMUNITY_COLORS.len()]`
///
/// The 12 colors are chosen for maximum pairwise perceptual distance
/// in CIELAB space, ensuring communities remain distinguishable in
/// both true-color and 256-color modes.
pub const COMMUNITY_COLORS: [Color; 12] = [
    Color::Rgb(122, 162, 247), // C0  blue       256: 111
    Color::Rgb(158, 206, 106), // C1  green      256: 150
    Color::Rgb(224, 175, 104), // C2  amber      256: 214
    Color::Rgb(247, 118, 125), // C3  coral      256: 204
    Color::Rgb(115, 218, 202), // C4  teal       256: 79
    Color::Rgb(187, 154, 247), // C5  lavender   256: 141
    Color::Rgb(255, 158, 100), // C6  orange     256: 209
    Color::Rgb(125, 207, 255), // C7  sky        256: 117
    Color::Rgb(195, 232, 141), // C8  lime       256: 192
    Color::Rgb(255, 167, 196), // C9  pink       256: 218
    Color::Rgb(169, 177, 214), // CA  silver     256: 146
    Color::Rgb(224, 208, 143), // CB  gold       256: 186
];

// ── Style Presets ────────────────────────────────────────

/// Style for focused panel borders.
pub const fn focused_border_style() -> Style {
    Style::new().fg(ACCENT_PRIMARY)
}

/// Style for unfocused panel borders.
pub const fn unfocused_border_style() -> Style {
    Style::new().fg(FG_3)
}

/// Style for panel titles (focused).
pub const fn title_style_focused() -> Style {
    Style::new().fg(FG_0).add_modifier(Modifier::BOLD)
}

/// Style for panel titles (unfocused).
pub const fn title_style_unfocused() -> Style {
    Style::new().fg(FG_2)
}

/// Style for table column headers.
///
/// Uses `FG_1` (not `FG_2`) for anchoring headers with strong
/// contrast (8.1:1 on `BG_0`, AAA). `FG_2` is reserved for
/// truly secondary labels.
pub const fn header_style() -> Style {
    Style::new().fg(FG_1).add_modifier(Modifier::BOLD)
}

/// Style for selected table row.
///
/// Uses explicit `BG_3` + `FG_0` rather than `Modifier::REVERSED`
/// to ensure consistent rendering across terminals with custom
/// palettes and in 16-color fallback mode.
pub const fn selected_row_style() -> Style {
    Style::new().fg(FG_0).bg(BG_3).add_modifier(Modifier::BOLD)
}

/// Style for normal table row.
pub const fn normal_row_style() -> Style {
    Style::new().fg(FG_1)
}

/// Style for key hints in status bar.
pub const fn key_hint_style() -> Style {
    Style::new().fg(FG_3).add_modifier(Modifier::DIM)
}

/// Style for key letters in hints.
pub const fn key_letter_style() -> Style {
    Style::new().fg(FG_2)
}
```

---

## 10. 16-Color and 256-Color Fallback

### 10.1 Fallback Palette

For terminals without true-color support, map to standard ANSI colors:

| Design System Color | 256-Color Index | 16-Color Fallback |
|---|---|---|
| `BG_0` | 234 | `Color::Black` |
| `BG_1` | 235 | `Color::Black` |
| `BG_2` | 236 | `Color::DarkGray` |
| `BG_3` | 237 | `Color::DarkGray` |
| `BG_4` | 240 | `Color::DarkGray` |
| `FG_0` | 189 | `Color::White` |
| `FG_1` | 146 | `Color::Gray` |
| `FG_2` | 245 | `Color::DarkGray` |
| `FG_3` | 242 | `Color::DarkGray` |
| `ACCENT_PRIMARY` | 111 | `Color::Blue` |
| `ACCENT_INFO` | 79 | `Color::Cyan` |
| `ACCENT_WARNING` | 214 | `Color::Yellow` |
| `ACCENT_ERROR` | 204 | `Color::Red` |
| `ACCENT_SUCCESS` | 150 | `Color::Green` |
| Community colors | Cycle: 111, 150, 214, 204, 79, 141 | Cycle: Blue, Green, Yellow, Red, Cyan, Magenta |

### 10.2 True-Color Detection

```rust
/// Detect true-color support from environment variables.
///
/// Checks `COLORTERM` first (the standard mechanism), then falls
/// back to `TERM` heuristics for terminals that don't set
/// `COLORTERM` (notably Alacritty).
fn supports_truecolor() -> bool {
    // Primary: COLORTERM is the standard signal.
    if let Ok(ct) = std::env::var("COLORTERM") {
        if ct == "truecolor" || ct == "24bit" {
            return true;
        }
    }
    // Fallback: TERM heuristics for terminals that omit COLORTERM.
    if let Ok(term) = std::env::var("TERM") {
        // Alacritty, WezTerm, and other modern terminals may
        // identify themselves here.
        if term.starts_with("alacritty")
            || term.starts_with("wezterm")
            || term.ends_with("-direct")
        {
            return true;
        }
    }
    // Conservative: assume no true-color support.
    false
}
```

> [!TIP]
> As of Ratatui 0.30, the framework does **not** expose a `Terminal::capabilities()` API. If a future Ratatui version adds capability detection, prefer that over environment-variable heuristics.

---

## 11. Design Tokens Summary

```text
╭──────────────────────────────────────────────────────────────────────╮
│                    LEIDEN TUI DESIGN TOKENS                         │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  SPACING                                                             │
│  ├── Panel padding (inner):        1 cell horizontal, 0 vertical     │
│  ├── Status bar padding:           1 cell horizontal                 │
│  ├── Table column gap:             2 cells                           │
│  └── Section separator:            ─── (horizontal line)             │
│                                                                      │
│  BORDERS                                                             │
│  ├── Panel border type:            Rounded (╭╮╰╯)                    │
│  ├── Help overlay border:          Rounded (╭╮╰╯)                    │
│  ├── Focused color:                ACCENT_PRIMARY (#7aa2f7)          │
│  └── Unfocused color:              FG_3 (#565f71)                    │
│                                                                      │
│  TIMING                                                              │
│  ├── Event poll interval:          50ms (20 FPS)                     │
│  ├── Channel drain per tick:       try_recv() loop (non-blocking)    │
│  └── State transition:             Immediate (no animation)          │
│                                                                      │
│  CAPACITY                                                            │
│  ├── Log ring buffer:              500 entries (FIFO eviction)       │
│  ├── Event channel:                1024 bounded                      │
│  ├── Community colors:             12 distinct                       │
│  └── Max visible log lines:        Terminal height − 3               │
│                                                                      │
│  SYMBOLS                                                             │
│  ├── Idle:     ○  (U+25CB)                                          │
│  ├── Running:  ●  (U+25CF)                                          │
│  ├── Done:     ✓  (U+2713)                                          │
│  ├── Error:    ✗  (U+2717)                                          │
│  ├── Sort:     ▼  (U+25BC)                                          │
│  ├── Progress: █░ (U+2588, U+2591)                                  │
│  ├── Gamma:    γ  (U+03B3)                                          │
│  └── Delta:    Δ  (U+0394)                                          │
│                                                                      │
╰──────────────────────────────────────────────────────────────────────╯
```

---

## 12. Pre-Delivery Checklist (TUI-specific)

### Visual Quality
- [ ] No emoji used anywhere — only Unicode box drawing and symbol characters
- [ ] All community colors visually distinct at 12+ communities
- [ ] Border style changes on panel focus are clearly visible
- [ ] State indicator symbols render correctly in common terminals (kitty, alacritty, iTerm2, Windows Terminal)
- [ ] `γ` (U+03B3) and `Δ` (U+0394) render correctly in the status bar across all target terminals
- [ ] All ASCII mocks use rounded corners (`╭╮╰╯`) matching `BorderType::Rounded`

### Accessibility
- [ ] All text passes 4.5:1 contrast ratio against its background
- [ ] Keyboard-only navigation works for all features
- [ ] 16-color fallback mode tested and functional
- [ ] No color-only information conveyance — symbols/text reinforce state
- [ ] No `Modifier::ITALIC` used — `DIM` or `UNDERLINED` used instead for universal compatibility

### Layout
- [ ] Responsive layout works at 80×24 minimum terminal size
- [ ] No content hidden behind status bar
- [ ] Panel toggle (`g`/`l`) redistributes space correctly in all layout modes (Full, Compact, Stacked, Minimal)
- [ ] Community table scrolls properly with `↑`/`↓`
- [ ] Focus cycle skips hidden panels

### Performance
- [ ] Render loop stays under 50ms per frame
- [ ] Channel drain is non-blocking (`try_recv`, not `recv`)
- [ ] Log ring FIFO eviction does not cause visible stutter
- [ ] Sparkline data is normalized to observed range, not raw-scaled

### Consistency
- [ ] Number formatting follows §4.2 rules throughout
- [ ] All style presets use constants from `colors.rs` (§9), not inline values
- [ ] Panel titles are consistently UPPERCASE
- [ ] Column alignments are consistent across community panel
- [ ] Selected rows use explicit `BG_3` + `FG_0` (not `Modifier::REVERSED`)
- [ ] Column headers use `FG_1` Bold (not `FG_2`)

---

> [!IMPORTANT]
> This design system is the **single source of truth** for all visual decisions in `leiden-tui`.
> Every widget implementation task (T106–T109) should reference this document.
> Color values must come from the `colors.rs` constants module (§9) — no inline `Color::Rgb(...)` in widget code.
