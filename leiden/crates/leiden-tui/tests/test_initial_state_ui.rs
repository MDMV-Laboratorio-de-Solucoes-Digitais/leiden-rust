//! Integration tests: initial unclustered state, explanation content, and
//! preset switching (T013, US1).

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use crossterm::event::{KeyCode, KeyEvent};
use leiden_tui::app::{App, AppState};
use leiden_tui::presets::PresetId;
use leiden_tui::ui;
use leiden_tui::{ExplanationState, TuiError};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

#[test]
fn initial_state_renders_messy_network_headline() {
    let mut app = App::new_idle();
    app.load_preset(PresetId::KarateClub);

    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).expect("creates test terminal");
    let _ = terminal.draw(|f| ui::render(f, &app)).expect("renders frame");

    let debug = format!("{:?}", terminal.backend().buffer());
    assert!(debug.contains("EXPLANATION"));
    assert!(debug.contains("MESSY NETWORK STARTING POINT"));
    assert!(debug.contains("Initial State"));
}

#[test]
fn initial_state_is_unclustered_monochrome() {
    let mut app = App::new_idle();
    app.load_preset(PresetId::KarateClub);

    // Unclustered state: no community centroids have been computed yet
    // and every node remains at its seeded position in [0.05, 0.95].
    assert!(app.simulation.community_centroids.is_empty());
    for pos in app.simulation.node_positions.values() {
        assert!(pos.x >= 0.05 && pos.x <= 0.95);
        assert!(pos.y >= 0.05 && pos.y <= 0.95);
    }
    assert_eq!(app.explanation.phase_name, "Initial State");
    assert_eq!(app.explanation.community_count, 0);
}

#[test]
fn default_preset_is_karate_club() {
    let mut app = App::new_idle();
    app.load_preset(PresetId::KarateClub);
    assert_eq!(app.preset, PresetId::KarateClub);
    assert_eq!(app.dataset_title, "Zachary's Karate Club");
    assert_eq!(app.partition.len(), 34);
    assert_eq!(app.dataset_edges.len(), 78);
}

#[test]
fn preset_key_2_loads_two_cliques_and_pauses() {
    let mut app = App::new_idle();
    app.load_preset(PresetId::KarateClub);
    app.handle_key(KeyEvent::from(KeyCode::Char('2')));

    assert_eq!(app.preset, PresetId::TwoCliques);
    assert_eq!(app.partition.len(), 16);
    assert_eq!(app.dataset_edges.len(), 56);
    // Auto-pause policy: playback stays paused after preset switch
    assert!(app.control.paused.load(std::sync::atomic::Ordering::SeqCst));
    // Explanation resets to Step 1
    assert_eq!(app.explanation.phase_name, "Initial State");
    assert_eq!(app.state, AppState::Idle);
}

#[test]
fn preset_key_3_loads_random_mess() {
    let mut app = App::new_idle();
    app.handle_key(KeyEvent::from(KeyCode::Char('3')));

    assert_eq!(app.preset, PresetId::RandomMess);
    assert_eq!(app.partition.len(), 30);
    assert_eq!(app.dataset_edges.len(), 60);
    assert_eq!(app.explanation.phase_name, "Initial State");
}

#[test]
fn preset_switch_preserves_topology_reset() {
    let mut app = App::new_idle();
    app.load_preset(PresetId::KarateClub);

    // Simulate having run some iterations
    app.iterations = 4;
    app.quality = 0.41;
    app.simulation
        .tick(&app.partition, &app.dataset_edges);

    // Switching presets resets iteration state and physics
    app.handle_key(KeyEvent::from(KeyCode::Char('1')));
    assert_eq!(app.iterations, 0);
    assert!(app.quality.abs() < 1e-9);
    assert!(app.simulation.community_centroids.is_empty());
}

#[test]
fn initial_explanation_grade_within_ceiling() {
    let state = ExplanationState::initial_unclustered(34, 78);
    assert!(state.reading_grade_level <= 8.0);
    assert!(leiden_tui::explanation::contains_jargon(&state.analogy_text).is_none());
}

#[test]
fn custom_file_error_surfaces_as_state() {
    let mut app = App::new_idle();
    app.load_file(std::path::Path::new("/nonexistent/graph.edg"));
    assert!(matches!(app.state, AppState::Error(_)));
    // The TuiError::DatasetNotFound variant exists for this path
    let err = TuiError::DatasetNotFound {
        path: "/nonexistent/graph.edg".to_string(),
    };
    assert!(err.to_string().contains("not found"));
}

#[test]
fn graph_canvas_renders_unclustered_nodes() {
    let mut app = App::new_idle();
    app.load_preset(PresetId::TwoCliques);

    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).expect("creates test terminal");
    let _ = terminal.draw(|f| ui::render(f, &app)).expect("renders frame");

    let debug = format!("{:?}", terminal.backend().buffer());
    assert!(debug.contains("GRAPH VISUALIZATION"));
    // Footer shows active dataset metadata
    assert!(debug.contains("Two Cliques"));
    assert!(debug.contains("(Active)"));
    assert!(debug.contains("16 nodes"));
    assert!(debug.contains("56 edges"));
}
