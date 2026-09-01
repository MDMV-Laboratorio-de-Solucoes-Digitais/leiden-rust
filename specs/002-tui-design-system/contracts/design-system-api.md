# Design System API Contract

**Branch**: `[002-tui-design-system]` | **Date**: 2026-09-01
**Output of**: `$speckit-plan` Phase 1 (Design & Contracts)

This contract defines the public API surface of the `leiden-tui` design system modules
(`src/ui/colors.rs` and `src/ui/styles.rs`). All items listed below are `pub` within the
`leiden-tui` crate. Since the design system has a single consumer (the `leiden-tui` binary),
these items are crate-internal and do not appear in an external library API.

---

## 1. `colors.rs` — Color Constants & Utilities

### 1.1 Exported Constants

| Constant | Type | Value | Description |
|---|---|---|---|
| `BG_0` | `Color` | `Rgb(26, 27, 38)` | Terminal background |
| `BG_1` | `Color` | `Rgb(31, 32, 46)` | Elevated surface |
| `BG_2` | `Color` | `Rgb(36, 38, 58)` | Status bar background |
| `BG_3` | `Color` | `Rgb(42, 45, 66)` | Selection background |
| `BG_4` | `Color` | `Rgb(59, 63, 88)` | Progress gauge empty |
| `FG_0` | `Color` | `Rgb(192, 202, 245)` | Bright foreground |
| `FG_1` | `Color` | `Rgb(169, 177, 214)` | Normal foreground |
| `FG_2` | `Color` | `Rgb(115, 122, 137)` | Muted foreground |
| `FG_3` | `Color` | `Rgb(86, 95, 113)` | Dim foreground |
| `ACCENT_PRIMARY` | `Color` | `Rgb(122, 162, 247)` | Focus/active blue |
| `ACCENT_INFO` | `Color` | `Rgb(115, 218, 202)` | Running/info teal |
| `ACCENT_WARNING` | `Color` | `Rgb(224, 175, 104)` | Warning amber |
| `ACCENT_ERROR` | `Color` | `Rgb(247, 118, 125)` | Error red |
| `ACCENT_SUCCESS` | `Color` | `Rgb(158, 206, 106)` | Done/success green |
| `COMMUNITY_COLORS` | `[Color; 12]` | See data-model §1.4 | Community hash palette |

### 1.2 ANSI Fallback Constants

| Constant | Type | Value | Maps From |
|---|---|---|---|
| `BG_0_ANSI` | `Color` | `Color::Black` | `BG_0` |
| `BG_1_ANSI` | `Color` | `Color::Black` | `BG_1` |
| `BG_2_ANSI` | `Color` | `Color::DarkGray` | `BG_2` |
| `BG_3_ANSI` | `Color` | `Color::DarkGray` | `BG_3` |
| `BG_4_ANSI` | `Color` | `Color::DarkGray` | `BG_4` |
| `FG_0_ANSI` | `Color` | `Color::White` | `FG_0` |
| `FG_1_ANSI` | `Color` | `Color::Gray` | `FG_1` |
| `FG_2_ANSI` | `Color` | `Color::DarkGray` | `FG_2` |
| `FG_3_ANSI` | `Color` | `Color::DarkGray` | `FG_3` |
| `ACCENT_PRIMARY_ANSI` | `Color` | `Color::Blue` | `ACCENT_PRIMARY` |
| `ACCENT_INFO_ANSI` | `Color` | `Color::Cyan` | `ACCENT_INFO` |
| `ACCENT_WARNING_ANSI` | `Color` | `Color::Yellow` | `ACCENT_WARNING` |
| `ACCENT_ERROR_ANSI` | `Color` | `Color::Red` | `ACCENT_ERROR` |
| `ACCENT_SUCCESS_ANSI` | `Color` | `Color::Green` | `ACCENT_SUCCESS` |
| `COMMUNITY_COLORS_ANSI` | `[Color; 6]` | `[Blue, Green, Yellow, Red, Cyan, Magenta]` | `COMMUNITY_COLORS` |

### 1.3 Exported Functions

#### `community_color`

```rust
/// Get a deterministic color for a community id.
#[must_use]
pub const fn community_color(community_id: u32) -> Color;
```

**Contract**:
- Returns `COMMUNITY_COLORS[community_id as usize % COMMUNITY_COLORS.len()]`.
- Same `community_id` always maps to same color (deterministic, FR-007).
- `community_color(0)` == `COMMUNITY_COLORS[0]`.
- `community_color(12)` == `COMMUNITY_COLORS[0]` (wraps).

#### `supports_truecolor`

```rust
/// Detect true-color support from environment variables.
#[must_use]
pub fn supports_truecolor() -> bool;
```

**Contract**:
- Returns `true` if `COLORTERM` is `"truecolor"` or `"24bit"` (FR-013).
- Returns `true` if `TERM` starts with `"alacritty"` or `"wezterm"`, or ends with `"-direct"`.
- Returns `false` otherwise (conservative default).
- Does not panic. Uses `if let Ok(...)` pattern for `env::var()`.

#### `resolve_color`

```rust
/// Select the appropriate color representation at runtime.
///
/// Returns the true-color RGB value when [`supports_truecolor()`] returns `true`,
/// otherwise returns the ANSI fallback color (FR-013, FR-014, data-model §2.1).
#[must_use]
pub fn resolve_color(color: Color, ansi: Color) -> Color {
    if supports_truecolor() {
        color
    } else {
        ansi
    }
}
```

**Contract**:
- When `supports_truecolor()` returns `true`, returns the `color` argument (RGB).
- When `supports_truecolor()` returns `false`, returns the `ansi` argument.
- Does not panic. Calls `supports_truecolor()` which itself never panics.
- All widget color selection MUST use `resolve_color()` to choose between true-color
  and ANSI representations — never hardcode `Color::Rgb` without a fallback path.

---

## 2. `styles.rs` — Style Presets & Theme Helpers

### 2.1 Style Preset Functions (`const fn`)

All style presets are `pub const fn` returning `Style`.

| Function | Foreground | Background | Modifier | Usage |
|---|---|---|---|---|
| `focused_border_style()` | `ACCENT_PRIMARY` | — | — | Focused panel border |
| `unfocused_border_style()` | `FG_3` | — | — | Unfocused panel border |
| `title_style_focused()` | `FG_0` | — | `BOLD` | Focused panel title |
| `title_style_unfocused()` | `FG_2` | — | — | Unfocused panel title |
| `header_style()` | `FG_1` | — | `BOLD` | Table column headers |
| `selected_row_style()` | `FG_0` | `BG_3` | `BOLD` | Selected table row |
| `normal_row_style()` | `FG_1` | — | — | Unselected table row |
| `key_hint_style()` | `FG_3` | — | `DIM` | Status bar key hints |
| `key_letter_style()` | `FG_2` | — | — | Key letters in hints |
| `log_error_style()` | `ACCENT_ERROR` | — | `BOLD` | `[ERROR]` prefix |
| `log_warn_style()` | `ACCENT_WARNING` | — | `BOLD` | `[WARN]` prefix |
| `log_info_style()` | `ACCENT_INFO` | — | — | `[INFO]` prefix |
| `log_debug_style()` | `FG_2` | — | — | `[DEBUG]` prefix |
| `log_trace_style()` | `FG_3` | — | `DIM` | `[TRACE]` prefix |

### 2.2 State Theme Functions (non-`const fn`)

These functions take `&AppState` and cannot be `const fn` because `AppState` is not a `const`-compatible type (contains `String` variant).

| Function | Signature | Returns |
|---|---|---|
| `state_color` | `fn(&AppState) -> Color` | Semantic color for the state |
| `state_indicator` | `fn(&AppState) -> &'static str` | Unicode indicator symbol |
| `state_label` | `fn(&AppState) -> &'static str` | Human-readable label |

**State mapping contract**:

| `AppState` | `state_color` | `state_indicator` | `state_label` |
|---|---|---|---|
| `Idle` | `FG_2` | `"○"` | `"Idle"` |
| `Running { .. }` | `ACCENT_INFO` | `"●"` | `"Running"` |
| `Done { .. }` | `ACCENT_SUCCESS` | `"✓"` | `"Done"` |
| `Error(_)` | `ACCENT_ERROR` | `"✗"` | `"Error"` |

### 2.3 Layout Functions

#### `layout_mode`

```rust
/// Determine layout mode from terminal width.
#[must_use]
pub const fn layout_mode(width: u16) -> LayoutMode;
```

**Contract**:

| Width Range | Returns |
|---|---|
| `≥ 120` | `LayoutMode::Full` |
| `80..=119` | `LayoutMode::Compact` |
| `60..=79` | `LayoutMode::Stacked` |
| `_ (< 60)` | `LayoutMode::Minimal` |

### 2.4 Block Builder Functions (non-`const fn`)

These functions return `Block<'_>` which requires runtime `format!()` for title strings.

| Function | Signature | Description |
|---|---|---|
| `focused_block` | `fn(&str) -> Block<'_>` | Accent border + bold title |
| `unfocused_block` | `fn(&str) -> Block<'_>` | Dim border + muted title |
| `panel_block` | `fn(&str, bool) -> Block<'_>` | Dispatches based on `is_focused` |

### 2.5 Exported Types

#### `LayoutMode`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    Full,
    Compact,
    Stacked,
    Minimal,
}
```

### 2.6 Unicode Symbol Constants

| Constant | Type | Value | Codepoint |
|---|---|---|---|
| `INDICATOR_IDLE` | `&str` | `"○"` | U+25CB |
| `INDICATOR_RUNNING` | `&str` | `"●"` | U+25CF |
| `INDICATOR_DONE` | `&str` | `"✓"` | U+2713 |
| `INDICATOR_ERROR` | `&str` | `"✗"` | U+2717 |
| `GRAPH_NODE` | `&str` | `"●"` | U+25CF |
| `SORT_DESC` | `&str` | `"▼"` | U+25BC |
| `SEPARATOR_DOT` | `&str` | `"·"` | U+00B7 |
| `ARROW_RIGHT` | `&str` | `"→"` | U+2192 |
| `GAMMA` | `&str` | `"γ"` | U+03B3 |
| `DELTA` | `&str` | `"Δ"` | U+0394 |

---

## 3. Invariants

1. **No `Modifier::ITALIC`** — the design system never produces a `Style` containing `Modifier::ITALIC` (FR-016).
2. **No `Modifier::REVERSED`** — selection is always `BG_3` + `FG_0` + `BOLD`, never reverse-video (FR-015).
3. **No emoji** — all symbols are Unicode BMP characters from the documented set (SC-009).
4. **Deterministic community colors** — `community_color(id)` is a pure function of `id` with no hidden state (FR-007).
5. **All `const fn` presets** — the 14 style preset functions compile as `const fn` and can be evaluated at compile time.
6. **WCAG AA compliance** — all documented fg/bg pairings achieve ≥ 4.5:1 contrast ratio (FR-012).
7. **No I/O in style presets** — `supports_truecolor()` is the only function that reads environment variables; all other functions are pure.
8. **Runtime color resolution** — all widget color selection MUST use `resolve_color()` to choose between true-color and ANSI fallback representations; never hardcode `Color::Rgb` without a fallback path (FR-013, FR-014).
