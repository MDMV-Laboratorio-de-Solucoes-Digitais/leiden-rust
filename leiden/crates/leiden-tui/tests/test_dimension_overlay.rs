//! Integration tests: undersized terminal overlay and resize restoration
//! (T034, Phase 6).

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use leiden_tui::app::App;
use leiden_tui::presets::PresetId;
use leiden_tui::ui::{self, TerminalDimensionGuard};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

/// Render `app` into a fresh `width × height` [`TestBackend`] and return the
/// rendered buffer's debug string for content assertions.
fn render_debug(app: &App, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("creates test terminal");
    let _ = terminal
        .draw(|f| ui::render(f, app))
        .expect("renders frame");
    format!("{:?}", terminal.backend().buffer())
}

#[test]
fn guard_standard_is_80x24() {
    let guard = TerminalDimensionGuard::standard();
    assert_eq!(guard.min_columns, 80);
    assert_eq!(guard.min_rows, 24);

    assert!(guard.is_valid(80, 24));
    assert!(!guard.is_valid(79, 24));
    assert!(!guard.is_valid(80, 23));
    assert!(guard.is_valid(120, 40));
}

#[test]
fn undersized_terminal_renders_overlay_only() {
    let mut app = App::new_idle();
    app.load_preset(PresetId::KarateClub);

    let debug = render_debug(&app, 72, 20);

    assert!(debug.contains("TERMINAL TOO SMALL"));
    assert!(debug.contains("Minimum required: 80 × 24"));
    assert!(debug.contains("Current size: 72 × 20"));
    // Normal panels are suspended while the overlay is displayed.
    assert!(!debug.contains("EXPLANATION"));
    assert!(!debug.contains("GRAPH VISUALIZATION"));
}

#[test]
fn resize_restoration_renders_normal_ui() {
    let mut app = App::new_idle();
    app.load_preset(PresetId::KarateClub);

    let debug = render_debug(&app, 100, 30);

    assert!(debug.contains("EXPLANATION"));
    assert!(debug.contains("GRAPH VISUALIZATION"));
    assert!(!debug.contains("TERMINAL TOO SMALL"));
}

#[test]
fn overlay_at_exact_boundary() {
    let mut app = App::new_idle();
    app.load_preset(PresetId::KarateClub);

    // Exactly at the 80 × 24 minimum the normal UI is still valid.
    let debug = render_debug(&app, 80, 24);

    assert!(debug.contains("EXPLANATION"));
    assert!(!debug.contains("TERMINAL TOO SMALL"));
}

#[test]
fn overlay_state_preserved_across_resize() {
    let mut app = App::new_idle();
    app.load_preset(PresetId::KarateClub);
    app.iterations = 7;
    app.quality = 0.33;

    // Render the undersized overlay; `ui::render` takes `&App`, so this must
    // not disturb application state.
    let _ = render_debug(&app, 72, 20);
    assert_eq!(app.iterations, 7);
    assert!((app.quality - 0.33).abs() < 1e-12);

    // The very same app still renders the full UI at a sufficient size.
    let restored = render_debug(&app, 100, 30);
    assert!(restored.contains("EXPLANATION"));
    assert!(!restored.contains("TERMINAL TOO SMALL"));
    assert_eq!(app.iterations, 7);
    assert!((app.quality - 0.33).abs() < 1e-12);
}
