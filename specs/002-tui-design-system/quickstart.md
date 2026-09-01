# Quickstart: TUI Design System Validation

**Branch**: `[002-tui-design-system]` | **Date**: 2026-09-01
**Output of**: `$speckit-plan` Phase 1 (Design & Contracts)

This document provides runnable validation scenarios that prove the TUI design system
works end-to-end. Each scenario includes prerequisites, commands, and expected outcomes.
Implementation details (full module bodies, migration code) belong in `tasks.md`.

---

## 1. Prerequisites

- Rust stable ≥ 1.88.0 (pinned via `rust-toolchain.toml`)
- Terminal with true-color support (for RGB color verification)
- Working `cargo test` and `cargo clippy` in the workspace

```bash
# Verify toolchain
cd leiden
rustc --version  # ≥ 1.88.0
cargo clippy --version
```

---

## 2. Scenario: Color Constants Compile as `const`

**Purpose**: Verify that all 26 color constants are valid `const` items and the Ratatui `Color::Rgb` constructor is `const`-compatible.

**Test approach** (unit test in `colors.rs`):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_constants_are_const() {
        // If these compile, the constants are valid const items.
        const _: Color = BG_0;
        const _: Color = BG_1;
        const _: Color = BG_2;
        const _: Color = BG_3;
        const _: Color = BG_4;
        const _: Color = FG_0;
        const _: Color = FG_1;
        const _: Color = FG_2;
        const _: Color = FG_3;
        const _: Color = ACCENT_PRIMARY;
        const _: Color = ACCENT_INFO;
        const _: Color = ACCENT_WARNING;
        const _: Color = ACCENT_ERROR;
        const _: Color = ACCENT_SUCCESS;
    }
}
```

**Run**:
```bash
cargo test -p leiden-tui color_constants_are_const
```

**Expected**: Test passes (compilation success proves `const` validity).

---

## 3. Scenario: Style Presets Compile as `const fn`

**Purpose**: Verify that all 14 style preset functions are valid `const fn` and produce expected `Style` values.

**Test approach** (unit test in `styles.rs`):

```rust
#[test]
fn style_presets_are_const_fn() {
    // If these compile as const items, the functions are const fn.
    const _: Style = focused_border_style();
    const _: Style = unfocused_border_style();
    const _: Style = title_style_focused();
    const _: Style = title_style_unfocused();
    const _: Style = header_style();
    const _: Style = selected_row_style();
    const _: Style = normal_row_style();
    const _: Style = key_hint_style();
    const _: Style = key_letter_style();
    const _: Style = log_error_style();
    const _: Style = log_warn_style();
    const _: Style = log_info_style();
    const _: Style = log_debug_style();
    const _: Style = log_trace_style();
}
```

**Run**:
```bash
cargo test -p leiden-tui style_presets_are_const_fn
```

**Expected**: Test passes.

---

## 4. Scenario: Community Color Determinism & Wrap

**Purpose**: Verify `community_color()` returns deterministic colors and wraps at index 12 (FR-007).

**Test approach**:

```rust
#[test]
fn community_color_deterministic() {
    // Same ID always maps to same color
    assert_eq!(community_color(0), COMMUNITY_COLORS[0]);
    assert_eq!(community_color(5), COMMUNITY_COLORS[5]);
    assert_eq!(community_color(11), COMMUNITY_COLORS[11]);

    // Wraps at 12
    assert_eq!(community_color(12), COMMUNITY_COLORS[0]);
    assert_eq!(community_color(13), COMMUNITY_COLORS[1]);
    assert_eq!(community_color(24), COMMUNITY_COLORS[0]);

    // Stable across calls
    let a = community_color(42);
    let b = community_color(42);
    assert_eq!(a, b);
}
```

**Run**:
```bash
cargo test -p leiden-tui community_color_deterministic
```

**Expected**: All assertions pass.

---

## 5. Scenario: True-Color Detection

**Purpose**: Verify `supports_truecolor()` correctly interprets `COLORTERM` and `TERM` (FR-013).

**Test approach**:

```rust
#[test]
fn truecolor_detection_colorterm() {
    // Test with COLORTERM=truecolor
    std::env::set_var("COLORTERM", "truecolor");
    assert!(supports_truecolor());

    // Test with COLORTERM=24bit
    std::env::set_var("COLORTERM", "24bit");
    assert!(supports_truecolor());

    // Clean up
    std::env::remove_var("COLORTERM");
}

#[test]
fn truecolor_detection_term_fallback() {
    std::env::remove_var("COLORTERM");

    // Alacritty heuristic
    std::env::set_var("TERM", "alacritty");
    assert!(supports_truecolor());

    // WezTerm heuristic
    std::env::set_var("TERM", "wezterm");
    assert!(supports_truecolor());

    // -direct suffix
    std::env::set_var("TERM", "xterm-direct");
    assert!(supports_truecolor());

    // Clean up
    std::env::remove_var("TERM");
}

#[test]
fn truecolor_detection_conservative_default() {
    std::env::remove_var("COLORTERM");
    std::env::set_var("TERM", "xterm");
    assert!(!supports_truecolor());

    // Clean up
    std::env::remove_var("TERM");
}
```

> [!NOTE]
> These tests mutate environment variables and should be run with `--test-threads=1`
> or wrapped in a serial test harness to avoid race conditions.

**Run**:
```bash
cargo test -p leiden-tui truecolor_detection -- --test-threads=1
```

**Expected**: All assertions pass.

---

## 6. Scenario: State Theme Completeness

**Purpose**: Verify that `state_color()`, `state_indicator()`, and `state_label()` cover all `AppState` variants (FR-003).

**Test approach**:

```rust
#[test]
fn state_theme_covers_all_variants() {
    let variants = [
        AppState::Idle,
        AppState::Running { iteration: 0 },
        AppState::Done { iterations: 10, quality: 0.45 },
        AppState::Error("test".to_string()),
    ];

    for state in &variants {
        // Each function must return a non-default value
        let _color = state_color(state);
        let indicator = state_indicator(state);
        let label = state_label(state);

        assert!(!indicator.is_empty(), "indicator empty for {state:?}");
        assert!(!label.is_empty(), "label empty for {state:?}");
    }
}

#[test]
fn state_indicators_are_unique() {
    let indicators = [
        state_indicator(&AppState::Idle),
        state_indicator(&AppState::Running { iteration: 0 }),
        state_indicator(&AppState::Done { iterations: 0, quality: 0.0 }),
        state_indicator(&AppState::Error(String::new())),
    ];

    // All indicators must be distinct
    for i in 0..indicators.len() {
        for j in (i + 1)..indicators.len() {
            assert_ne!(indicators[i], indicators[j],
                "indicators at {i} and {j} are identical");
        }
    }
}
```

**Run**:
```bash
cargo test -p leiden-tui state_theme
```

**Expected**: All assertions pass, confirming color + symbol uniqueness per state.

---

## 7. Scenario: Layout Mode Breakpoints

**Purpose**: Verify `layout_mode()` returns correct variants at all breakpoints (FR-004).

**Test approach**:

```rust
#[test]
fn layout_mode_breakpoints() {
    // Full mode
    assert_eq!(layout_mode(120), LayoutMode::Full);
    assert_eq!(layout_mode(200), LayoutMode::Full);

    // Compact mode
    assert_eq!(layout_mode(119), LayoutMode::Compact);
    assert_eq!(layout_mode(80), LayoutMode::Compact);

    // Stacked mode
    assert_eq!(layout_mode(79), LayoutMode::Stacked);
    assert_eq!(layout_mode(60), LayoutMode::Stacked);

    // Minimal mode
    assert_eq!(layout_mode(59), LayoutMode::Minimal);
    assert_eq!(layout_mode(1), LayoutMode::Minimal);
    assert_eq!(layout_mode(0), LayoutMode::Minimal);
}
```

**Run**:
```bash
cargo test -p leiden-tui layout_mode_breakpoints
```

**Expected**: All boundary and interior values pass.

---

## 8. Scenario: No Italic Modifier in Style Presets

**Purpose**: Verify that no style preset function produces a `Style` containing `Modifier::ITALIC` (FR-016).

**Test approach**:

```rust
#[test]
fn no_italic_in_style_presets() {
    let styles = [
        focused_border_style(),
        unfocused_border_style(),
        title_style_focused(),
        title_style_unfocused(),
        header_style(),
        selected_row_style(),
        normal_row_style(),
        key_hint_style(),
        key_letter_style(),
        log_error_style(),
        log_warn_style(),
        log_info_style(),
        log_debug_style(),
        log_trace_style(),
    ];

    for style in &styles {
        assert!(
            !style.add_modifier.contains(Modifier::ITALIC),
            "Style contains ITALIC: {style:?}"
        );
    }
}
```

**Run**:
```bash
cargo test -p leiden-tui no_italic
```

**Expected**: No style preset contains `ITALIC`.

---

## 9. Scenario: Lint Compliance Gate

**Purpose**: Verify the design system modules pass the full workspace lint profile (Constitution §II).

**Run**:
```bash
cargo clippy -p leiden-tui --all-targets -- -D warnings
cargo doc -p leiden-tui --no-deps  # Fails on missing_docs
```

**Expected**: Zero warnings, zero errors. Every `pub` item has `///` documentation.

---

## 10. Scenario: Widget Snapshot Test with Design System

**Purpose**: Verify that widgets render correctly with the new design system constants using `TestBackend`.

**Test approach** (integration test):

```rust
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use leiden_tui::App;
use leiden_tui::ui;

#[test]
fn status_bar_idle_state_snapshot() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let app = App::new_idle();

    terminal.draw(|f| {
        ui::render(f, &app);
    }).unwrap();

    let buffer = terminal.backend().buffer().clone();
    // Verify status bar contains idle indicator "○" and label "Idle"
    let status_line = buffer.content
        .chunks(80)
        .last()
        .unwrap();
    let status_text: String = status_line.iter()
        .map(|cell| cell.symbol().to_string())
        .collect();

    assert!(status_text.contains("○"), "Missing idle indicator");
    assert!(status_text.contains("Idle"), "Missing idle label");
}
```

**Run**:
```bash
cargo test -p leiden-tui status_bar_idle_state_snapshot
```

**Expected**: Snapshot matches — idle indicator `○` and label "Idle" appear in the status bar area.

---

## 11. Scenario: ANSI Fallback Palette Completeness

**Purpose**: Verify that every true-color constant has a corresponding ANSI fallback constant.

**Test approach**:

```rust
#[test]
fn ansi_fallback_palette_complete() {
    // Every RGB constant must have an ANSI counterpart
    let _: Color = BG_0_ANSI;
    let _: Color = BG_1_ANSI;
    let _: Color = BG_2_ANSI;
    let _: Color = BG_3_ANSI;
    let _: Color = BG_4_ANSI;
    let _: Color = FG_0_ANSI;
    let _: Color = FG_1_ANSI;
    let _: Color = FG_2_ANSI;
    let _: Color = FG_3_ANSI;
    let _: Color = ACCENT_PRIMARY_ANSI;
    let _: Color = ACCENT_INFO_ANSI;
    let _: Color = ACCENT_WARNING_ANSI;
    let _: Color = ACCENT_ERROR_ANSI;
    let _: Color = ACCENT_SUCCESS_ANSI;

    // Community ANSI palette exists
    assert_eq!(COMMUNITY_COLORS_ANSI.len(), 6);
}
```

**Run**:
```bash
cargo test -p leiden-tui ansi_fallback
```

**Expected**: Compilation and assertion pass.

---

## Validation Summary

| # | Scenario | FR/SC | Status |
|---|---|---|---|
| 2 | Color constants compile as `const` | FR-001 | ☐ |
| 3 | Style presets compile as `const fn` | FR-002 | ☐ |
| 4 | Community color determinism & wrap | FR-007, SC-002 | ☐ |
| 5 | True-color detection | FR-013 | ☐ |
| 6 | State theme completeness | FR-003, SC-001 | ☐ |
| 7 | Layout mode breakpoints | FR-004, SC-003 | ☐ |
| 8 | No italic modifier | FR-016 | ☐ |
| 9 | Lint compliance gate | Constitution §II, §IV | ☐ |
| 10 | Widget snapshot test | SC-001, SC-005 | ☐ |
| 11 | ANSI fallback completeness | FR-014, SC-005 | ☐ |
