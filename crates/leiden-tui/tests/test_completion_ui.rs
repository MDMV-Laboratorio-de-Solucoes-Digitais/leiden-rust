//! Integration test: completion state rendering (T030, US3).

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use leiden_tui::ExplanationState;
use leiden_tui::app::{App, AppState};
use leiden_tui::presets::PresetId;
use leiden_tui::ui;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

/// Build an app simulating algorithm completion: `KarateClub` preset loaded,
/// `Done` lifecycle state, and the final "Neat Communities Discovered!"
/// explanation narrative.
///
/// `app.quality` is set alongside `AppState::Done` because the status bar
/// renders the `App::quality` field (`Q={:.4}`); the `Done` variant's own
/// quality payload is never read by a widget, and
/// `ExplanationState::completed` ignores its quality argument.
fn completed_app() -> App {
    let mut app = App::new_idle();
    app.load_preset(PresetId::KarateClub);
    app.state = AppState::Done {
        iterations: 3,
        quality: 0.4127,
    };
    app.quality = 0.4127;
    app.explanation = ExplanationState::completed(5, 0.4127);
    app
}

/// Render the full UI on a 100×30 `TestBackend` and return the buffer debug
/// string for substring assertions.
fn render_debug_string(app: &App) -> String {
    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).expect("creates test terminal");
    let _ = terminal
        .draw(|f| ui::render(f, app))
        .expect("renders frame");
    let buffer = terminal.backend().buffer();
    format!("{buffer:?}")
}

#[test]
fn done_state_renders_completion_badge() {
    let app = completed_app();
    let debug_str = render_debug_string(&app);
    assert!(
        debug_str.contains("DONE"),
        "status bar should show the DONE state badge, got:\n{debug_str}"
    );
    assert!(
        debug_str.contains("Finished"),
        "explanation phase badge should read Finished, got:\n{debug_str}"
    );
}

#[test]
fn done_state_renders_completed_headline() {
    let app = completed_app();
    let debug_str = render_debug_string(&app);
    assert!(
        debug_str.contains("COMMUNITIES DISCOVERED"),
        "completed headline should be rendered, got:\n{debug_str}"
    );
}

#[test]
fn done_state_shows_quality_metric() {
    let app = completed_app();
    let debug_str = render_debug_string(&app);
    assert!(
        debug_str.contains("0.4127"),
        "final quality metric should be rendered, got:\n{debug_str}"
    );
}

#[test]
fn idle_state_does_not_show_completion() {
    let mut app = App::new_idle();
    app.load_preset(PresetId::KarateClub);
    let debug_str = render_debug_string(&app);
    assert!(
        !debug_str.contains("COMMUNITIES DISCOVERED"),
        "idle state must not show the completion headline, got:\n{debug_str}"
    );
}
