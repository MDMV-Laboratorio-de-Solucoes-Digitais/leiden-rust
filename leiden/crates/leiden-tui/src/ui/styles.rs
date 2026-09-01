//! UI style presets, theme helpers, and responsive layout definitions.

use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders};

use crate::app::AppState;
use crate::ui::colors::{
    ACCENT_ERROR, ACCENT_INFO, ACCENT_PRIMARY, ACCENT_SUCCESS, ACCENT_WARNING, BG_3, FG_0, FG_1,
    FG_2, FG_3,
};

// --- 1. Unicode Symbol Constants ---

/// State indicator for `AppState::Idle` — white circle (U+25CB).
pub const INDICATOR_IDLE: &str = "○";
/// State indicator for `AppState::Running` — black circle (U+25CF).
pub const INDICATOR_RUNNING: &str = "●";
/// State indicator for `AppState::Done` — check mark (U+2713).
pub const INDICATOR_DONE: &str = "✓";
/// State indicator for `AppState::Error` — ballot x (U+2717).
pub const INDICATOR_ERROR: &str = "✗";
/// Graph node symbol — black circle (U+25CF).
pub const GRAPH_NODE: &str = "●";
/// Sort indicator — black down-pointing triangle (U+25BC).
pub const SORT_DESC: &str = "▼";
/// Separator dot — middle dot (U+00B7).
pub const SEPARATOR_DOT: &str = "·";
/// Arrow right — rightwards arrow (U+2192).
pub const ARROW_RIGHT: &str = "→";
/// Greek gamma — for resolution parameter display (U+03B3).
pub const GAMMA: &str = "γ";
/// Greek delta — for quality delta display (U+0394).
pub const DELTA: &str = "Δ";

// --- 2. Border & Focus Styles ---

/// Style for focused panel borders.
#[must_use]
pub const fn focused_border_style() -> Style {
    Style::new().fg(ACCENT_PRIMARY)
}

/// Style for unfocused panel borders.
#[must_use]
pub const fn unfocused_border_style() -> Style {
    Style::new().fg(FG_3)
}

/// Style for panel titles when the panel is focused.
#[must_use]
pub const fn title_style_focused() -> Style {
    Style::new().fg(FG_0).add_modifier(Modifier::BOLD)
}

/// Style for panel titles when the panel is unfocused.
#[must_use]
pub const fn title_style_unfocused() -> Style {
    Style::new().fg(FG_2)
}

// --- 3. Table Styles ---

/// Style for table column headers.
///
/// Uses `FG_1` (not `FG_2`) for anchoring headers with strong
/// contrast (8.1:1 on `BG_0`, AAA). `FG_2` is reserved for
/// truly secondary labels.
#[must_use]
pub const fn header_style() -> Style {
    Style::new().fg(FG_1).add_modifier(Modifier::BOLD)
}

/// Style for selected table row.
///
/// Uses explicit `BG_3` + `FG_0` rather than `Modifier::REVERSED`
/// to ensure consistent rendering across terminals with custom
/// palettes and in 16-color fallback mode (FR-015).
#[must_use]
pub const fn selected_row_style() -> Style {
    Style::new().fg(FG_0).bg(BG_3).add_modifier(Modifier::BOLD)
}

/// Style for normal (unselected) table row.
#[must_use]
pub const fn normal_row_style() -> Style {
    Style::new().fg(FG_1)
}

// --- 4. Status Bar Styles ---

/// Style for key hints in the status bar (right-aligned).
#[must_use]
pub const fn key_hint_style() -> Style {
    Style::new().fg(FG_3).add_modifier(Modifier::DIM)
}

/// Style for key letters in hints (e.g., the "q" in "q:quit").
#[must_use]
pub const fn key_letter_style() -> Style {
    Style::new().fg(FG_2)
}

// --- 5. Log Severity Styles ---

/// Style for log level `[ERROR]` prefix (FR-008).
#[must_use]
pub const fn log_error_style() -> Style {
    Style::new().fg(ACCENT_ERROR).add_modifier(Modifier::BOLD)
}

/// Style for log level `[WARN]` prefix (FR-008).
#[must_use]
pub const fn log_warn_style() -> Style {
    Style::new().fg(ACCENT_WARNING).add_modifier(Modifier::BOLD)
}

/// Style for log level `[INFO]` prefix (FR-008).
#[must_use]
pub const fn log_info_style() -> Style {
    Style::new().fg(ACCENT_INFO)
}

/// Style for log level `[DEBUG]` prefix (FR-008).
#[must_use]
pub const fn log_debug_style() -> Style {
    Style::new().fg(FG_2)
}

/// Style for log level `[TRACE]` prefix (FR-008).
#[must_use]
pub const fn log_trace_style() -> Style {
    Style::new().fg(FG_3).add_modifier(Modifier::DIM)
}

// --- 6. Layout Mode ---

/// Terminal-width-dependent layout arrangement.
///
/// Determines how panels are arranged based on the current
/// terminal width (FR-004).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    /// ≥120 columns: community (40%) + graph (60%) side-by-side,
    /// log below (35%), status bar at bottom.
    Full,
    /// 80–119 columns: community (50%) + graph (50%) side-by-side,
    /// log shortened below.
    Compact,
    /// 60–79 columns: community → graph → log in a single column.
    Stacked,
    /// <60 columns: only the focused panel + status bar visible.
    Minimal,
}

/// Determine layout mode from terminal width.
///
/// Returns the appropriate [`LayoutMode`] variant based on the
/// current terminal width in columns. Breakpoints follow the
/// responsive design specified in the design system (FR-004).
#[must_use]
pub const fn layout_mode(width: u16) -> LayoutMode {
    match width {
        120.. => LayoutMode::Full,
        80..=119 => LayoutMode::Compact,
        60..=79 => LayoutMode::Stacked,
        _ => LayoutMode::Minimal,
    }
}

// --- 7. State Theme Helpers ---

/// Map `AppState` to its semantic color (FR-003).
///
/// Each state has a unique accent color that is used for the status
/// bar indicator, label, and progress elements.
#[must_use]
pub const fn state_color(state: &AppState) -> Color {
    match state {
        AppState::Idle => FG_2,
        AppState::Running { .. } => ACCENT_INFO,
        AppState::Done { .. } => ACCENT_SUCCESS,
        AppState::Error(_) => ACCENT_ERROR,
    }
}

/// Map `AppState` to its Unicode indicator symbol (FR-003).
///
/// Each state has a unique symbol that reinforces the state
/// independently of color, ensuring accessibility for users
/// with color vision deficiency.
#[must_use]
pub const fn state_indicator(state: &AppState) -> &'static str {
    match state {
        AppState::Idle => INDICATOR_IDLE,
        AppState::Running { .. } => INDICATOR_RUNNING,
        AppState::Done { .. } => INDICATOR_DONE,
        AppState::Error(_) => INDICATOR_ERROR,
    }
}

/// Map `AppState` to its human-readable label (FR-003).
#[must_use]
pub const fn state_label(state: &AppState) -> &'static str {
    match state {
        AppState::Idle => "Idle",
        AppState::Running { .. } => "Running",
        AppState::Done { .. } => "Done",
        AppState::Error(_) => "Error",
    }
}

// --- 8. Block Builder Functions ---

/// Create a focused panel block with accent border and bold title.
///
/// Uses `BorderType::Rounded` (FR-010) and `ACCENT_PRIMARY` border
/// color to indicate keyboard focus (FR-006).
#[must_use]
pub fn focused_block(title: &str) -> Block<'_> {
    Block::default()
        .title(format!(" {title} "))
        .title_style(title_style_focused())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(focused_border_style())
}

/// Create an unfocused panel block with dim border and muted title.
///
/// Uses `BorderType::Rounded` (FR-010) and `FG_3` border color
/// for non-focused panels.
#[must_use]
pub fn unfocused_block(title: &str) -> Block<'_> {
    Block::default()
        .title(format!(" {title} "))
        .title_style(title_style_unfocused())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(unfocused_border_style())
}

/// Create the appropriate panel block based on focus state.
///
/// Convenience function that dispatches to [`focused_block`] or
/// [`unfocused_block`] based on whether the panel is currently focused.
#[must_use]
pub fn panel_block(title: &str, is_focused: bool) -> Block<'_> {
    if is_focused {
        focused_block(title)
    } else {
        unfocused_block(title)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::AppState;
    use ratatui::style::{Modifier, Style};

    #[test]
    fn style_presets_are_const_fn() {
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

    #[test]
    fn state_theme_covers_all_variants() {
        let variants = [
            AppState::Idle,
            AppState::Running { iteration: 0 },
            AppState::Done {
                iterations: 10,
                quality: 0.45,
            },
            AppState::Error("test".to_string()),
        ];

        for state in &variants {
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
            state_indicator(&AppState::Done {
                iterations: 0,
                quality: 0.0,
            }),
            state_indicator(&AppState::Error(String::new())),
        ];

        for i in 0..indicators.len() {
            for j in (i + 1)..indicators.len() {
                assert_ne!(
                    indicators[i], indicators[j],
                    "indicators at {i} and {j} are identical"
                );
            }
        }
    }

    #[test]
    fn layout_mode_breakpoints() {
        assert_eq!(layout_mode(120), LayoutMode::Full);
        assert_eq!(layout_mode(200), LayoutMode::Full);

        assert_eq!(layout_mode(119), LayoutMode::Compact);
        assert_eq!(layout_mode(80), LayoutMode::Compact);

        assert_eq!(layout_mode(79), LayoutMode::Stacked);
        assert_eq!(layout_mode(60), LayoutMode::Stacked);

        assert_eq!(layout_mode(59), LayoutMode::Minimal);
        assert_eq!(layout_mode(1), LayoutMode::Minimal);
        assert_eq!(layout_mode(0), LayoutMode::Minimal);
    }

    #[test]
    fn block_builders_work() {
        let fb = focused_block("Test");
        let ub = unfocused_block("Test");
        let pb_f = panel_block("Test", true);
        let pb_u = panel_block("Test", false);

        let _ = (fb, ub, pb_f, pb_u);
    }
}
