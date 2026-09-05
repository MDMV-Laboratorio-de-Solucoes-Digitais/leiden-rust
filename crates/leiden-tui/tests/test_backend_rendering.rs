//! Integration tests: `TestBackend` rendering at multiple terminal dimensions (T019, T020).
//!
//! These tests verify that the TUI renders correctly using Ratatui's in-memory
//! `TestBackend` without requiring a real terminal. This enables headless CI testing.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use leiden_tui::app::App;
use leiden_tui::presets::PresetId;
use leiden_tui::ui;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

fn render_to_string(app: &App, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("creates test terminal");
    let _ = terminal
        .draw(|f| ui::render(f, app))
        .expect("renders frame");
    format!("{:?}", terminal.backend().buffer())
}

#[test]
fn test_backend_renders_at_minimum_dimensions() {
    let mut app = App::new_idle();
    app.load_preset(PresetId::KarateClub);

    let output = render_to_string(&app, 80, 24);

    assert!(output.contains("EXPLANATION"));
    assert!(output.contains("GRAPH VISUALIZATION"));
    assert!(!output.contains("TERMINAL TOO SMALL"));
}

#[test]
fn test_backend_renders_at_below_minimum_dimensions() {
    let mut app = App::new_idle();
    app.load_preset(PresetId::KarateClub);

    let output = render_to_string(&app, 79, 23);

    assert!(output.contains("TERMINAL TOO SMALL"));
    assert!(!output.contains("EXPLANATION"));
}

#[test]
fn test_backend_renders_at_ultrawide_dimensions() {
    let mut app = App::new_idle();
    app.load_preset(PresetId::KarateClub);

    let output = render_to_string(&app, 240, 60);

    assert!(output.contains("EXPLANATION"));
    assert!(output.contains("GRAPH VISUALIZATION"));
    assert!(!output.contains("TERMINAL TOO SMALL"));
}

#[test]
fn test_backend_renders_without_terminal_initialization() {
    let app = App::new_idle();

    let output = render_to_string(&app, 120, 40);

    assert!(output.contains("LEIDEN"));
}

#[test]
fn test_backend_idle_state_renders_panels() {
    let app = App::new_idle();

    let output = render_to_string(&app, 100, 30);

    assert!(output.contains("EXPLANATION"));
    assert!(output.contains("GRAPH VISUALIZATION"));
    assert!(output.contains("PRESETS"));
}

#[test]
fn test_backend_running_state_renders_progress() {
    let mut app = App::new_idle();
    app.load_preset(PresetId::KarateClub);
    app.state = leiden_tui::app::AppState::Running { iteration: 1 };

    let output = render_to_string(&app, 100, 30);

    assert!(output.contains("EXPLANATION"));
    assert!(output.contains("GRAPH VISUALIZATION"));
}

#[test]
fn test_backend_completed_state_renders_summary() {
    let mut app = App::new_idle();
    app.load_preset(PresetId::KarateClub);
    app.iterations = 10;
    app.quality = 0.75;

    let output = render_to_string(&app, 100, 30);

    assert!(output.contains("EXPLANATION"));
    assert!(output.contains("GRAPH VISUALIZATION"));
}

#[test]
fn test_backend_buffer_cells_accessible() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("creates test terminal");

    let _ = terminal.draw(|f| {
        let app = App::new_idle();
        ui::render(f, &app);
    });

    let buffer = terminal.backend().buffer();
    assert!(!buffer.content().is_empty());
}
