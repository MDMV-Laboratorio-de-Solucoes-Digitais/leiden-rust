//! Internal library modules for the `leiden` CLI binary.

pub mod error;
pub mod format;
pub mod parse;

pub use error::CliError;
pub use format::{Assignment, PartitionOutput, render_json_output, render_text_output};
pub use parse::{Args, parse_edge_list, parse_graph_input, parse_json_input};

/// Test-only utilities for property-based testing.
///
/// This module is `#[cfg(test)]` — zero production code impact.
#[cfg(test)]
pub mod testing {
    pub mod config;
    pub mod invariants;
}
