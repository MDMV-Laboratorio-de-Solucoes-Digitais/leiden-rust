//! Domain error types for `leiden-tui`.
//!
//! All fallible operations return `TuiError` variants rather than panicking,
//! per Constitution Principle III (panic-free error propagation).

use thiserror::Error;

/// Errors that can occur in the TUI layer during dataset loading, parsing,
/// and rendering operations.
#[derive(Debug, Error)]
pub enum TuiError {
    /// A CLI-supplied graph file path does not exist or is unreadable.
    #[error("dataset file not found: {path}")]
    DatasetNotFound {
        /// The path that was not found.
        path: String,
    },

    /// The graph file content could not be parsed.
    #[error("failed to parse graph file: {message}")]
    ParseError {
        /// Human-readable description of the parse failure.
        message: String,
    },

    /// An invalid parameter was provided to a TUI operation.
    #[error("invalid parameter: {message}")]
    InvalidParameter {
        /// Description of the invalid value.
        message: String,
    },

    /// A low-level I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
