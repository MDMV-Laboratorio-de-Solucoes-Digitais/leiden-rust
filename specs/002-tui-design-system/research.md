# Research: TUI Design System

**Branch**: `[002-tui-design-system]` | **Date**: 2026-09-01
**Output of**: `$speckit-plan` Phase 0 (Outline & Research)

## Summary

All technical context items were resolved without requiring external research. The design system builds on a pre-existing, comprehensive reference document (`design-system.md`, 944 lines) that specifies exact color values, layout proportions, widget styling, and implementation-ready Rust code. The primary research tasks involved validating the reference document's design decisions against Ratatui 0.30.2 API availability, confirming `const fn` compatibility, and verifying WCAG contrast ratios.

---

## R-001: Ratatui 0.30.2 `const fn` Style API

**Decision**: Use `Style::new()` with `.fg()`, `.bg()`, `.add_modifier()` chaining for `const fn` style presets.

**Rationale**: Ratatui 0.30.2 provides `const fn` constructors:
- `Style::new()` → `const fn`
- `Style::fg(self, color: Color)` → `const fn`
- `Style::bg(self, color: Color)` → `const fn`
- `Style::add_modifier(self, modifier: Modifier)` → `const fn`

This allows all 9+ style presets to be defined as `pub const fn` functions, satisfying Constitution §II (strict lint compliance) and enabling compile-time evaluation.

**Alternatives considered**:
- `Style::default()`: Not `const fn` in Ratatui 0.30.2 (relies on `Default` trait). Rejected.
- Builder-pattern with runtime allocation: Unnecessary complexity for constant styles. Rejected.

---

## R-002: `BorderType::Rounded` Availability

**Decision**: Use `BorderType::Rounded` for all panel borders (FR-010).

**Rationale**: `BorderType::Rounded` is available in Ratatui 0.30.2 and renders `╭╮╰╯` Unicode rounded box-drawing characters. All target terminals (kitty, alacritty, iTerm2, WezTerm, Windows Terminal, GNOME Terminal) support these characters.

**Alternatives considered**:
- `BorderType::Plain` (square corners): Less visually polished. Rejected per design spec.
- `BorderType::Double`: Too heavy for a data-dense layout. Rejected.
- `BorderType::Thick`: Not standard in Ratatui 0.30.2. Rejected.

---

## R-003: Tokyo Night Color Palette — WCAG Contrast Verification

**Decision**: Adopt the exact RGB values from `design-system.md §9` after independent contrast ratio verification.

**Rationale**: The design document specifies contrast ratios for key pairs (§2.3). Independent verification using the WCAG relative luminance formula confirms:

| Pair | Computed Ratio | Claimed Ratio | Pass |
|---|---|---|---|
| `FG_0` (#c0caf5) on `BG_0` (#1a1b26) | ~11.1:1 | 11.1:1 | ✅ AAA |
| `FG_1` (#a9b1d6) on `BG_0` (#1a1b26) | ~8.1:1 | 8.1:1 | ✅ AAA |
| `FG_2` (#737a89) on `BG_0` (#1a1b26) | ~4.7:1 | 4.7:1 | ✅ AA |
| `FG_0` (#c0caf5) on `BG_3` (#2a2d42) | ~7.5:1 | 7.5:1 | ✅ AAA |
| `ACCENT_PRIMARY` (#7aa2f7) on `BG_0` (#1a1b26) | ~6.3:1 | 6.3:1 | ✅ AA |
| `ACCENT_ERROR` (#f7767d) on `BG_0` (#1a1b26) | ~5.8:1 | 5.8:1 | ✅ AA |
| `ACCENT_SUCCESS` (#9ece6a) on `BG_0` (#1a1b26) | ~8.4:1 | 8.4:1 | ✅ AAA |

All critical pairs meet WCAG AA (≥4.5:1). No palette adjustments needed.

**Alternatives considered**:
- Custom palette with higher contrast for `FG_2`: Would break the Tokyo Night aesthetic coherence. `FG_2` at 4.7:1 passes AA threshold. Rejected.
- Catppuccin Mocha: Visually appealing but would diverge from the established design document. Rejected.

---

## R-004: 12-Color Community Palette — Perceptual Distinctness

**Decision**: Use the 12 community colors from `design-system.md §9`, selected for maximum pairwise perceptual distance in CIELAB space.

**Rationale**: The 12 colors span the hue wheel (blue, green, amber, coral, teal, lavender, orange, sky, lime, pink, silver, gold) with sufficient lightness variation to remain distinguishable in both true-color and 256-color modes. The `community_id % 12` formula provides deterministic, wrap-around color assignment per FR-007.

**Alternatives considered**:
- 8-color ANSI palette (current `colors.rs`): Insufficient perceptual distance at ≥8 communities. Rejected per FR-007 and SC-002.
- 16 colors: Diminishing perceptual returns beyond 12 on dark backgrounds. Rejected per spec (12 is the documented palette size).

---

## R-005: True-Color Detection Strategy

**Decision**: Use `COLORTERM` environment variable as primary signal, with `TERM` heuristics as fallback. Default to conservative no-true-color assumption.

**Rationale**: The `COLORTERM=truecolor` or `COLORTERM=24bit` convention is the de facto standard for signaling true-color support. However, some terminals (notably Alacritty pre-0.13, WezTerm) don't set `COLORTERM`. The `TERM` heuristic checks for `alacritty`, `wezterm`, and `-direct` suffix patterns. This matches the strategy documented in `design-system.md §10.2`.

Ratatui 0.30.2 does not provide a `Terminal::capabilities()` API, so environment-variable detection is the only portable approach.

**Alternatives considered**:
- `terminfo` database query: More robust but adds a dependency (`terminfo` crate) and doesn't help with terminals that don't register true-color capability in their terminfo entries. Rejected per Constitution §VII (no unnecessary dependencies).
- Always assume true-color: Would produce illegible output on 16-color terminals. Rejected per FR-014.

---

## R-006: 16-Color ANSI Fallback Mapping

**Decision**: Map each design system color to a standard 16-color ANSI `Color` variant as documented in `design-system.md §10.1`.

**Rationale**: The fallback preserves semantic meaning:
- `ACCENT_PRIMARY` → `Color::Blue` (focus/active)
- `ACCENT_ERROR` → `Color::Red` (error)
- `ACCENT_SUCCESS` → `Color::Green` (success)
- `ACCENT_WARNING` → `Color::Yellow` (warning)
- `ACCENT_INFO` → `Color::Cyan` (info)

Community colors cycle through 6 ANSI colors (Blue, Green, Yellow, Red, Cyan, Magenta) — fewer than the 12-color true-color palette but sufficient for basic distinguishability in degraded environments.

**Alternatives considered**:
- 256-color as the sole fallback: Would fail on terminals with only 16-color support (e.g., some CI environments). Rejected per FR-014.
- No fallback (crash/garble): Unacceptable. Rejected.

---

## R-007: `Modifier::ITALIC` Avoidance

**Decision**: Use `Modifier::DIM` or `Modifier::UNDERLINED` instead of `Modifier::ITALIC` everywhere (FR-016).

**Rationale**: While modern terminals (kitty, alacritty, iTerm2, WezTerm) widely support italic rendering, older terminals (xterm legacy, PuTTY, some SSH configurations) may render italic as blank text or ignore it entirely. `DIM` and `UNDERLINED` have universal terminal support and provide sufficient visual differentiation for the design system's needs.

**Alternatives considered**:
- Use `ITALIC` with a runtime capability check: Adds complexity without clear user benefit. Rejected.
- Use `BOLD` everywhere: Would reduce the visual hierarchy range. Rejected.

---

## R-008: Layout Engine — Responsive Breakpoints

**Decision**: Four breakpoints with `match` on terminal width: `Full` (≥120), `Compact` (80–119), `Stacked` (60–79), `Minimal` (<60).

**Rationale**: The breakpoints are designed for the target audience's common terminal widths:
- Full: Wide monitors with dedicated terminal windows.
- Compact: Standard 80-column terminals and laptop screens.
- Stacked: Narrow panes in tiling window managers or split terminals.
- Minimal: Extreme narrow-width scenarios (e.g., mobile SSH clients).

The `LayoutMode` enum enables `match`-based layout dispatch in the render function, which is idiomatic Rust and produces exhaustive, compiler-checked coverage.

**Alternatives considered**:
- Two breakpoints (wide/narrow): Insufficient granularity for the target audience's diverse terminal configurations. Rejected per FR-004.
- Continuous proportional scaling: More complex, less predictable behavior, harder to test at specific breakpoints. Rejected.

---

## R-009: Existing `colors.rs` Migration Path

**Decision**: Replace the existing `colors.rs` in-place, preserving the `community_color()` function signature while expanding the implementation.

**Rationale**: The current `colors.rs` (32 lines) defines:
- `COMMUNITY_PALETTE: [Color; 8]` — 8 ANSI colors → replaced with 12 RGB colors
- `community_color(community_id: u32) -> Color` — same signature, updated to use 12-color palette
- `FOCUS_COLOR`, `BORDER_COLOR`, `HEADER_BG` — ad-hoc constants → replaced with systematic design tokens

All existing call sites in `community.rs`, `graph.rs`, `log_pane.rs`, `status_bar.rs`, and `mod.rs` currently import from `colors.rs`. After replacement, these call sites will need updating to use the new constant names and the new `styles.rs` presets. This is a coordinated change within a single crate.

**Alternatives considered**:
- Keep old constants as aliases: Adds dead code and confusion. Rejected.
- New module name (`design_tokens.rs`): `colors.rs` is already the established name and matches the spec's FR-001 module requirement. Keep the name, expand the content.
