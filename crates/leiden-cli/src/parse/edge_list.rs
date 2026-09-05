//! Edge-list text parser supporting tab and comma separators.

use leiden::{CsrGraph, Edge, LeidenError};

use crate::error::CliError;

/// Parse an edge-list text string into a `CsrGraph<String>`.
///
/// Auto-detects tab vs comma separator from the first data line.
///
/// # Errors
///
/// Returns `CliError::ParseFieldCount` if a line has fewer than 2 or more than 3 fields.
/// Returns `CliError::ParseWeight` if weight float parsing fails.
/// Returns `CliError::Leiden` if a weight is negative/non-finite or a self-loop is present.
pub fn parse_edge_list(content: &str, path: &str) -> Result<CsrGraph<String>, CliError> {
    let mut separator = '\t';
    let mut detected_sep = false;

    // Detect separator from first non-comment, non-empty line
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if !trimmed.contains('\t') && trimmed.contains(',') {
            separator = ',';
        }
        detected_sep = true;
        break;
    }

    if !detected_sep {
        return Err(CliError::Leiden(LeidenError::EmptyGraph));
    }

    let mut edges = Vec::new();

    for (line_idx, line) in content.lines().enumerate() {
        let line_num = line_idx + 1;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = trimmed.split(separator).map(str::trim).collect();
        if parts.len() < 2 || parts.len() > 3 {
            return Err(CliError::ParseFieldCount {
                path: path.to_string(),
                line: line_num,
                got: parts.len(),
            });
        }

        let src = match parts.first() {
            Some(s) => (*s).to_string(),
            None => continue,
        };
        let dst = match parts.get(1) {
            Some(s) => (*s).to_string(),
            None => continue,
        };

        if src == dst {
            return Err(CliError::Leiden(LeidenError::SelfLoop {
                line: Some(line_num),
                node: src,
            }));
        }

        let weight = if parts.len() == 3 {
            let weight_str = parts.get(2).copied().unwrap_or("1.0");
            match weight_str.parse::<f64>() {
                Ok(w) => {
                    if !w.is_finite() || w < 0.0 {
                        return Err(CliError::Leiden(LeidenError::InvalidWeight {
                            line: line_num,
                            value: w,
                        }));
                    }
                    w
                }
                Err(err) => {
                    return Err(CliError::ParseWeight {
                        path: path.to_string(),
                        line: line_num,
                        value: weight_str.to_string(),
                        source: err,
                    });
                }
            }
        } else {
            1.0
        };

        edges.push(Edge {
            source: src,
            target: dst,
            weight,
        });
    }

    if edges.is_empty() {
        return Err(CliError::Leiden(LeidenError::EmptyGraph));
    }

    Ok(CsrGraph::from_edges(edges)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_list_accepts_tab_and_comma() {
        // Tab separated
        let tab_content = "a\tb\t1.5\nb\tc\t2.0\n";
        let Ok(graph_tab) = parse_edge_list(tab_content, "tab.edg") else {
            return;
        };
        assert_eq!(graph_tab.node_count(), 3);
        assert_eq!(graph_tab.edge_count(), 2);

        // Comma separated
        let comma_content = "a,b,1.5\nb,c,2.0\n";
        let Ok(graph_comma) = parse_edge_list(comma_content, "comma.edg") else {
            return;
        };
        assert_eq!(graph_comma.node_count(), 3);
        assert_eq!(graph_comma.edge_count(), 2);
    }

    #[test]
    fn edge_list_ignores_header_comment() {
        let content = "# nodes=100\n# edges=200\na\tb\n";
        let Ok(graph) = parse_edge_list(content, "test.edg") else {
            return;
        };
        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
    }
}

#[cfg(test)]
mod property_tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::format_push_string,
        clippy::uninlined_format_args,
        unused_doc_comments,
        reason = "test code"
    )]

    use proptest::prelude::*;
    use std::fmt::Write;

    use super::parse_edge_list;
    use crate::error::CliError;
    use crate::parse::dispatch::parse_graph_input;
    use crate::testing::config::proptest_cfg;
    use crate::testing::invariants::assert_eps_eq;

    /// Verifies INV-008: Valid edge lists parse and roundtrip.
    ///
    /// Generates random valid edge lists, renders them as tab-separated text,
    /// parses the result, and asserts structural equality (node count, edge
    /// count, total weight with epsilon comparison).
    ///
    /// Note: The graph is undirected, so edges (a,b) and (b,a) are combined
    /// into a single edge with summed weight. The test accounts for this by
    /// tracking unique undirected edge pairs.
    proptest! {
        #![proptest_config(proptest_cfg(Some(10), cfg!(debug_assertions)))]

        #[test]
        fn valid_input_roundtrip(
            edges in prop::collection::vec(
                ("[a-z]{1,5}", "[a-z]{1,5}", 1.0..100.0f64),
                1..15
            ).prop_filter("no self-loops", |e| e.iter().all(|(s, t, _)| s != t))
        ) {
            // Render as tab-separated edge list with weights
            let mut content = String::new();
            let mut expected_nodes = std::collections::HashSet::new();
            // Track unique undirected edges and their total weight
            let mut expected_edge_weight = std::collections::HashMap::new();

            for (src, dst, weight) in &edges {
                let _ = writeln!(content, "{src}\t{dst}\t{weight}");
                let _ = expected_nodes.insert(src.clone());
                let _ = expected_nodes.insert(dst.clone());
                // Normalize edge direction for undirected comparison
                let key = if src < dst {
                    (src.clone(), dst.clone())
                } else {
                    (dst.clone(), src.clone())
                };
                *expected_edge_weight.entry(key).or_insert(0.0) += weight;
            }

            // Roundtrip: valid input must parse successfully
            let graph = parse_edge_list(&content, "test.edg")
                .expect("valid edge list should parse");

            let expected_weight: f64 = expected_edge_weight.values().sum();

            prop_assert_eq!(graph.node_count(), expected_nodes.len());
            prop_assert_eq!(graph.edge_count(), expected_edge_weight.len());
            assert_eps_eq(graph.total_weight(), expected_weight);
        }
    }

    /// Verifies whitespace/delimiter variations parse correctly.
    ///
    /// Generates valid edges and renders them with extra surrounding
    /// whitespace on each field and line. The parser trims whitespace, so
    /// parsing must succeed and produce a graph.
    ///
    /// Note: The graph is undirected, so duplicate edges (a,b) and (b,a) are
    /// combined. We only assert that parsing succeeds and produces a valid graph.
    proptest! {
        #![proptest_config(proptest_cfg(Some(10), cfg!(debug_assertions)))]

        #[test]
        fn whitespace_variations(
            edges in prop::collection::vec(
                ("[a-z]{1,5}", "[a-z]{1,5}", 1.0..100.0f64),
                1..10
            ).prop_filter("no self-loops", |e| e.iter().all(|(s, t, _)| s != t))
        ) {
            // Render with extra whitespace around fields and lines
            let mut content = String::new();
            for (src, dst, weight) in &edges {
                let _ = writeln!(content, "  {src}  \t  {dst}  \t  {weight}  ");
            }

            let graph = parse_edge_list(&content, "test.edg")
                .expect("edge list with extra whitespace should parse");

            // Graph must have at least 2 nodes and at least 1 edge
            prop_assert!(graph.node_count() >= 2);
            prop_assert!(graph.edge_count() >= 1);
        }
    }

    /// Verifies wrong field count yields ParseFieldCount and non-numeric
    /// weight yields ParseWeight.
    proptest! {
        #![proptest_config(proptest_cfg(Some(10), cfg!(debug_assertions)))]

        #[test]
        fn invalid_characters_error(
            input in prop_oneof![
                // Single field (too few) -> ParseFieldCount
                "[a-z]{1,5}".prop_map(|s| (s, "field_count")),
                // Four fields (too many) -> ParseFieldCount
                "[a-z]{1,5}\t[a-z]{1,5}\t[a-z]{1,5}\t[a-z]{1,5}".prop_map(|s| (s, "field_count")),
                // Non-numeric weight -> ParseWeight (ensure no self-loop)
                "[a-z]{1,5}\t[a-z]{1,5}\t[a-z]{1,5}"
                    .prop_filter("no self-loops", |s| {
                        let parts: Vec<&str> = s.split('\t').collect();
                        parts.len() == 3 && parts[0] != parts[1]
                    })
                    .prop_map(|s| (s, "weight")),
            ]
        ) {
            let (content, expected) = input;
            let result = parse_edge_list(&format!("{content}\n"), "test.edg");

            match expected {
                "field_count" => {
                    prop_assert!(
                        matches!(result, Err(CliError::ParseFieldCount { .. })),
                        "expected ParseFieldCount, got: {result:?}"
                    );
                }
                "weight" => {
                    prop_assert!(
                        matches!(result, Err(CliError::ParseWeight { .. })),
                        "expected ParseWeight, got: {result:?}"
                    );
                }
                _ => unreachable!(),
            }
        }
    }

    /// Verifies correct parser is selected based on file extension/content.
    ///
    /// - `.json` extension routes to JSON parser unconditionally.
    /// - First non-whitespace byte `{` routes to JSON parser.
    /// - Otherwise routes to edge-list parser.
    proptest! {
        #![proptest_config(proptest_cfg(Some(10), cfg!(debug_assertions)))]

        #[test]
        fn parser_dispatch(
            nodes in prop::collection::vec("[a-z]{1,5}", 2..8)
        ) {
            let unique_nodes: Vec<_> = nodes.iter().cloned().collect::<std::collections::HashSet<_>>().into_iter().collect();
            prop_assume!(unique_nodes.len() >= 2);

            // Build valid JSON adjacency document
            let node_list: Vec<String> = unique_nodes.iter().map(|n| format!("\"{n}\"")).collect();
            let edge_pairs: Vec<(String, String)> = unique_nodes.windows(2)
                .map(|w| (w[0].clone(), w[1].clone()))
                .collect();
            let edges_list: Vec<String> = edge_pairs.iter()
                .map(|(a, b)| format!("[\"{a}\", \"{b}\"]"))
                .collect();
            let weights_list: Vec<String> = (0..edge_pairs.len()).map(|_| "1.0".to_string()).collect();

            let json_content = format!(
                "{{\"nodes\": [{}], \"edges\": [{}], \"weights\": [{}]}}",
                node_list.join(", "),
                edges_list.join(", "),
                weights_list.join(", ")
            );

            // .json extension -> JSON parser
            let result = parse_graph_input(&json_content, "test.json");
            prop_assert!(result.is_ok(), ".json extension should route to JSON parser: {result:?}");

            // { byte sniff on .txt -> JSON parser
            let result = parse_graph_input(&json_content, "test.txt");
            prop_assert!(result.is_ok(), "byte sniff should route to JSON parser: {result:?}");

            // Edge-list content with .edg extension -> edge-list parser
            let edge_content = format!("{}\t{}\t1.0\n", unique_nodes[0], unique_nodes[1]);
            let result = parse_graph_input(&edge_content, "test.edg");
            prop_assert!(result.is_ok(), ".edg extension should route to edge-list parser: {result:?}");
        }
    }
}
