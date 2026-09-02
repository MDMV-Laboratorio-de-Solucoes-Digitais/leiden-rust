//! Output formatting modules for JSON and plain text.

pub mod json_output;
pub mod text_output;

pub use json_output::{Assignment, PartitionOutput, render_json_output};
pub use text_output::render_text_output;
