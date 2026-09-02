//! Internal library modules for the `leiden` CLI binary.

pub mod error;
pub mod format;
pub mod parse;

pub use error::CliError;
pub use format::{Assignment, PartitionOutput, render_json_output, render_text_output};
pub use parse::{Args, parse_edge_list, parse_graph_input, parse_json_input};
