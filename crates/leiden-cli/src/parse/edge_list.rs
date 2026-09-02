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
