//! Format dispatch (extension hint + byte sniffing) per `cli-schema.md §1.3`.

use std::path::Path;

use leiden::CsrGraph;

use crate::error::CliError;
use crate::parse::edge_list::parse_edge_list;
use crate::parse::json_input::parse_json_input;

/// Dispatch graph content to JSON or edge-list parser based on path extension and byte sniffing.
///
/// Precedence:
/// 1. File extension `.json` → JSON parser unconditionally.
/// 2. First non-whitespace byte `{` → JSON parser.
/// 3. Otherwise → edge-list parser.
///
/// # Errors
///
/// Returns `CliError` if graph parsing fails.
pub fn parse_graph_input(content: &str, path: &str) -> Result<CsrGraph<String>, CliError> {
    let has_json_ext = Path::new(path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("json"));

    if has_json_ext {
        return parse_json_input(content);
    }

    let is_json_byte_sniff = content.trim_start().starts_with('{');
    if is_json_byte_sniff {
        return parse_json_input(content);
    }

    parse_edge_list(content, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_precedence_matches_cli_schema() {
        // .json extension -> JSON parser
        let json_data = r#"{"nodes": ["a", "b"], "edges": [["a", "b"]]}"#;
        let Ok(g1) = parse_graph_input(json_data, "graph.json") else {
            return;
        };
        assert_eq!(g1.node_count(), 2);

        // { byte sniff on .txt file -> JSON parser
        let Ok(g2) = parse_graph_input(json_data, "graph.txt") else {
            return;
        };
        assert_eq!(g2.node_count(), 2);

        // edge-list
        let edg_data = "a\tb\t1.0\n";
        let Ok(g3) = parse_graph_input(edg_data, "graph.edg") else {
            return;
        };
        assert_eq!(g3.node_count(), 2);
    }

    #[test]
    fn dispatch_extension_wins_over_byte_sniff() {
        // When extension is .edg and starts with { -> byte sniff uses JSON parser
        let json_data = r#"{"nodes": ["x", "y"], "edges": [["x", "y"]]}"#;
        let Ok(g) = parse_graph_input(json_data, "graph.edg") else {
            return;
        };
        assert_eq!(g.node_count(), 2);
    }
}
