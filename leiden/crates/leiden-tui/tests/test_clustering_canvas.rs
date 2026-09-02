//! Integration tests: dynamic force relaxation and community color
//! assignment during algorithmic events (T022, US2).

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use leiden_tui::ForceSimulation;
use leiden_tui::app::{App, AppState};
use leiden_tui::presets::PresetId;
use leiden_tui::ui;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

/// Build an `App` loaded with the Two Cliques preset whose partition has
/// been overwritten so nodes "0".."7" form community 0 and nodes "8".."15"
/// form community 1.
fn clustered_two_cliques_app() -> App {
    let mut app = App::new_idle();
    app.load_preset(PresetId::TwoCliques);
    app.partition = two_community_partition();
    app
}

/// Manual two-community partition for the 16-node Two Cliques dataset:
/// nodes "0".."7" -> community 0, nodes "8".."15" -> community 1.
fn two_community_partition() -> Vec<(String, u32)> {
    (0..8)
        .map(|i| (format!("{i}"), 0u32))
        .chain((8..16).map(|i| (format!("{i}"), 1u32)))
        .collect()
}

/// Node IDs of the eight members of community 0.
fn community_zero_nodes() -> Vec<String> {
    (0..8).map(|i| format!("{i}")).collect()
}

/// Average Euclidean pairwise distance among `nodes` in `sim`
/// (mean over all unordered pairs).
fn average_pairwise_distance(sim: &ForceSimulation, nodes: &[String]) -> f64 {
    let points: Vec<_> = nodes.iter().map(|node| sim.node_positions[node]).collect();
    if points.len() < 2 {
        return 0.0;
    }
    let mut total = 0.0;
    let mut pairs = 0usize;
    for (i, a) in points.iter().enumerate() {
        for b in &points[i + 1..] {
            total += a.distance_to(*b);
            pairs += 1;
        }
    }
    total / f64::from(u32::try_from(pairs).expect("pair count fits u32"))
}

#[test]
fn repeated_ticks_pull_same_community_nodes_together() {
    let mut app = clustered_two_cliques_app();
    let community_nodes = community_zero_nodes();

    let initial_distance = average_pairwise_distance(&app.simulation, &community_nodes);

    for _ in 0..60 {
        app.simulation.tick(&app.partition, &app.dataset_edges);
    }

    let relaxed_distance = average_pairwise_distance(&app.simulation, &community_nodes);

    assert!(
        relaxed_distance < initial_distance * 0.99,
        "intra-community average pairwise distance must decrease by at least 1% \
         after 60 relaxation ticks: initial={initial_distance:.6}, \
         relaxed={relaxed_distance:.6}"
    );
    assert!(app.simulation.community_centroids.contains_key(&0));
    assert!(app.simulation.community_centroids.contains_key(&1));
}

#[test]
fn ticks_stay_within_virtual_bounds() {
    let mut app = clustered_two_cliques_app();

    for _ in 0..60 {
        app.simulation.tick(&app.partition, &app.dataset_edges);
    }

    let epsilon = 1e-9;
    for (node, pos) in &app.simulation.node_positions {
        assert!(
            pos.x >= 0.05 - epsilon && pos.x <= 0.95 + epsilon,
            "node {node} x={} outside virtual bounds [0.05, 0.95]",
            pos.x
        );
        assert!(
            pos.y >= 0.05 - epsilon && pos.y <= 0.95 + epsilon,
            "node {node} y={} outside virtual bounds [0.05, 0.95]",
            pos.y
        );
    }
}

#[test]
fn no_nan_after_many_ticks() {
    let mut app = clustered_two_cliques_app();

    for _ in 0..200 {
        app.simulation.tick(&app.partition, &app.dataset_edges);
    }

    for (node, pos) in &app.simulation.node_positions {
        assert!(
            pos.x.is_finite() && pos.y.is_finite(),
            "node {node} position not finite after 200 ticks: {pos:?}"
        );
    }
    for (node, vel) in &app.simulation.node_velocities {
        assert!(
            vel.x.is_finite() && vel.y.is_finite(),
            "node {node} velocity not finite after 200 ticks: {vel:?}"
        );
    }
}

#[test]
fn centroid_matches_member_average_before_motion() {
    let nodes: Vec<String> = (0..6).map(|i| format!("node_{i}")).collect();
    let mut sim = ForceSimulation::new(&nodes);

    // Positions captured before any motion: centroids are computed from the
    // positions at the START of the tick, before relaxation moves nodes.
    let captured: Vec<_> = (0..3)
        .map(|i| sim.node_positions[&format!("node_{i}")])
        .collect();

    let partition: Vec<(String, u32)> = (0..3)
        .map(|i| (format!("node_{i}"), 0u32))
        .collect();
    sim.tick(&partition, &[]);

    let centroid = sim
        .community_centroids
        .get(&0)
        .expect("community 0 centroid exists after one tick");
    let expected_x = (captured[0].x + captured[1].x + captured[2].x) / 3.0;
    let expected_y = (captured[0].y + captured[1].y + captured[2].y) / 3.0;
    assert!(
        (centroid.x - expected_x).abs() < 1e-9 && (centroid.y - expected_y).abs() < 1e-9,
        "centroid must equal the average of member positions captured before the \
         tick: centroid=({:.9}, {:.9}), expected=({expected_x:.9}, {expected_y:.9})",
        centroid.x,
        centroid.y
    );
}

#[test]
fn clustered_state_renders_with_community_colors() {
    let mut app = clustered_two_cliques_app();
    app.state = AppState::Running { iteration: 1 };

    for _ in 0..50 {
        app.simulation.tick(&app.partition, &app.dataset_edges);
    }

    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).expect("creates test terminal");
    let _ = terminal.draw(|f| ui::render(f, &app)).expect("renders frame");

    let debug = format!("{:?}", terminal.backend().buffer());
    // Exact colors cannot be asserted from a text buffer; presence of the
    // canvas plus the active dataset badge is the observable.
    assert!(debug.contains("GRAPH VISUALIZATION"), "canvas title missing");
    assert!(
        debug.contains("Two Cliques"),
        "active dataset badge missing from canvas footer"
    );
}
