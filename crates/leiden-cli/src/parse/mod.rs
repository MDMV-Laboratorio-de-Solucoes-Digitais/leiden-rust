//! Input parsing modules for CLI arguments and graph formats.

pub mod args;
pub mod dispatch;
pub mod edge_list;
pub mod json_input;

pub use args::Args;
pub use dispatch::parse_graph_input;
pub use edge_list::parse_edge_list;
pub use json_input::parse_json_input;
