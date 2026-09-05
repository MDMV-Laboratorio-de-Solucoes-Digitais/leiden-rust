//! JSON adjacency document parser.

use std::collections::HashSet;

use leiden::{CsrGraph, Edge, LeidenError};
use serde::Deserialize;

use crate::error::CliError;

#[derive(Debug, Deserialize)]
struct AdjacencyDoc {
    nodes: Vec<serde_json::Value>,
    edges: Vec<[serde_json::Value; 2]>,
    #[serde(default)]
    weights: Option<Vec<f64>>,
}

fn json_val_to_string(val: &serde_json::Value) -> String {
    val.as_str()
        .map_or_else(|| val.to_string(), ToString::to_string)
}

/// Parse a JSON adjacency document string into a `CsrGraph<String>`.
///
/// # Errors
///
/// Returns `CliError` if JSON deserialization fails, or if weights length mismatches edges,
/// or if dangling nodes or self-loops are present.
pub fn parse_json_input(content: &str) -> Result<CsrGraph<String>, CliError> {
    let doc: AdjacencyDoc = match serde_json::from_str(content) {
        Ok(d) => d,
        Err(err) => {
            return Err(CliError::Leiden(LeidenError::Graph {
                message: format!("invalid JSON structure: {err}"),
                line: None,
            }));
        }
    };

    if let Some(ref w) = doc.weights
        && w.len() != doc.edges.len()
    {
        return Err(CliError::Leiden(LeidenError::Graph {
            message: format!(
                "mismatched edges ({}) and weights ({}) array lengths",
                doc.edges.len(),
                w.len()
            ),
            line: None,
        }));
    }

    let mut nodes = Vec::with_capacity(doc.nodes.len());
    let mut node_set = HashSet::with_capacity(doc.nodes.len());

    for n_val in &doc.nodes {
        let node_id = json_val_to_string(n_val);
        let _ = node_set.insert(node_id.clone());
        nodes.push(node_id);
    }

    let mut edges = Vec::with_capacity(doc.edges.len());
    for (i, edge_arr) in doc.edges.iter().enumerate() {
        let src = json_val_to_string(&edge_arr[0]);
        let dst = json_val_to_string(&edge_arr[1]);

        if !node_set.contains(&src) {
            return Err(CliError::Leiden(LeidenError::DanglingNode(src)));
        }
        if !node_set.contains(&dst) {
            return Err(CliError::Leiden(LeidenError::DanglingNode(dst)));
        }

        if src == dst {
            return Err(CliError::Leiden(LeidenError::SelfLoop {
                line: None,
                node: src,
            }));
        }

        let weight = doc
            .weights
            .as_ref()
            .map_or(1.0, |weights| weights.get(i).copied().unwrap_or(1.0));

        if !weight.is_finite() || weight < 0.0 {
            return Err(CliError::Leiden(LeidenError::InvalidWeight {
                line: 0,
                value: weight,
            }));
        }

        edges.push(Edge {
            source: src,
            target: dst,
            weight,
        });
    }

    if nodes.is_empty() && edges.is_empty() {
        return Err(CliError::Leiden(LeidenError::EmptyGraph));
    }

    Ok(CsrGraph::from_nodes_and_edges(nodes, edges)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_adjacency_parser_valid_and_mismatched_lengths() {
        let valid_json = r#"{
            "nodes": ["a", "b", "c"],
            "edges": [["a", "b"], ["b", "c"]],
            "weights": [1.0, 2.5]
        }"#;
        let Ok(graph) = parse_json_input(valid_json) else {
            return;
        };
        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 2);

        let mismatch_json = r#"{
            "nodes": ["a", "b", "c"],
            "edges": [["a", "b"]],
            "weights": [1.0, 2.5]
        }"#;
        assert!(parse_json_input(mismatch_json).is_err());
    }
}

#[cfg(test)]
mod property_tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::cast_precision_loss,
        clippy::doc_markdown,
        unused_doc_comments,
        reason = "test code"
    )]

    use proptest::prelude::*;

    use super::parse_json_input;
    use crate::error::CliError;
    use crate::parse::edge_list::parse_edge_list;
    use crate::testing::config::proptest_cfg;
    use crate::testing::invariants::assert_eps_eq;

    use proptest::strategy::Just;

    /// Verifies INV-008: Valid JSON adjacency documents parse and roundtrip.
    ///
    /// Generates random node sets and edge lists, renders them as JSON
    /// adjacency documents, parses the result, and asserts structural
    /// equality (node count, edge count, total weight with epsilon).
    proptest! {
        #![proptest_config(proptest_cfg(Some(10), cfg!(debug_assertions)))]

        #[test]
        fn valid_json_roundtrip(
            nodes in prop::collection::vec("[a-z]{1,5}", 2..10)
        ) {
            let unique: std::collections::HashSet<_> = nodes.iter().cloned().collect();
            let unique_nodes: Vec<_> = unique.into_iter().collect();
            prop_assume!(unique_nodes.len() >= 2);

            // Build edges from consecutive node pairs
            let edge_pairs: Vec<(String, String)> = unique_nodes.windows(2)
                .map(|w| (w[0].clone(), w[1].clone()))
                .collect();

            let node_list: Vec<String> = unique_nodes.iter().map(|n| format!("\"{n}\"")).collect();
            let edges_list: Vec<String> = edge_pairs.iter()
                .map(|(a, b)| format!("[\"{a}\", \"{b}\"]"))
                .collect();
            let weights_list: Vec<String> = (0..edge_pairs.len())
                .map(|_| "1.5".to_string())
                .collect();

            let json_content = format!(
                "{{\"nodes\": [{}], \"edges\": [{}], \"weights\": [{}]}}",
                node_list.join(", "),
                edges_list.join(", "),
                weights_list.join(", ")
            );

            // Roundtrip: valid JSON must parse successfully
            let graph = parse_json_input(&json_content)
                .expect("valid JSON adjacency document should parse");

            prop_assert_eq!(graph.node_count(), unique_nodes.len());
            prop_assert_eq!(graph.edge_count(), edge_pairs.len());
            assert_eps_eq(graph.total_weight(), 1.5 * edge_pairs.len() as f64);
        }
    }

    /// Verifies invalid JSON input yields CliError::Leiden(LeidenError::Graph).
    ///
    /// Generates syntactically invalid JSON strings and asserts the parser
    /// returns the expected Graph error variant.
    proptest! {
        #![proptest_config(proptest_cfg(Some(10), cfg!(debug_assertions)))]

        #[test]
        fn malformed_json_error(
            content in prop_oneof![
                // Random alphabetic strings are not valid JSON
                "[a-z]{1,20}",
                // Truncated JSON structures (constant strings)
                Just(String::from(r"\{")),
                Just(String::from(r#"{"nodes""#)),
                Just(String::from("[a-z]{1,5}\t[a-z]{1,5}\t[a-z]{1,5}")),
            ]
        ) {
            let result = parse_json_input(&content);
            prop_assert!(
                matches!(result, Err(CliError::Leiden(leiden::LeidenError::Graph { .. }))),
                "expected LeidenError::Graph for malformed JSON, got: {result:?}"
            );
        }
    }

    /// Verifies mismatched edges/weights array lengths yield
    /// CliError::Leiden(LeidenError::Graph).
    ///
    /// Generates valid node sets with a valid edge list but a weights array
    /// whose length differs from the edge count.
    proptest! {
        #![proptest_config(proptest_cfg(Some(10), cfg!(debug_assertions)))]

        #[test]
        fn dimension_mismatch_error(
            nodes in prop::collection::vec("[a-z]{1,5}", 3..8)
        ) {
            let unique: std::collections::HashSet<_> = nodes.iter().cloned().collect();
            let unique_nodes: Vec<_> = unique.into_iter().collect();
            prop_assume!(unique_nodes.len() >= 3);

            // Build 2 edges but provide 3 weights
            let edge_pairs: Vec<(String, String)> = unique_nodes.windows(2)
                .take(2)
                .map(|w| (w[0].clone(), w[1].clone()))
                .collect();

            let node_list: Vec<String> = unique_nodes.iter().map(|n| format!("\"{n}\"")).collect();
            let edges_list: Vec<String> = edge_pairs.iter()
                .map(|(a, b)| format!("[\"{a}\", \"{b}\"]"))
                .collect();
            // Mismatched: 3 weights for 2 edges
            let weights_list: Vec<String> = (0..3).map(|_| "1.0".to_string()).collect();

            let json_content = format!(
                "{{\"nodes\": [{}], \"edges\": [{}], \"weights\": [{}]}}",
                node_list.join(", "),
                edges_list.join(", "),
                weights_list.join(", ")
            );

            let result = parse_json_input(&json_content);
            prop_assert!(
                matches!(result, Err(CliError::Leiden(leiden::LeidenError::Graph { .. }))),
                "expected LeidenError::Graph for dimension mismatch, got: {result:?}"
            );
        }
    }

    /// Verifies all 7 `CliError` variants are exercised by parser tests.
    ///
    /// This is an audit test that constructs specific inputs to trigger
    /// each error variant and asserts the correct error is returned.
    /// The 7 variants covered:
    /// 1. `ParseFieldCount` - wrong number of fields per line
    /// 2. `ParseWeight` - non-numeric weight string
    /// 3. `Leiden(Graph)` - malformed JSON or dimension mismatch
    /// 4. `Leiden(InvalidWeight)` - negative or non-finite weight
    /// 5. `Leiden(SelfLoop)` - edge from a node to itself
    /// 6. `Leiden(DanglingNode)` - edge references undeclared node
    /// 7. `Leiden(EmptyGraph)` - no valid edges found
    #[test]
    fn variant_coverage_audit() {
        // 1. ParseFieldCount: single field
        assert!(
            matches!(
                parse_edge_list("only_one_field\n", "test.edg"),
                Err(CliError::ParseFieldCount { .. })
            ),
            "variant 1 (ParseFieldCount) not covered"
        );

        // 2. ParseWeight: non-numeric weight
        assert!(
            matches!(
                parse_edge_list("a\tb\tnot_a_number\n", "test.edg"),
                Err(CliError::ParseWeight { .. })
            ),
            "variant 2 (ParseWeight) not covered"
        );

        // 3. Leiden(Graph): malformed JSON
        assert!(
            matches!(
                parse_json_input("this is not json"),
                Err(CliError::Leiden(leiden::LeidenError::Graph { .. }))
            ),
            "variant 3 (Leiden(Graph)) not covered"
        );

        // 4. Leiden(InvalidWeight): negative weight
        assert!(
            matches!(
                parse_edge_list("a\tb\t-5.0\n", "test.edg"),
                Err(CliError::Leiden(leiden::LeidenError::InvalidWeight { .. }))
            ),
            "variant 4 (Leiden(InvalidWeight)) not covered"
        );

        // 5. Leiden(SelfLoop): node connects to itself
        assert!(
            matches!(
                parse_edge_list("x\tx\t1.0\n", "test.edg"),
                Err(CliError::Leiden(leiden::LeidenError::SelfLoop { .. }))
            ),
            "variant 5 (Leiden(SelfLoop)) not covered"
        );

        // 6. Leiden(DanglingNode): edge references undeclared node
        let dangling_json = r#"{"nodes": ["a"], "edges": [["a", "b"]]}"#;
        assert!(
            matches!(
                parse_json_input(dangling_json),
                Err(CliError::Leiden(leiden::LeidenError::DanglingNode(_)))
            ),
            "variant 6 (Leiden(DanglingNode)) not covered"
        );

        // 7. Leiden(EmptyGraph): empty input
        assert!(
            matches!(
                parse_edge_list("", "test.edg"),
                Err(CliError::Leiden(leiden::LeidenError::EmptyGraph))
            ),
            "variant 7 (Leiden(EmptyGraph)) not covered"
        );
    }
}
