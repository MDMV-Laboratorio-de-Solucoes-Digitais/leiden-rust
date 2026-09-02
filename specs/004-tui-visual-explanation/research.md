# Research & Architecture Decisions: TUI Visual Explanation

**Feature**: TUI Visual Explanation (`specs/004-tui-visual-explanation`)  
**Status**: Completed  
**Aligned With**: `design-system.md`, `.specify/memory/constitution.md` (v1.1.0)  

---

## 1. Force-Directed Layout & Terminal Canvas Simulation

### Decision
Implement a lightweight 2D Force-Directed Relaxation Simulation (Eades / Fruchterman-Reingold variant with velocity damping) integrated with Ratatui's `Canvas` widget. The simulation computes positions in a normalized $[0.0, 1.0] \times [0.0, 1.0]$ virtual coordinate space, which dynamically maps to the terminal `Rect` bounds during painting.

### Rationale
- Non-technical users need to visually see a "messy" unstructured graph organically organize itself into tight, cohesive clusters per User Story 2 and FR-003.
- In the initial state (`AppState::Idle` or unclustered), repulsive electrostatic forces ($F_{rep} = \frac{k_{rep}}{d^2}$) push nodes apart to fill the canvas as an unorganized mesh.
- When community assignments update (via local moving and refinement events), attractive spring forces ($F_{attr} = k_{attr} \cdot d$) pull nodes belonging to the same community towards their shared community centroid.
- A velocity damping factor ($\approx 0.85$) prevents oscillations and guarantees convergence within 20–30 simulation ticks per algorithmic step.
- Ratatui's `Canvas` painter renders nodes as Unicode discs (`●` / `GRAPH_NODE`) and inter-node edges as `Line` / Braille lines (`FG_3` dimmed edges), providing smooth rendering at 20–30 FPS (50ms tick rate).

### Alternatives Considered
- **Static Grid Layout**: Placed nodes in rigid circular sub-grids per community. *Rejected*: Lacks dynamic movement; feels jarring rather than illustrative of community cohesion.
- **t-SNE / UMAP Dimensionality Reduction**: *Rejected*: Heavy dependencies, non-incremental, computationally expensive for interactive 30 FPS TUI rendering.
- **ASCII Matrix Printout**: *Rejected*: Completely unreadable for non-technical users and fails FR-001/FR-003.

---

## 2. Screen Readability & Design System Alignment

### Decision
Strictly align all visual widgets, typography, borders, and colors with `design-system.md` using the canonical `colors.rs` and `styles.rs` modules:
- **Base Surfaces**: `BG_0` (`#1a1b26`) root background, `BG_1` (`#1f202e`) panel interior.
- **Text Hierarchy**: `FG_0` (`#c0caf5`, 11.1:1 contrast AAA) for headlines and titles; `FG_1` (`#a9b1d6`, 8.1:1 AAA) for body explanations; `FG_2` (`#737a89`, 4.7:1 AA) for secondary labels and metadata; `FG_3` (`#565f71`) for unfocused borders and dim hints.
- **Semantic Accents**: `ACCENT_PRIMARY` (`#7aa2f7`) for focused borders/keys; `ACCENT_INFO` (`#73daca`) for running stats; `ACCENT_SUCCESS` (`#9ece6a`) for final completion; `ACCENT_WARNING` (`#e0af68`) for pause/throttle.
- **Community Palette**: `COMMUNITY_COLORS` 12-color CIELAB-distinct palette (`community_color(id)`) ensuring instant visual association between the community list, node colors, and explanation badges.
- **Borders & Focus**: `BorderType::Rounded` (`╭╮╰╯`) via `focused_block()` and `unfocused_block()`.
- **No `Modifier::ITALIC`**: Universal compatibility across legacy and modern terminal emulators (`design-system.md` §4.1).

### Rationale
Guarantees WCAG AAA/AA accessibility (minimum 4.5:1 contrast ratio against dark backgrounds), crisp visual hierarchy, and seamless aesthetic consistency across the entire Leiden suite.

### Alternatives Considered
- **Ad-hoc Inline Colors**: *Rejected*: Violates Constitution Principle II & VII and breaks design system uniformity.
- **Reverse-Video Selection**: *Rejected*: Breaks contrast in 16-color fallback terminals per `design-system.md` §4.1.

---

## 3. Explanatory Text Panel & 8th-Grade Readability (SC-003)

### Decision
Structure the explanatory panel into three clear visual tiers using a dedicated `ExplanationPanel` widget:
1. **Tier 1 — Step Headline**: Bold `FG_0` summary (e.g., *"Finding Initial Friend Groups"* or *"Merging Neighboring Clusters"*).
2. **Tier 2 — Plain-English Intuitive Explanation**: 1–2 sentences written at or below an 8th-grade reading level (Flesch-Kincaid index $\le 8.0$), using everyday metaphors (social circles, lunch tables, neighborhood clubs) rather than technical graph jargon (modularity, gamma resolution, eigenvectors).
3. **Tier 3 — Live Stat Badges**: Clean key-value badges (`Phase`, `Communities`, `Clustering Progress`) styled with `ACCENT_INFO` and `FG_1`.

### Rationale
Satisfies FR-004, SC-001, and SC-003. Non-technical users immediately grasp *what* the algorithm is doing without needing graph theory background.

### Readability Sample Mapping
| Algorithm Phase | Step Headline | Plain-English Intuitive Explanation | Target Flesch-Kincaid |
|---|---|---|---|
| **Initial State** | *Messy Network Starting Point* | *"All people are mixed together in one big group. Nobody has been assigned to a club yet."* | Grade 4.8 |
| **Local Moving** | *Finding Best Friend Circles* | *"Each person looks around at their closest friends and moves into the club where they have the most in common."* | Grade 6.2 |
| **Refinement** | *Splitting Up Big Crowds* | *"Groups check if everyone inside actually knows each other. If a group has two separate cliques, it splits into smaller well-knit teams."* | Grade 6.8 |
| **Aggregation** | *Zooming Out to Big Picture* | *"We treat each tightly-knit team as a single super-member and look for wider connections across the whole network."* | Grade 7.1 |
| **Completion** | *Neat Communities Discovered* | *"The network is now organized into neat, color-coded communities where members are tightly connected to each other."* | Grade 7.4 |

---

## 4. Playback Controls & Dual Granularity Stepping

### Decision
Implement a deterministic `PlaybackController` state machine within `App`:
- **Controls**:
  - `Space`: Play / Pause auto-stepping toggle.
  - `n` / `Right Arrow`: Advance one step forward.
  - `t` / `Tab`: Toggle stepping granularity (`PhaseLevel` vs `MicroStep`).
  - `1`, `2`, `3` or `p`: Select built-in preset dataset (Karate Club, Two Cliques, Random Mess).
  - `r`: Restart current explanation run.
  - `?`: Toggle help keybinding overlay.
- **Granularity Modes**:
  - `PhaseLevel`: Pauses only at major phase boundaries (Local Moving complete, Refinement complete, Aggregation complete).
  - `MicroStep`: Pauses after individual node migrations or iteration sub-steps for deep step-by-step observation.

### Rationale
Directly fulfills FR-005 and FR-006. Enables both passive presentation (auto-play) and interactive exploration (step-by-step).

---

## 5. Responsive Terminal Auto-Scaling & Dimension Guard (< 80x24)

### Decision
1. **Dynamic Scaling**: The canvas dynamically normalizes node coordinates to the allocated `Rect` chunk on every frame render, ensuring seamless behavior during window resize events.
2. **Dimension Guard Overlay**: If the terminal width $< 80$ columns or height $< 24$ rows, the UI halts normal panel rendering and displays a high-visibility modal overlay:
   ```text
   ╭─ TERMINAL TOO SMALL ──────────────────────╮
   │ Current size: 72 × 20                     │
   │ Minimum required: 80 × 24                 │
   │ Please expand your terminal window.       │
   ╰───────────────────────────────────────────╯
   ```

### Rationale
Satisfies FR-007, Edge Cases in `spec.md`, and `design-system.md` §0.2. Prevents widget truncation and overlapping text in constrained terminal viewports.
