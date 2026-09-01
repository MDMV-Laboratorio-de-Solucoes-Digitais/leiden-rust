# Feature Specification: TUI Design System

**Feature Branch**: `[002-tui-design-system]`

**Created**: 2026-08-31

**Status**: Draft

**Input**: User description: "Leiden TUI Design System — a comprehensive design system for the `leiden-tui` interactive Terminal UI for Leiden community detection, covering color palette, layout specifications, widget styling, typography, state-driven theming, progress visualization, border/focus system, color fallback strategy, and implementation constants."

## Clarifications

### Session 2026-08-31

- Q: Should the TUI design system be a standalone crate or a module within `leiden-tui`? → A: Module within `leiden-tui` crate (e.g., `src/ui/colors.rs` + `src/ui/styles.rs`)
- Q: How many past iterations should the quality sparkline display before scrolling off? → A: 20 iterations (fixed window)
- Q: Should the help overlay organize key bindings into labeled groups or a flat list? → A: Grouped by category (e.g., "Navigation", "Panels", "General")
- Q: Is there a maximum number of communities before visual degradation is acceptable? → A: 100 communities (panel scrolls, no special handling beyond color cycling)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Visually Distinguishable Application States (Priority: P1)

As a researcher running the Leiden algorithm via `leiden-tui`, I want to instantly know whether the algorithm is idle, running, finished, or in error by glancing at the terminal, so that I can trust the system state without reading log messages.

**Why this priority**: State clarity is the single most critical visual requirement. Without it, the user cannot trust or act on the TUI output. Every other visual feature (community colors, log pane styling) is secondary to knowing *what the system is doing*.

**Independent Test**: Can be fully tested by launching `leiden-tui`, transitioning through all four `AppState` variants (`Idle → Running → Done`, `Running → Error`, `Error → Idle`), and verifying that each state is visually distinct — confirmed by both color and symbol indicator differences.

**Acceptance Scenarios**:

1. **Given** the TUI is in `Idle` state, **When** I look at the status bar, **Then** I see a muted (`FG_2`) circle indicator `○` and the label "Idle" in the same muted color.
2. **Given** the TUI is in `Running` state at iteration 3 of 10, **When** I look at the status bar, **Then** I see a teal (`ACCENT_INFO`) filled circle `●`, the label "Running" in teal, and a progress gauge showing `3/10`.
3. **Given** the algorithm has converged, **When** the TUI transitions to `Done` state, **Then** I see a green (`ACCENT_SUCCESS`) checkmark `✓` and the label "Done" with the final quality value highlighted in green.
4. **Given** a parse error occurred, **When** the TUI transitions to `Error` state, **Then** I see a red (`ACCENT_ERROR`) cross `✗` and the label "Error" with the error message replacing progress information.
5. **Given** the TUI is in `Error` state, **When** I press a key, **Then** the TUI transitions to `Idle` state, the status bar returns to muted coloring, and the log ring is preserved.

---

### User Story 2 - Community Identification Across Panels (Priority: P1)

As a developer inspecting partition results, I want each community to have a consistent, visually distinct color across the community panel and graph view, so that I can cross-reference which nodes belong to which community without reading numeric labels.

**Why this priority**: Cross-panel color correspondence is the primary mechanism for visual analysis of graph partitions — the core use case of the tool. Without it, the graph view provides no actionable information.

**Independent Test**: Can be fully tested by loading a graph with at least 12 communities, verifying that community `N` uses the same color (`COMMUNITY_COLORS[N % COMMUNITY_COLORS.len()]`) in both the community panel's color block and the graph view's node circles, and confirming that all 12 colors are perceptually distinguishable.

**Acceptance Scenarios**:

1. **Given** a partition with 7 communities, **When** I view the community panel and graph view side-by-side, **Then** each community's color block `██` in the community panel matches the node circle `●` color in the graph view for the same community ID.
2. **Given** a partition with more than 12 communities, **When** I view the community panel, **Then** community colors cycle deterministically via `community_id % COMMUNITY_COLORS.len()` and the same community ID always maps to the same color.
3. **Given** I select a community row in the community panel, **When** the selection highlight applies (`BG_3` background), **Then** the community color block remains unchanged (community-colored, not overridden by the selection style) while all non-block text cells switch to `FG_0` bold.

---

### User Story 3 - Responsive Multi-Panel Layout (Priority: P2)

As a developer using terminals of varying sizes, I want the TUI layout to adapt to my terminal width while keeping all critical information visible, so that I can use `leiden-tui` on both wide monitors and compact laptop screens.

**Why this priority**: Layout flexibility directly affects usability across the target audience's diverse terminal configurations. A rigid layout breaks on smaller screens, rendering the tool unusable for a significant portion of users.

**Independent Test**: Can be fully tested by resizing the terminal to each breakpoint (≥120, 80–119, 60–79, <60 columns) and verifying that panels rearrange according to the layout mode rules, with no content hidden behind the status bar and no horizontal overflow.

**Acceptance Scenarios**:

1. **Given** a terminal width ≥ 120 columns, **When** I launch `leiden-tui`, **Then** I see the Full layout: community panel (40%) and graph view (60%) side-by-side in the top area (65%), with the log pane below (35%) and a single-line status bar at the bottom.
2. **Given** a terminal width of 80–119 columns, **When** I view the TUI, **Then** the Compact layout shows community and graph panels at 50%/50% with a shortened log pane.
3. **Given** a terminal width of 60–79 columns, **When** I view the TUI, **Then** the Stacked layout shows community, graph, and log panels in a single vertical column.
4. **Given** a terminal width below 60 columns, **When** I view the TUI, **Then** the Minimal layout shows only the focused panel plus the status bar, with `g` and `l` toggling panel visibility.
5. **Given** any layout mode, **When** I press `g` to hide the graph panel, **Then** the remaining visible panels redistribute to fill the available space according to the panel toggle rules for the current layout mode.

---

### User Story 4 - Log Event Severity Differentiation (Priority: P2)

As a developer debugging algorithm behavior, I want log events to be color-coded by severity level, so that I can quickly scan for warnings and errors in a stream of INFO events.

**Why this priority**: The log pane is a critical diagnostic tool during algorithm execution. Without color-coded severity, the user must read every log line to find important events — defeating the purpose of real-time streaming.

**Independent Test**: Can be fully tested by feeding the log pane a mix of `TRACE`, `DEBUG`, `INFO`, `WARN`, and `ERROR` events and verifying that each level uses its designated color and style (e.g., `[ERROR]` = red bold, `[WARN]` = amber bold, `[INFO]` = teal normal).

**Acceptance Scenarios**:

1. **Given** the log pane displays events, **When** an `ERROR` event appears, **Then** the `[ERROR]` prefix is rendered in `ACCENT_ERROR` with bold modifier, and the remainder of the line uses the standard field styling.
2. **Given** the log pane displays events, **When** an `INFO` event appears, **Then** the `[INFO]` prefix is rendered in `ACCENT_INFO` without bold, distinguishable from the `ERROR` prefix.
3. **Given** the log pane has received 501 events (exceeding the 500-entry ring buffer), **When** I scroll to the top, **Then** the oldest event has been evicted (FIFO), and no blocking or stutter occurred during eviction.

---

### User Story 5 - Keyboard-Only Navigation with Focus Indication (Priority: P2)

As a terminal-native user, I want to navigate between panels using keyboard shortcuts and see clear visual feedback on which panel has focus, so that I can operate the TUI without a mouse and always know where my keystrokes will take effect.

**Why this priority**: Terminal users expect keyboard-first interaction. Focus indication prevents user confusion about which panel is active — critical for a multi-panel layout.

**Independent Test**: Can be fully tested by pressing `Tab` to cycle focus through visible panels and verifying that the focused panel's border changes from `FG_3` to `ACCENT_PRIMARY`, and that input events (e.g., `↑`/`↓` for community selection) only affect the focused panel.

**Acceptance Scenarios**:

1. **Given** the community panel is focused, **When** I look at its border, **Then** it is rendered with `ACCENT_PRIMARY` (#7aa2f7) and the title is bold `FG_0`, while the graph view and log pane borders are `FG_3` with `FG_2` titles.
2. **Given** the community panel is focused, **When** I press `Tab`, **Then** focus moves to the graph view (border changes to `ACCENT_PRIMARY`), and the community panel border reverts to `FG_3`.
3. **Given** the graph panel is hidden via `g`, **When** I press `Tab`, **Then** focus skips the graph view and moves to the next visible panel.
4. **Given** only one panel is visible (Minimal mode), **When** I press `Tab`, **Then** nothing happens (Tab is a no-op).

---

### User Story 6 - Accessible Color Contrast and Fallback (Priority: P3)

As a user with a terminal that does not support true-color, I want the TUI to degrade gracefully to 256-color or 16-color ANSI while maintaining legibility, so that I can use `leiden-tui` on any terminal environment.

**Why this priority**: While the primary audience uses modern terminals, CI environments, SSH sessions, and some development setups may not support true-color. Graceful degradation prevents the tool from being unusable in these contexts.

**Independent Test**: Can be fully tested by setting `COLORTERM=` (unset) and `TERM=xterm` to force 16-color mode, launching the TUI, and verifying that all text remains legible and state indicators are distinguishable.

**Acceptance Scenarios**:

1. **Given** a terminal with true-color support (`COLORTERM=truecolor`), **When** the TUI renders, **Then** all colors use the full RGB palette from the design system.
2. **Given** a terminal without `COLORTERM` set but with `TERM=alacritty`, **When** the `supports_truecolor()` function is called, **Then** it returns `true` (fallback heuristic for Alacritty).
3. **Given** a terminal with only 16-color support, **When** the TUI renders, **Then** `ACCENT_PRIMARY` maps to `Color::Blue`, `ACCENT_ERROR` maps to `Color::Red`, `ACCENT_SUCCESS` maps to `Color::Green`, and all text maintains at least 4.5:1 contrast ratio against its background.
4. **Given** any color mode, **When** the TUI displays state indicators, **Then** the indicator symbols (`○`, `●`, `✓`, `✗`) reinforce the state independently of color, ensuring accessibility for users with color vision deficiency.

---

### User Story 7 - Progress Visualization During Execution (Priority: P3)

As a researcher monitoring a long-running Leiden computation, I want to see a visual progress gauge and a quality trend sparkline, so that I can estimate time remaining and verify that modularity is converging.

**Why this priority**: Progress feedback reduces user anxiety during long runs and provides early detection of convergence issues — valuable but not essential for basic operation.

**Independent Test**: Can be fully tested by running the algorithm on a fixture and verifying that the progress gauge fills proportionally to `iteration / iteration_cap`, and that the sparkline's bar heights increase monotonically for a converging run.

**Acceptance Scenarios**:

1. **Given** the algorithm is running at iteration 5 of 10, **When** I look at the status bar, **Then** I see a progress gauge filled to approximately 50% with the label `5/10` and the current quality value.
2. **Given** the algorithm has completed 4 iterations with increasing quality, **When** I look at the sparkline, **Then** the bar heights increase from left to right, and the final quality value is displayed after the sparkline. The sparkline displays a fixed window of the most recent 20 iterations.
3. **Given** the algorithm has completed more than 20 iterations, **When** I look at the sparkline, **Then** only the 20 most recent quality values are shown (oldest values scroll off the left), and the displayed trend reflects the latest convergence behavior.
4. **Given** the quality decreased on one iteration (ΔQ < 0), **When** the status bar displays ΔQ, **Then** the value is rendered in `ACCENT_ERROR` (red) with a negative sign.

---

### Edge Cases

- What happens when the terminal is smaller than 80×24 (the documented minimum)? — The Minimal layout mode activates, showing only the focused panel plus status bar. Below 60 columns, single-panel mode is enforced.
- How does the TUI handle zero communities (empty or degenerate graph)? — The community panel shows an empty table with a footer reading "0 communities · 0 nodes". The graph view shows no nodes. The status bar reflects the `Terminated { reason: DegenerateInput, ... }` event.
- What happens when a community ID exceeds 11 (the community color array length)? — The color wraps deterministically: `COMMUNITY_COLORS[community_id % COMMUNITY_COLORS.len()]`. Community 12 uses the same color as community 0.
- What happens when a partition exceeds 100 communities (the documented practical ceiling per FR-018)? — The community panel scrolls to accommodate the excess rows; color cycling continues to recycle via `community_id % COMMUNITY_COLORS.len()`. No warning, special indicator, or distinct visual treatment is shown — the design system treats this as an expected degradation mode rather than an error condition.
- How does the help overlay interact with the underlying panels? — The help overlay renders on top of all panels with `BG_1` background, does not dismiss the panels, and any key press closes the overlay.
- What happens when all panels are hidden (both `show_graph = false` and `show_log = false`)? — The community panel occupies the full screen (it cannot be hidden), plus the status bar.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST define a color palette module (`colors.rs`) containing all color constants as `pub const` values of type `ratatui::style::Color`, organized in four layers: Base (5 background colors), Text (4 foreground colors), Accent (5 semantic colors), and Community (12 hash colors).
- **FR-002**: The system MUST provide a `pub const fn` style preset for each recurring visual pattern: focused border, unfocused border, focused title, unfocused title, column header, selected row, normal row, key hint, and key letter — 9 base style presets. Five additional log severity style presets (`log_error_style`, `log_warn_style`, `log_info_style`, `log_debug_style`, `log_trace_style`) are covered by FR-008. All 14 style presets MUST be `const fn` per Constitution §II.
- **FR-003**: The system MUST map each `AppState` variant (`Idle`, `Running`, `Done`, `Error`) to a unique combination of color, Unicode indicator symbol, and text label, ensuring state is distinguishable by both color and symbol.
- **FR-004**: The system MUST provide a responsive layout engine with four breakpoints (`Full` ≥120 cols, `Compact` 80–119 cols, `Stacked` 60–79 cols, `Minimal` <60 cols) that adjusts panel arrangement without content overflow or horizontal scrolling.
- **FR-005**: The system MUST support panel toggling via `g` (graph) and `l` (log) keys, redistributing space among remaining visible panels according to the toggle matrix below. The community panel is permanent and cannot be hidden:

  | Layout Mode | `show_graph` | `show_log` | Redistribution |
  |---|---|---|---|
  | Full/Compact (≥80 cols) | `false` | `true` | Community full-width top, log pane bottom |
  | Full/Compact (≥80 cols) | `true` | `false` | Community + graph side-by-side, full height |
  | Full/Compact (≥80 cols) | `false` | `false` | Community full screen + status bar |
  | Stacked (60–79 cols) | `false` | `true` | Community (60%) → Log (40%) vertical stack |
  | Stacked (60–79 cols) | `true` | `false` | Community (50%) → Graph (50%) vertical stack |
  | Stacked (60–79 cols) | `false` | `false` | Community full screen + status bar |
  | Minimal (<60 cols) | either | either | Only focused panel visible; `g`/`l` cycles focus; community panel is always fallback |

  See `design-system.md §3.5` for the complete toggle matrix.
- **FR-006**: The system MUST provide a focus cycle (`Tab` key) across all visible panels, with the focused panel indicated by border color change from `FG_3` to `ACCENT_PRIMARY`, and the cycle skipping hidden panels.
- **FR-007**: The system MUST assign community colors deterministically using the formula `COMMUNITY_COLORS[community_id as usize % COMMUNITY_COLORS.len()]`, ensuring the same community ID always maps to the same color across the community panel and graph view.
- **FR-008**: The system MUST style log events by severity level with designated color and modifier: `ERROR` in `ACCENT_ERROR` bold, `WARN` in `ACCENT_WARNING` bold, `INFO` in `ACCENT_INFO` normal, `DEBUG` in `FG_2` normal, `TRACE` in `FG_3` dim.
- **FR-009**: The system MUST render a progress gauge in the status bar during `Running` state, displaying the ratio of `iteration / iteration_cap` and the current quality value.
- **FR-010**: The system MUST use `BorderType::Rounded` for all panel borders, using Unicode rounded box-drawing characters (`╭╮╰╯`).
- **FR-011**: The system MUST format numeric values consistently: modularity to 4 decimal places, ΔQ to 4 decimal places with sign, node/edge counts with comma-separated thousands, iteration count as `current/cap`, and γ to 2 decimal places.
- **FR-012**: The system MUST ensure all foreground-background color pairs meet a minimum WCAG AA contrast ratio of 4.5:1, verified for all documented pairings.
- **FR-013**: The system MUST detect true-color support via `COLORTERM` environment variable (primary) and `TERM` heuristics (fallback for Alacritty, WezTerm, `-direct` suffix), defaulting to conservative no-true-color assumption.
- **FR-014**: The system MUST provide a 16-color ANSI fallback mapping for all design system colors when true-color is unavailable, maintaining legibility and state distinguishability.
- **FR-015**: The system MUST render selected community rows using explicit color tokens (`BG_3` background, `FG_0` foreground, bold modifier) rather than `Modifier::REVERSED`, ensuring consistent behavior across terminal palettes.
- **FR-016**: The system MUST avoid `Modifier::ITALIC` entirely, using `Modifier::DIM` or `Modifier::UNDERLINED` instead for universal terminal compatibility.
- **FR-017**: The system MUST render a help overlay (triggered by `?` key) displaying all key bindings organized into labeled groups (e.g., "Navigation", "Panels", "General"), dismissible by any key press, with `BG_1` background and `ACCENT_PRIMARY` border.
- **FR-018**: The system MUST support a practical maximum of 100 communities before visual degradation is accepted (panel scrolls, color cycling via `community_id % COMMUNITY_COLORS.len()` continues to recycle, no special warning or distinct visual handling beyond the standard scrolling behavior). Partitions exceeding 100 communities remain functional but are explicitly deferred from any specialized design treatment.

### Key Entities

- **Color Palette**: The complete set of named color constants organized in Base, Text, Accent, and Community layers, defined as Ratatui `Color::Rgb` values with 256-color and 16-color fallbacks.
- **Style Preset**: A named, `const fn`-defined `ratatui::style::Style` combining foreground color, optional background color, and modifier flags, used consistently across all widgets.
- **Layout Mode**: One of four terminal-width-dependent arrangements (`Full`, `Compact`, `Stacked`, `Minimal`) determining panel size and orientation.
- **Focus State**: The currently focused panel in the Tab cycle, indicated by an `ACCENT_PRIMARY` border, receiving exclusive keyboard input.
- **State Theme**: The mapping from `AppState` variant to semantic color, Unicode indicator symbol, and text label for the status bar.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All four `AppState` variants are visually distinguishable within 1 second of glancing at the status bar, confirmed by distinct color + symbol combinations (color alone is never the sole differentiator).
- **SC-002**: All 12 community colors are perceptually distinguishable when displayed simultaneously in the community panel and graph view.
- **SC-003**: The TUI renders without content overflow or horizontal scrolling at all four layout breakpoints (≥120, 80–119, 60–79, <60 columns) at a minimum height of 24 rows.
- **SC-004**: All documented foreground-background color pairs achieve a minimum WCAG AA contrast ratio of 4.5:1 (verified against the design system's color values).
- **SC-005**: The TUI remains fully operable and legible in 16-color ANSI mode, with all states and severity levels distinguishable.
- **SC-006**: Keyboard-only navigation (Tab, arrow keys, shortcut keys) covers 100% of user interactions — no feature requires a mouse.
- **SC-007**: The render loop completes each frame in under 50 ms, including channel drain and widget rendering, at 20 FPS poll rate.
- **SC-008**: Number formatting is consistent across all widgets: modularity to 4 decimal places, counts with thousands separators, iterations as `current/cap`.
- **SC-009**: No emoji characters appear anywhere in the rendered TUI output — only Unicode box-drawing, symbol, and Greek characters from the documented character set.
- **SC-010**: Partitions with up to 100 communities render fully in the community panel without scrolling. Partitions exceeding 100 communities remain visually functional (color cycling + panel scrolling) with no requirement for additional warning UI or virtualization beyond the standard scrolling behavior.

## Assumptions

- **Target audience**: Rust developers and graph researchers familiar with terminal-based tools (`htop`, `k9s`, `lazygit`). They prefer information density over visual simplicity.
- **Terminal environment**: Users operate modern terminals (kitty, alacritty, iTerm2, WezTerm, Windows Terminal, GNOME Terminal) that support true-color and Unicode BMP characters. CI and SSH environments may have reduced capabilities.
- **Dark background**: The terminal background is dark (close to `#1a1b26`). The design system does not support light terminal backgrounds.
- **Ratatui version**: The design system targets Ratatui 0.30.2 APIs (pinned per Constitution §VII). `const fn` style constructors and `BorderType::Rounded` are available in this version.
- **Single theme**: Only one visual theme (dark, Tokyo Night-inspired) is shipped in v1. A theme system or user-configurable colors are out of scope.
- **No animation**: All state transitions are immediate. No easing curves, transition animations, or frame interpolation are used. The render loop redraws the entire frame on each tick.
- **Existing architecture**: The design system is implemented as modules within the `leiden-tui` crate (e.g., `src/ui/colors.rs`, `src/ui/styles.rs`), not as a separate library crate, since it has a single consumer. It integrates with the existing crate structure as defined in `plan.md §Project Structure`, specifically the `src/ui/` module directory and the `App`/`AppState` types from `data-model.md §3.4`.
- **Constitution compliance**: The `colors.rs` module must carry `///` doc comments on every `pub` item (Constitution §IV, `missing_docs = deny`) and all style presets must be `const fn` to satisfy the strict lint profile (Constitution §II).
