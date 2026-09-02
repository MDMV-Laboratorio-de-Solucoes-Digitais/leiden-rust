//! Integration tests: curated demo presets and CLI dataset loading (T012).

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use leiden_tui::TuiError;
use leiden_tui::presets::{PresetDataset, PresetId};

#[test]
fn karate_club_dataset_shape() {
    let ds = PresetDataset::get(PresetId::KarateClub);
    let nodes = ds.node_count;
    let edge_total = ds.edge_count;
    assert_eq!(ds.id, PresetId::KarateClub, "preset id must be KarateClub");
    assert_eq!(nodes, 34, "KarateClub node_count: expected 34, got {nodes}");
    assert_eq!(
        edge_total, 78,
        "KarateClub edge_count: expected 78, got {edge_total}"
    );
    for (src, tgt) in &ds.edges {
        for name in [src.as_str(), tgt.as_str()] {
            let n: usize = name.parse().expect("karate node names must be numeric");
            assert!(n <= 33, "karate node name {name} outside \"0\"..=\"33\"");
        }
    }
}

#[test]
fn two_cliques_dataset_shape() {
    let ds = PresetDataset::get(PresetId::TwoCliques);
    let nodes = ds.node_count;
    let edge_total = ds.edge_count;
    assert_eq!(ds.id, PresetId::TwoCliques, "preset id must be TwoCliques");
    assert_eq!(nodes, 16, "TwoCliques node_count: expected 16, got {nodes}");
    assert_eq!(
        edge_total, 56,
        "TwoCliques edge_count: expected 56, got {edge_total}"
    );
}

#[test]
fn random_mess_dataset_shape() {
    let ds = PresetDataset::get(PresetId::RandomMess);
    let nodes = ds.node_count;
    let edge_total = ds.edge_count;
    assert_eq!(ds.id, PresetId::RandomMess, "preset id must be RandomMess");
    assert_eq!(nodes, 30, "RandomMess node_count: expected 30, got {nodes}");
    assert_eq!(
        edge_total, 60,
        "RandomMess edge_count: expected 60, got {edge_total}"
    );
}

#[test]
fn dataset_counts_consistent_with_edges() {
    for (id, label) in [
        (PresetId::KarateClub, "KarateClub"),
        (PresetId::TwoCliques, "TwoCliques"),
        (PresetId::RandomMess, "RandomMess"),
    ] {
        let ds = PresetDataset::get(id);
        let unique: std::collections::HashSet<&str> = ds
            .edges
            .iter()
            .flat_map(|(src, tgt)| [src.as_str(), tgt.as_str()])
            .collect();
        let unique_len = unique.len();
        let edge_len = ds.edges.len();
        let node_count = ds.node_count;
        let edge_count = ds.edge_count;
        assert_eq!(
            node_count, unique_len,
            "{label}: node_count {node_count} != {unique_len} unique node names in edges"
        );
        assert_eq!(
            edge_count, edge_len,
            "{label}: edge_count {edge_count} != edges.len() {edge_len}"
        );
    }
}

#[test]
fn all_presets_lists_three_builtins() {
    let presets = PresetDataset::all_presets();
    let len = presets.len();
    assert_eq!(len, 3, "expected exactly 3 built-in presets, got {len}");
    assert_eq!(
        presets[0].id,
        PresetId::KarateClub,
        "first preset must be KarateClub"
    );
    assert_eq!(
        presets[1].id,
        PresetId::TwoCliques,
        "second preset must be TwoCliques"
    );
    assert_eq!(
        presets[2].id,
        PresetId::RandomMess,
        "third preset must be RandomMess"
    );
}

#[test]
fn from_cli_path_parses_edge_list() {
    let path = std::env::temp_dir().join(format!(
        "leiden_tui_test_presets_{}.edg",
        std::process::id()
    ));
    // The shared leiden-cli edge-list parser auto-detects tab vs comma
    // separators; space-separated fields are rejected with a field-count
    // error, so the edge rows below are tab-separated.
    let content = "# tiny graph\nalpha\tbeta\nbeta\tgamma\ngamma\talpha\ndelta\tepsilon\n";
    std::fs::write(&path, content).expect("failed to write temp edge-list file");

    let result = PresetDataset::from_cli_path(&path);
    let ds = result.expect("from_cli_path must parse a valid tab-separated edge list");
    assert_eq!(
        ds.id,
        PresetId::Custom,
        "CLI-loaded dataset must have id Custom"
    );
    let nodes = ds.node_count;
    let edge_total = ds.edge_count;
    assert_eq!(nodes, 5, "expected 5 nodes, got {nodes}");
    assert_eq!(edge_total, 4, "expected 4 edges, got {edge_total}");

    // The parser may store either direction; assert the undirected pair.
    let has_alpha_beta = ds
        .edges
        .iter()
        .any(|(src, tgt)| (src == "alpha" && tgt == "beta") || (src == "beta" && tgt == "alpha"));
    assert!(
        has_alpha_beta,
        "edges must contain undirected pair (alpha, beta); got {:?}",
        ds.edges
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn from_cli_path_missing_file_returns_error() {
    let path = std::env::temp_dir().join("leiden_tui_test_presets_missing_no_such_file.edg");
    let result = PresetDataset::from_cli_path(&path);
    assert!(
        matches!(result, Err(TuiError::DatasetNotFound { .. })),
        "expected TuiError::DatasetNotFound, got {result:?}"
    );
}

#[test]
fn from_cli_path_garbage_content_returns_error() {
    let path = std::env::temp_dir().join(format!(
        "leiden_tui_test_presets_garbage_{}.edg",
        std::process::id()
    ));
    std::fs::write(&path, "this is not a valid graph @@@")
        .expect("failed to write temp garbage file");
    let result = PresetDataset::from_cli_path(&path);
    assert!(
        result.is_err(),
        "garbage content must return Err, got {result:?}"
    );
    let _ = std::fs::remove_file(path);
}

#[test]
fn preset_titles_are_nonempty() {
    for id in [
        PresetId::KarateClub,
        PresetId::TwoCliques,
        PresetId::RandomMess,
    ] {
        assert!(!id.title().is_empty(), "title for {id:?} must be non-empty");
        assert!(
            !id.description().is_empty(),
            "description for {id:?} must be non-empty"
        );
    }
}
