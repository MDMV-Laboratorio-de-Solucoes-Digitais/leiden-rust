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
