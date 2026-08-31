//! Error types for the Leiden library.

/// All fallible operations in the library return this error type.
///
/// Variants carry enough context (line/field, offending value) to satisfy
/// FR-008. No `From<std::io::Error>` blanket impl: fallible conversions use
/// `TryFrom` (Principle III, `fallible_impl_from = deny`).
#[derive(Debug, thiserror::Error)]
pub enum LeidenError {
    /// Wraps a general graph-input-shape failure that does not fit any other
    /// variant. Used for input-shape errors that lack a specific field or value
    /// to highlight (e.g. malformed edge-list header `# nodes=N` with a count
    /// that disagrees with the actual unique nodes, per `cli-schema.md §1.3.1`).
    /// For specific per-field errors, prefer `InvalidWeight`, `SelfLoop`,
    /// `DanglingNode`, or `EmptyGraph` instead of this catch-all. The `line`
    /// field is `Some(N)` when the CLI parser emits this error (source line
    /// known) and `None` when the library emits it directly (no source-line
    /// context in the `IntoIterator<Item = Edge<Id>>` API).
    #[error("graph input: {message}")]
    Graph {
        /// Human-readable description of the input-shape failure.
        message: String,
        /// Source line number when known (CLI path); `None` on the library path.
        line: Option<usize>,
    },

    /// Invalid edge weight: must be finite and non-negative.
    #[error("invalid weight `{value}` at line {line}: must be finite and \u{2265} 0")]
    InvalidWeight {
        /// Source line number of the offending weight.
        line: usize,
        /// The offending weight value.
        value: f64,
    },

    /// Self-loop rejected at the input boundary.
    ///
    /// `line` is `Some(N)` when emitted by the CLI parser (where the source line
    /// number is known) and `None` when emitted by `CsrGraph::from_edges` (whose
    /// `IntoIterator<Item = Edge<Id>>` API carries no line context). The `node`
    /// field is the offending user-supplied node id rendered as a `String` at
    /// both boundaries. This shape is locked by `spec.md` FR-008 and `tasks.md`
    /// T024a (library, `line == None`) + T081 (CLI, `line == Some(N)`).
    #[error("self-loop at line {line:?} on node `{node}`: not permitted")]
    SelfLoop {
        /// Source line number when known; `None` on the library path.
        line: Option<usize>,
        /// Offending node id rendered as a string.
        node: String,
    },

    /// Node id appears in edges but not in any declared node set.
    #[error("node id `{0}` appears in edges but not in any declared node set")]
    DanglingNode(String),

    /// Resolution parameter `gamma` must be finite and strictly positive.
    #[error("resolution \u{03b3} must be > 0; got {0}")]
    InvalidGamma(f64),

    /// Iteration cap must be at least one.
    #[error("iteration cap must be \u{2265} 1; got {0}")]
    InvalidIterationCap(u32),

    /// Graph contains no nodes.
    #[error("graph is empty: no nodes")]
    EmptyGraph,
}

#[cfg(test)]
mod tests {
    use super::LeidenError;

    #[test]
    fn display_graph_variant() {
        let err = LeidenError::Graph {
            message: String::from("header mismatch"),
            line: Some(3),
        };
        assert_eq!(err.to_string(), "graph input: header mismatch");
    }

    #[test]
    fn display_invalid_weight() {
        let err = LeidenError::InvalidWeight { line: 7, value: -1.0 };
        assert_eq!(
            err.to_string(),
            "invalid weight `-1` at line 7: must be finite and \u{2265} 0"
        );
    }

    #[test]
    fn display_self_loop_with_line() {
        let err = LeidenError::SelfLoop {
            line: Some(4),
            node: String::from("a"),
        };
        assert_eq!(
            err.to_string(),
            "self-loop at line Some(4) on node `a`: not permitted"
        );
    }

    #[test]
    fn display_self_loop_without_line() {
        let err = LeidenError::SelfLoop {
            line: None,
            node: String::from("x"),
        };
        assert_eq!(
            err.to_string(),
            "self-loop at line None on node `x`: not permitted"
        );
    }

    #[test]
    fn display_dangling_node() {
        let err = LeidenError::DanglingNode(String::from("z"));
        assert_eq!(
            err.to_string(),
            "node id `z` appears in edges but not in any declared node set"
        );
    }

    #[test]
    fn display_invalid_gamma() {
        let err = LeidenError::InvalidGamma(0.0);
        assert_eq!(err.to_string(), "resolution \u{03b3} must be > 0; got 0");
    }

    #[test]
    fn display_invalid_iteration_cap() {
        let err = LeidenError::InvalidIterationCap(0);
        assert_eq!(err.to_string(), "iteration cap must be \u{2265} 1; got 0");
    }

    #[test]
    fn display_empty_graph() {
        let err = LeidenError::EmptyGraph;
        assert_eq!(err.to_string(), "graph is empty: no nodes");
    }

    #[test]
    fn debug_implemented_for_all_variants() {
        let variants: Vec<LeidenError> = vec![
            LeidenError::Graph {
                message: String::from("m"),
                line: None,
            },
            LeidenError::InvalidWeight { line: 1, value: 0.5 },
            LeidenError::SelfLoop {
                line: None,
                node: String::from("n"),
            },
            LeidenError::DanglingNode(String::from("d")),
            LeidenError::InvalidGamma(1.0),
            LeidenError::InvalidIterationCap(1),
            LeidenError::EmptyGraph,
        ];
        for variant in &variants {
            let debug = format!("{variant:?}");
            assert!(!debug.is_empty());
        }
    }
}
