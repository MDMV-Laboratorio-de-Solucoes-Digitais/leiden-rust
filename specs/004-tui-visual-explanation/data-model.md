# Data Model: TUI Visual Explanation

**Feature**: TUI Visual Explanation (`specs/004-tui-visual-explanation`)  
**Status**: Completed  
**Aligned With**: `design-system.md`, `.specify/memory/constitution.md` (v1.1.0)  

---

## 1. Deep Module Architecture & Seams

The visual explanation architecture is decomposed into four deep modules, each hiding internal mathematical or linguistic complexity behind minimal, testable interfaces:

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                           leiden-tui App State                          │
└──────────────┬───────────────────┬───────────────────┬──────────────────┘
               │                   │                   │
    [ Physics-Seam ]       [ Content-Seam ]    [ Control-Seam ]
               │                   │                   │
               ▼                   ▼                   ▼
      ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
      │ ForceSimulation │ │ExplanationState │ │PlaybackControl  │
      │ • Spring-charge │ │ • Flesch-Kincaid│ │ • Granularity   │
      │ • Centroid pull │ │ • Jargon filter │ │ • State machine │
      │ • Damping/clamp │ │ • Text wrapper  │ │ • Play/pause/step│
      └─────────────────┘ └─────────────────┘ └─────────────────┘
                                   │
                             [ Data-Seam ]
                                   │
                                   ▼
                          ┌─────────────────┐
                          │  PresetDataset  │
                          │ • Curated graphs│
                          │ • CLI file I/O  │
                          │ • Topology init │
                          └─────────────────┘
```

---

## 2. Domain Entities & Type Definitions

### 2.1 `Point2D` (Spatial Geometry)
Represents a continuous 2D position or displacement vector within normalized $[0.0, 1.0] \times [0.0, 1.0]$ virtual unit space.

```rust
/// A 2D spatial coordinate or displacement vector.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2D {
    /// Normalized horizontal position [0.0, 1.0]
    pub x: f64,
    /// Normalized vertical position [0.0, 1.0]
    pub y: f64,
}

impl Point2D {
    /// Create a new 2D point.
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self;

    /// Calculate Euclidean distance to another point.
    #[must_use]
    pub fn distance_to(self, other: Self) -> f64;

    /// Vector addition with scalar scaling.
    #[must_use]
    pub fn add_scaled(self, vec: Self, scalar: f64) -> Self;

    /// Clamp coordinates within a bounding box.
    #[must_use]
    pub fn clamp(self, min_x: f64, max_x: f64, min_y: f64, max_y: f64) -> Self;
}
```

---

### 2.2 `ForceSimulation` (Physics Engine for Canvas — Deep Module)
Calculates real-time 2D node positioning using spring-electrical force relaxation to visually cluster nodes into communities.

**Public Seam**: `tick()` advances physics, `screen_coordinates()` maps to terminal `Rect`.

```rust
/// State of the 2D force-directed layout simulation.
#[derive(Debug, Clone)]
pub struct ForceSimulation {
    /// Current 2D positions of all nodes in virtual [0.05, 0.95] unit space.
    pub node_positions: std::collections::HashMap<String, Point2D>,
    /// Current velocity vectors for smooth damping.
    pub node_velocities: std::collections::HashMap<String, Point2D>,
    /// Calculated target centroids for each active community.
    pub community_centroids: std::collections::HashMap<u32, Point2D>,
    /// Velocity damping coefficient (fixed: 0.85).
    pub damping: f64,
    /// Repulsion force coefficient between disjoint nodes (fixed: 0.005).
    pub repulsion_constant: f64,
    /// Attraction force coefficient toward community centroids (fixed: 0.08).
    pub attraction_constant: f64,
    /// Minimum separation distance to prevent overlap (fixed: 0.04).
    pub min_separation: f64,
}

impl ForceSimulation {
    /// Initialize simulation with deterministic seed distribution.
    #[must_use]
    pub fn new(nodes: &[String]) -> Self;

    /// Advance simulation physics by one tick based on partition assignments.
    pub fn tick(&mut self, partition: &[(String, u32)], edges: &[(String, String)]);

    /// Project normalized node positions to terminal screen Rect without modifying physics state.
    #[must_use]
    pub fn screen_coordinates(&self, area: ratatui::layout::Rect) -> std::collections::HashMap<String, (f64, f64)>;

    /// Reset simulation back to initial unorganized mesh.
    pub fn reset(&mut self, nodes: &[String]);
}
```

---

### 2.3 `ExplanationState` (3-Part Readability Content — Deep Module)
Encapsulates 3-tier plain-English explanation text, reading-level validation, and word wrapping (FR-004, SC-003).

**Public Seam**: `from_leiden_event()` updates narrative state, `wrapped_analogy_lines()` formats lines for rendering.

```rust
/// 3-part structured explanation state for non-technical users.
#[derive(Debug, Clone, PartialEq)]
pub struct ExplanationState {
    /// Tier 1: Bold summary headline (e.g., "STEP 1 OF 3: FINDING FRIEND CIRCLES")
    pub headline: String,
    /// Tier 2: Plain-English intuitive analogy (<= 8th grade reading level, <= 240 chars)
    pub analogy_text: String,
    /// Tier 3: Current algorithm phase name (e.g., "Local Moving", "Refinement")
    pub phase_name: String,
    /// Active community count detected so far
    pub community_count: usize,
    /// Percentage progress through current phase [0.0, 1.0]
    pub phase_progress: f64,
    /// Verified Flesch-Kincaid grade level score
    pub reading_grade_level: f32,
}

impl ExplanationState {
    /// Create initial unclustered explanation state.
    #[must_use]
    pub fn initial_unclustered(total_nodes: usize, total_edges: usize) -> Self;

    /// Update explanation from Leiden execution event.
    #[must_use]
    pub fn from_leiden_event(event: &leiden::events::LeidenEvent, current_communities: usize) -> Self;

    /// Create final completion summary.
    #[must_use]
    pub fn completed(community_count: usize, quality: f64) -> Self;

    /// Wrap analogy text to fit panel width (max 76 chars/line, max 3 lines, word-boundary split).
    #[must_use]
    pub fn wrapped_analogy_lines(&self, max_width: usize) -> Vec<String>;
}
```

---

### 2.4 `PresetDataset` (Curated Demo & CLI Input — Deep Module)
Encapsulates built-in demo datasets and CLI custom file ingestion (FR-006).

**Public Seam**: `get(id)` loads dataset, `from_cli_path()` parses external file, `all_presets()` lists available presets.

```rust
/// Identifier for available demo presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresetId {
    /// Zachary's Karate Club (34 nodes, 78 edges)
    KarateClub,
    /// Two interconnected cliques (16 nodes, 56 edges)
    TwoCliques,
    /// Random messy unclustered network (30 nodes, 60 edges)
    RandomMess,
    /// Custom dataset loaded from CLI file path
    Custom,
}

/// A curated demo dataset.
#[derive(Debug, Clone)]
pub struct PresetDataset {
    /// Unique preset identifier
    pub id: PresetId,
    /// Display title
    pub title: &'static str,
    /// Plain-English description
    pub description: &'static str,
    /// Node count
    pub node_count: usize,
    /// Edge count
    pub edge_count: usize,
    /// Graph edges as (source, target) pairs
    pub edges: Vec<(String, String)>,
}

impl PresetDataset {
    /// Load preset by identifier.
    #[must_use]
    pub fn get(id: PresetId) -> Self;

    /// Load custom dataset from CLI path; returns domain error if unreadable.
    pub fn from_cli_path(path: &std::path::Path) -> Result<Self, crate::error::TuiError>;

    /// List all available built-in demo presets.
    #[must_use]
    pub fn all_presets() -> Vec<Self>;
}
```

---

### 2.5 `PlaybackController` (Execution & Granularity Control — Deep Module)
Manages play/pause state machine, step advancement, and granularity toggling (FR-005).

**Public Seam**: `toggle_play()`, `request_step()`, `toggle_granularity()`, `on_preset_switch()`.

```rust
/// Stepping granularity mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GranularityMode {
    /// Pauses at major algorithm phases (Local Moving, Refinement, Aggregation)
    #[default]
    PhaseLevel,
    /// Pauses after individual node migrations and sub-steps
    MicroStep,
}

/// Controls interactive playback and stepping state.
#[derive(Debug, Clone)]
pub struct PlaybackController {
    /// Whether auto-play is actively running
    pub is_playing: bool,
    /// Auto-play tick speed in milliseconds (fixed: 200ms)
    pub tick_speed_ms: u64,
    /// Single manual step requested flag
    pub step_requested: bool,
    /// Active granularity mode (persists across preset switches)
    pub granularity: GranularityMode,
}

impl PlaybackController {
    /// Create default paused controller in PhaseLevel mode.
    #[must_use]
    pub fn new() -> Self;

    /// Toggle play/pause state.
    pub fn toggle_play(&mut self);

    /// Request a single step forward (auto-pauses if playing).
    pub fn request_step(&mut self);

    /// Toggle between PhaseLevel and MicroStep granularity.
    pub fn toggle_granularity(&mut self);

    /// Handle preset switch: resets state to Step 1, auto-pauses, preserves granularity.
    pub fn on_preset_switch(&mut self);
}
```

---

### 2.6 `TerminalDimensionGuard` (Screen Readability Guard)
Validates terminal geometry against minimum supported bounds (FR-007, `design-system.md` §0.2).

```rust
/// Guards layout against undersized terminal viewports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalDimensionGuard {
    /// Minimum supported terminal columns (80)
    pub min_columns: u16,
    /// Minimum supported terminal rows (24)
    pub min_rows: u16,
}

impl TerminalDimensionGuard {
    /// Standard guard with 80x24 minimum requirement.
    #[must_use]
    pub const fn standard() -> Self {
        Self {
            min_columns: 80,
            min_rows: 24,
        }
    }

    /// Check if given terminal dimensions are sufficient.
    #[must_use]
    pub const fn is_valid(&self, width: u16, height: u16) -> bool {
        width >= self.min_columns && height >= self.min_rows
    }
}
```

---

## 3. Invariant & Contract Rules

1. **CLI vs. In-App Precedence (CHK001)**: If a CLI path is passed on launch, the TUI initializes in `PresetId::Custom`. Pressing `1`, `2`, `3` immediately transitions to that built-in preset without requiring a CLI restart.
2. **Preset Switch Lifecycle (CHK024)**: Switching presets via `1`, `2`, `3` ALWAYS resets the explanation state to Step 1 and auto-pauses playback, while preserving the user's selected `GranularityMode`.
3. **Text Wrapping Invariant (CHK015)**: `ExplanationState::wrapped_analogy_lines()` wraps text at word boundaries to $\le 76$ columns (leaving 2 padding columns on each side in an 80-column panel) with a maximum of 3 lines.
4. **Resize State Preservation**: Terminal resizing recalculates only the `screen_coordinates()` mapping matrix; the underlying normalized $[0.05, 0.95]$ node coordinates, algorithm iteration state, and partition data are 100% preserved.
5. **Panic-Free Error Discipline**: All file parsing, dimension validation, and physics steps return fallible `Result<T, TuiError>` or clean fallback defaults; zero `unwrap()` or `expect()` calls exist across all modules.
