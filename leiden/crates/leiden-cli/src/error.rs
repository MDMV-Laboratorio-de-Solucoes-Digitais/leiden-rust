//! Error types for the `leiden` CLI binary.

use leiden::LeidenError;

/// Error type encompassing all CLI failure modes with exit-code mapping.
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    /// Line has an invalid number of delimiter-separated columns.
    #[error("{path}:{line}: expected 2 or 3 fields, got {got}")]
    ParseFieldCount {
        /// Source path or `<stdin>`.
        path: String,
        /// 1-indexed line number.
        line: usize,
        /// Number of columns encountered.
        got: usize,
    },

    /// Float weight string failed to parse.
    #[error("{path}:{line}: invalid weight `{value}`: {source}")]
    ParseWeight {
        /// Source path or `<stdin>`.
        path: String,
        /// 1-indexed line number.
        line: usize,
        /// Offending weight string.
        value: String,
        /// Underlying parse error.
        #[source]
        source: std::num::ParseFloatError,
    },

    /// Unrecognized output format passed to `--format`.
    #[error("unsupported output format `{0}`; expected `json` or `text`")]
    UnsupportedFormat(String),

    /// I/O failure reading input or writing output/log files.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    /// Domain algorithm or graph validation error.
    #[error("leiden: {0}")]
    Leiden(#[from] LeidenError),
}

impl CliError {
    /// Return the process exit code for this error per `cli-schema.md §1.6`.
    #[must_use]
    pub const fn exit_code(&self) -> i32 {
        match self {
            Self::UnsupportedFormat(_) => 2,
            Self::Leiden(LeidenError::InvalidGamma(_) | LeidenError::InvalidIterationCap(_)) => 3,
            Self::ParseFieldCount { .. }
            | Self::ParseWeight { .. }
            | Self::Leiden(
                LeidenError::InvalidWeight { .. }
                | LeidenError::SelfLoop { .. }
                | LeidenError::DanglingNode(_)
                | LeidenError::EmptyGraph
                | LeidenError::Graph { .. },
            ) => 4,
            Self::Io(_) => 5,
        }
    }
}
