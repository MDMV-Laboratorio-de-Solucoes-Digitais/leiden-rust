//! Command-line argument definitions and parsing.

use clap::Parser;

/// Non-interactive CLI for deterministic Leiden community detection.
#[derive(Debug, Parser, Clone)]
#[command(
    name = "leiden",
    about = "Deterministic Leiden community detection",
    version,
    author
)]
pub struct Args {
    /// Path to input graph file (edge-list or JSON), or `-` for stdin.
    #[arg(value_name = "GRAPH_FILE")]
    pub graph_file: Option<String>,

    /// Resolution parameter gamma.
    #[arg(
        long,
        default_value_t = 1.0,
        value_name = "F",
        allow_hyphen_values = true
    )]
    pub gamma: f64,

    /// Randomness seed for stochastic refinement (forward compatibility in v1).
    #[arg(long, value_name = "U")]
    pub seed: Option<u64>,

    /// Maximum outer-loop iterations.
    #[arg(long, default_value_t = 10, value_name = "N")]
    pub iteration_cap: u32,

    /// Output format: 'json' or 'text'.
    #[arg(long, default_value = "json", value_name = "FMT")]
    pub format: String,

    /// Optional file path for structured tracing log events.
    #[arg(long, value_name = "PATH")]
    pub log_file: Option<String>,

    /// Tracing log level: 'trace', 'debug', 'info', 'warn', 'error'.
    #[arg(long, default_value = "info", value_name = "LVL")]
    pub log_level: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_args_parsing() {
        let Ok(args) = Args::try_parse_from([
            "leiden",
            "--gamma",
            "1.5",
            "--seed",
            "42",
            "--iteration-cap",
            "20",
            "--format",
            "text",
            "--log-file",
            "/tmp/test.log",
            "--log-level",
            "debug",
            "my_graph.edg",
        ]) else {
            return;
        };

        assert!((args.gamma - 1.5).abs() < f64::EPSILON);
        assert_eq!(args.seed, Some(42));
        assert_eq!(args.iteration_cap, 20);
        assert_eq!(args.format, "text");
        assert_eq!(args.log_file.as_deref(), Some("/tmp/test.log"));
        assert_eq!(args.log_level, "debug");
        assert_eq!(args.graph_file.as_deref(), Some("my_graph.edg"));
    }
}
