//! Integration tests: `ForceSimulation` physics relaxation math (T007).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::float_cmp,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use leiden_tui::{ForceSimulation, Point2D};
use ratatui::layout::Rect;

fn make_nodes(n: usize) -> Vec<String> {
    (0..n).map(|i| format!("node_{i}")).collect()
}

#[test]
fn simulation_new_initializes_all_nodes() {
    let nodes = make_nodes(5);
    let sim = ForceSimulation::new(&nodes);
    assert_eq!(sim.node_positions.len(), 5);
    assert_eq!(sim.node_velocities.len(), 5);
    assert!(sim.community_centroids.is_empty());

    // All positions should be in [0.05, 0.95]
    for pos in sim.node_positions.values() {
        assert!(pos.x >= 0.05 && pos.x <= 0.95, "x={} out of bounds", pos.x);
        assert!(pos.y >= 0.05 && pos.y <= 0.95, "y={} out of bounds", pos.y);
    }
}

#[test]
fn simulation_new_empty_nodes() {
    let sim = ForceSimulation::new(&[]);
    assert!(sim.node_positions.is_empty());
    assert!(sim.node_velocities.is_empty());
}

#[test]
fn simulation_new_deterministic() {
    let nodes = make_nodes(10);
    let sim1 = ForceSimulation::new(&nodes);
    let sim2 = ForceSimulation::new(&nodes);

    for node in &nodes {
        let p1 = sim1.node_positions.get(node).expect("node exists");
        let p2 = sim2.node_positions.get(node).expect("node exists");
        assert_eq!(p1, p2, "Positions should be deterministic for node {node}");
    }
}

#[test]
fn simulation_tick_with_empty_partition() {
    let nodes = make_nodes(5);
    let mut sim = ForceSimulation::new(&nodes);
    let edges: Vec<(String, String)> = Vec::new();
    let partition: Vec<(String, u32)> = Vec::new();

    // Should not panic with empty partition
    sim.tick(&partition, &edges);

    // Positions should still be valid
    for pos in sim.node_positions.values() {
        assert!(pos.x.is_finite(), "x should be finite");
        assert!(pos.y.is_finite(), "y should be finite");
    }
}

#[test]
fn simulation_tick_avoids_nan() {
    let nodes = make_nodes(3);
    let mut sim = ForceSimulation::new(&nodes);
    let edges = vec![
        ("node_0".to_string(), "node_1".to_string()),
        ("node_1".to_string(), "node_2".to_string()),
    ];
    let partition = vec![
        ("node_0".to_string(), 0),
        ("node_1".to_string(), 0),
        ("node_2".to_string(), 1),
    ];

    for _ in 0..10 {
        sim.tick(&partition, &edges);
    }

    // No NaN or infinite values should appear after multiple ticks
    for pos in sim.node_positions.values() {
        assert!(pos.x.is_finite(), "Position x must not be NaN");
        assert!(pos.y.is_finite(), "Position y must not be NaN");
    }
    for vel in sim.node_velocities.values() {
        assert!(vel.x.is_finite(), "Velocity x must not be NaN");
        assert!(vel.y.is_finite(), "Velocity y must not be NaN");
    }
}

#[test]
fn simulation_tick_community_centroids_computed() {
    let nodes = make_nodes(4);
    let mut sim = ForceSimulation::new(&nodes);
    let partition = vec![
        ("node_0".to_string(), 0),
        ("node_1".to_string(), 0),
        ("node_2".to_string(), 1),
        ("node_3".to_string(), 1),
    ];
    let edges: Vec<(String, String)> = Vec::new();

    // Capture positions before the tick — centroids are computed from these
    let initial_positions: Vec<Point2D> = ["node_0", "node_1"]
        .iter()
        .map(|n| *sim.node_positions.get(*n).expect("node exists"))
        .collect();

    sim.tick(&partition, &edges);

    assert_eq!(sim.community_centroids.len(), 2);
    assert!(sim.community_centroids.contains_key(&0));
    assert!(sim.community_centroids.contains_key(&1));

    // Centroids are computed from positions at the START of the tick,
    // before relaxation moves the nodes.
    let c0 = sim.community_centroids.get(&0).expect("centroid exists");
    let expected_x = f64::midpoint(initial_positions[0].x, initial_positions[1].x);
    let expected_y = f64::midpoint(initial_positions[0].y, initial_positions[1].y);
    assert!((c0.x - expected_x).abs() < 1e-9);
    assert!((c0.y - expected_y).abs() < 1e-9);
}

#[test]
fn simulation_tick_moves_nodes_toward_centroid() {
    let nodes = make_nodes(4);
    let mut sim1 = ForceSimulation::new(&nodes);
    let sim2 = ForceSimulation::new(&nodes);

    let partition = vec![
        ("node_0".to_string(), 0),
        ("node_1".to_string(), 0),
        ("node_2".to_string(), 0),
        ("node_3".to_string(), 0),
    ];
    let edges: Vec<(String, String)> = Vec::new();

    sim1.tick(&partition, &edges);

    for node in &nodes {
        let p_before = sim2.node_positions.get(node).expect("node exists");
        let p_after = sim1.node_positions.get(node).expect("node exists");
        // After one tick, nodes should have moved (attracted to centroid)
        assert!(p_after.x.is_finite() && p_after.y.is_finite());
        // Positions should be different after tick (attraction pulled them)
        let dx = (p_after.x - p_before.x).abs();
        let dy = (p_after.y - p_before.y).abs();
        assert!(
            dx > 1e-6 || dy > 1e-6,
            "Node {node} should have moved after tick"
        );
    }
}

#[test]
fn simulation_screen_coordinates_maps_to_area() {
    let nodes = make_nodes(3);
    let sim = ForceSimulation::new(&nodes);
    let area = Rect::new(0, 0, 80, 24);

    let coords = sim.screen_coordinates(area);
    assert_eq!(coords.len(), 3);

    for (node, (x, y)) in &coords {
        let _ = node;
        // Coordinates should be within the area bounds
        assert!(
            *x >= 0.0 && *x <= f64::from(area.width) + 1.0,
            "x={x} out of area"
        );
        assert!(
            *y >= 0.0 && *y <= f64::from(area.height) + 1.0,
            "y={y} out of area"
        );
    }
}

#[test]
fn simulation_screen_coordinates_respects_offset() {
    let nodes = make_nodes(2);
    let sim = ForceSimulation::new(&nodes);
    let area = Rect::new(10, 5, 80, 24);

    let coords = sim.screen_coordinates(area);
    for coords_val in coords.values() {
        assert!(coords_val.0 >= f64::from(area.x));
        assert!(coords_val.1 >= f64::from(area.y));
    }
}

#[test]
fn simulation_reset_restores_initial_positions() {
    let nodes = make_nodes(5);
    let mut sim = ForceSimulation::new(&nodes);

    // Run several ticks
    let partition: Vec<(String, u32)> = nodes.iter().map(|n| (n.clone(), 0)).collect();
    for _ in 0..10 {
        sim.tick(&partition, &[]);
    }

    // Reset
    sim.reset(&nodes);

    // After reset, positions should match the original seeded positions
    let original_sim = ForceSimulation::new(&nodes);
    for node in &nodes {
        let reset_pos = sim
            .node_positions
            .get(node)
            .expect("node exists after reset");
        let original_pos = original_sim.node_positions.get(node).expect("node exists");
        assert_eq!(
            reset_pos, original_pos,
            "Reset position should match seed for {node}"
        );
    }

    // Reset should also clear velocities
    for vel in sim.node_velocities.values() {
        assert!((vel.x - 0.0).abs() < 1e-9);
        assert!((vel.y - 0.0).abs() < 1e-9);
    }

    // Reset should clear centroids
    assert!(sim.community_centroids.is_empty());
}

#[test]
fn simulation_tick_with_single_node() {
    let nodes = vec!["only".to_string()];
    let mut sim = ForceSimulation::new(&nodes);
    let partition: Vec<(String, u32)> = vec![("only".to_string(), 0)];
    sim.tick(&partition, &[]);

    let pos = sim.node_positions.get("only").expect("node exists");
    assert!(pos.x.is_finite() && pos.y.is_finite());
    // Should stay within bounds
    assert!(pos.x >= 0.05 && pos.x <= 0.95);
    assert!(pos.y >= 0.05 && pos.y <= 0.95);
}

#[test]
fn simulation_tick_collinear_nodes_no_nan() {
    // Nodes placed at same position — zero-division edge case
    let nodes = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let mut sim = ForceSimulation::new(&nodes);
    // Force all nodes to the same position
    for pos in sim.node_positions.values_mut() {
        *pos = Point2D::new(0.5, 0.5);
    }
    let partition: Vec<(String, u32)> = vec![
        ("a".to_string(), 0),
        ("b".to_string(), 0),
        ("c".to_string(), 1),
    ];
    sim.tick(&partition, &[]);

    for pos in sim.node_positions.values() {
        assert!(!pos.x.is_nan(), "NaN in position.x");
        assert!(!pos.y.is_nan(), "NaN in position.y");
    }
}
