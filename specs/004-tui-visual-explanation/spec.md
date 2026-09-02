# Feature Specification: TUI Visual Explanation

**Feature Branch**: `[004-tui-visual-explanation]`

**Created**: 2026-09-01

**Status**: Draft

**Input**: User description: "I ran the TUI, but I didn't understand nothing that was being shown. Please make it a dumb person like me can understand what's going on. I want the TUI to show a messy graph turning into a neat coese community."

## Clarifications

### Session 2026-09-01

- Q: How should the graph visually transform on the terminal canvas as communities are detected? → A: Dynamic force-directed layout where nodes physically move across the canvas to pull closer to community members and repel non-members step-by-step.
- Q: What playback controls and stepping granularity should the TUI provide for navigating the explanation? → A: Auto-Play + Phase Stepping with a Dual Granularity toggle (Spacebar for Play/Pause, `n`/Right Arrow to advance, `t` or toggle between major phases and fine-grained micro-steps).
- Q: How should the TUI provide datasets for users to explore the visual explanation? → A: Built-in curated demo presets (e.g. Karate Club, Two Cliques, Random Mess) selectable in-app, plus support for custom dataset file paths via CLI argument.
- Q: How should the explanatory panel present information to guide the user through each phase? → A: Structured 3-part panel with a Step Headline, Plain-English Intuitive Explanation (everyday analogy/context), and Simple Live Stats (community count, phase name).
- Q: How should the TUI handle terminal resizing and small screen dimensions? → A: Responsive auto-scaling (coordinates dynamically normalize to canvas dimensions) with a "Terminal too small (minimum 80x24)" pause overlay when below minimum size.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Understand the Initial State (Priority: P1)

A non-technical user starts the TUI (with or without a custom file argument) and sees a visual representation of a "messy" graph or can choose a built-in demo preset. There is clear, simple text explaining what they are looking at.

**Why this priority**: Essential context for understanding the before-and-after transformation.

**Independent Test**: Run the TUI and verify the initial screen displays the unclustered graph and introductory text.

**Acceptance Scenarios**:

1. **Given** the TUI is launched without arguments, **When** the initial view loads, **Then** the user can choose from built-in demo presets (e.g. Karate Club, Two Cliques) or start with a default messy demo graph.
2. **Given** the TUI is launched with a custom dataset file, **When** the initial view loads, **Then** the user sees their graph in an unclustered layout and text explaining this is the starting "messy" state.

---

### User Story 2 - Watch the Graph Transform (Priority: P1)

The user watches the graph transform via auto-play or manual stepping, seeing nodes physically move across the canvas via a force-directed layout to cluster into cohesive communities while changing colors. The user can toggle between high-level phase steps and node-level micro-steps. Explanatory text updates to describe what is happening.

**Why this priority**: This is the core requirement to make the algorithm understandable to a layperson.

**Independent Test**: Trigger the algorithm execution in the TUI and verify the graph physically repositions nodes into clusters, updates colors, and changes text explanations alongside each step.

**Acceptance Scenarios**:

1. **Given** the TUI is in the initial state, **When** the user advances the process (either playing or stepping), **Then** nodes dynamically migrate across the 2D canvas toward their assigned community cluster, adopt distinct community colors, and simple text explains the grouping action.
2. **Given** the user wants more or less detail, **When** the granularity toggle is activated, **Then** manual stepping switches between high-level phase jumps and detailed node-by-node moves.

---

### User Story 3 - View the Final Communities (Priority: P2)

The user sees the final result where the messy graph has become a set of neat, color-coded communities, along with a summary of what was found.

**Why this priority**: Completes the narrative of the algorithm's purpose.

**Independent Test**: Allow the algorithm to finish and verify the final screen clearly shows distinct communities and a summary message.

**Acceptance Scenarios**:

1. **Given** the algorithm is running, **When** it completes, **Then** the TUI displays the fully clustered graph and a simple summary like "Found 5 cohesive communities".

### Edge Cases

- **Small Terminal Dimensions (< 80x24)**: The TUI pauses rendering and displays an informative full-screen overlay ("Terminal too small — please resize to at least 80x24") until window dimensions are expanded.
- **Terminal Resize During Playback**: Graph canvas boundaries and force simulation coordinates dynamically re-normalize to the new terminal geometry without losing playback state or restarting the algorithm.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The TUI MUST render a visual representation of the network graph (nodes and edges).
- **FR-002**: The initial state MUST display the graph without community assignments (e.g., all one color) to represent the "messy" starting point.
- **FR-003**: The TUI MUST visually update the graph representation using a 2D force-directed layout simulation where nodes physically move across the canvas to cluster with their assigned community and update their colors as the algorithm progresses step-by-step.
- **FR-004**: The TUI MUST display an explanatory text panel structured in 3 sections: (1) Step Headline, (2) Plain-English Intuitive Explanation (explaining what is happening using simple, non-technical analogies), and (3) Simple Live Stats (current phase name, active community count).
- **FR-005**: The TUI MUST provide playback controls including Auto-Play toggle (Play/Pause via Spacebar), Manual Step Forward (`n` or Right Arrow), and a Granularity Mode Toggle (switching between major phase-level stepping and node-level micro-stepping).
- **FR-006**: The TUI MUST include built-in curated demo datasets (including Zachary's Karate Club, Two Cliques, and a Random Messy Graph) selectable in-app, while accepting optional dataset file paths via CLI invocation.
- **FR-007**: The TUI MUST dynamically adapt graph rendering to terminal window resize events by re-normalizing layout coordinates, and MUST display a pause overlay requiring at least 80x24 columns/rows if the terminal is below minimum supported dimensions.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A non-technical user can successfully describe the basic purpose of the algorithm (finding communities) after interacting with the TUI.
- **SC-002**: The TUI can render and animate a graph of at least 50 nodes and 100 edges smoothly at 20–30 FPS (standard 50ms event tick loop / 20 FPS, with frame computation budget ≤ 16ms).
- **SC-003**: The explanatory text in the TUI is written at an 8th-grade reading level (Flesch-Kincaid readability test).

## Assumptions

- Users have a terminal emulator capable of displaying standard TUI graphics (e.g., block characters, colors).
- The existing Leiden algorithm implementation can provide intermediate state updates to the UI.
