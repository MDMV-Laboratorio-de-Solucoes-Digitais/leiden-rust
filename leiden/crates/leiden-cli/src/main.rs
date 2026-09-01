//! `leiden` non-interactive CLI binary.

use std::fs::File;
use std::io::{self, Read, Write};
use std::process::ExitCode;
use std::str::FromStr;
use std::sync::mpsc;

use clap::Parser;
use leiden::{Leiden, LeidenError, LeidenEvent, LeidenParameters, TerminationReason};
use leiden_cli::{Args, CliError, parse_graph_input, render_json_output, render_text_output};
use tracing::Level;
use tracing_subscriber::filter::EnvFilter;

fn init_tracing(log_level_str: &str, log_file_path: Option<&str>) -> Result<(), CliError> {
    let level = Level::from_str(log_level_str).unwrap_or(Level::INFO);
    let filter = EnvFilter::default().add_directive(level.into());

    if let Some(path) = log_file_path {
        let file = File::create(path)?;
        let subscriber = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer(file)
            .without_time()
            .with_target(false)
            .with_level(false)
            .finish();
        let _ = tracing::subscriber::set_global_default(subscriber);
    } else {
        let subscriber = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer(io::stderr)
            .without_time()
            .with_target(false)
            .with_level(false)
            .finish();
        let _ = tracing::subscriber::set_global_default(subscriber);
    }

    Ok(())
}

fn read_input(graph_file: Option<&str>) -> Result<(String, String), CliError> {
    match graph_file {
        None | Some("-") => {
            let mut buffer = String::new();
            let _ = io::stdin().read_to_string(&mut buffer)?;
            Ok((buffer, "<stdin>".to_string()))
        }
        Some(path) => {
            let metadata = std::fs::metadata(path).map_err(|err| {
                CliError::Io(io::Error::new(err.kind(), format!("{path}: {err}")))
            })?;
            if metadata.is_dir() {
                return Err(CliError::Io(io::Error::new(
                    io::ErrorKind::IsADirectory,
                    format!("{path}: Is a directory (os error 21)"),
                )));
            }
            let content = std::fs::read_to_string(path).map_err(|err| {
                CliError::Io(io::Error::new(err.kind(), format!("{path}: {err}")))
            })?;
            Ok((content, path.to_string()))
        }
    }
}

fn print_cli_error(err: &CliError, path: &str) {
    match err {
        CliError::UnsupportedFormat(fmt) => {
            eprintln!("unsupported output format '{fmt}'; expected 'json' or 'text'");
        }
        CliError::Io(io_err) => {
            let msg = io_err.to_string();
            if msg.starts_with("io: ") {
                eprintln!("{msg}");
            } else if msg.contains(':') {
                eprintln!("io: {msg}");
            } else {
                eprintln!("io: {path}: {msg}");
            }
        }
        CliError::ParseFieldCount { path: p, line, got } => {
            eprintln!("malformed: {p}:{line}: expected 2 or 3 fields, got {got}");
        }
        CliError::ParseWeight {
            path: p,
            line,
            value,
            ..
        } => {
            eprintln!("malformed: {p}:{line}: invalid weight `{value}`: must be finite and ≥ 0");
        }
        CliError::Leiden(leiden_err) => match leiden_err {
            LeidenError::InvalidWeight { line, value } => {
                let formatted = if (*value - value.round()).abs() < f64::EPSILON {
                    format!("{value:.1}")
                } else {
                    format!("{value}")
                };
                eprintln!(
                    "malformed: {path}:{line}: invalid weight `{formatted}`: must be finite and ≥ 0"
                );
            }
            LeidenError::SelfLoop {
                line: Some(l),
                node,
            } => {
                eprintln!("malformed: {path}:{l}: self-loop on node '{node}': not permitted");
            }
            LeidenError::SelfLoop { line: None, node } => {
                eprintln!("malformed: {path}: self-loop on node '{node}': not permitted");
            }
            LeidenError::DanglingNode(node) => {
                eprintln!(
                    "malformed: node id `{node}` appears in edges but not in any declared node set"
                );
            }
            LeidenError::EmptyGraph => {
                eprintln!("malformed: graph is empty: no nodes");
            }
            LeidenError::InvalidGamma(val) => {
                eprintln!("malformed: resolution γ must be > 0; got {val}");
            }
            LeidenError::InvalidIterationCap(cap) => {
                eprintln!("malformed: iteration cap must be ≥ 1; got {cap}");
            }
            LeidenError::Graph {
                message,
                line: Some(l),
            } => {
                eprintln!("malformed: {path}:{l}: {message}");
            }
            LeidenError::Graph {
                message,
                line: None,
            } => {
                eprintln!("malformed: {path}: {message}");
            }
        },
    }
}

fn run_cli() -> Result<(), (CliError, String)> {
    let args = Args::parse();

    if args.format != "json" && args.format != "text" {
        return Err((
            CliError::UnsupportedFormat(args.format),
            args.graph_file.unwrap_or_else(|| "<stdin>".to_string()),
        ));
    }

    if let Err(err) = init_tracing(&args.log_level, args.log_file.as_deref()) {
        return Err((
            err,
            args.graph_file.unwrap_or_else(|| "<stdin>".to_string()),
        ));
    }

    let (content, path) = read_input(args.graph_file.as_deref()).map_err(|err| {
        (
            err,
            args.graph_file
                .clone()
                .unwrap_or_else(|| "<stdin>".to_string()),
        )
    })?;

    let graph = parse_graph_input(&content, &path).map_err(|err| (err, path.clone()))?;

    tracing::info!(
        "loaded graph: nodes={} edges={} total_weight={:.1}",
        graph.node_count(),
        graph.edge_count(),
        graph.total_weight()
    );

    let (tx, rx) = mpsc::channel();

    let seed_val = args.seed.unwrap_or(0);
    let params = LeidenParameters {
        gamma: args.gamma,
        seed: Some(seed_val),
        iteration_cap: args.iteration_cap,
    };

    let result = Leiden::new()
        .with_parameters(params.clone())
        .with_event_sink(tx)
        .run(&graph)
        .map_err(|err| (CliError::Leiden(err), path.clone()))?;

    while let Ok(event) = rx.try_recv() {
        match event {
            LeidenEvent::IterationFinished { index, quality } => {
                tracing::info!("iteration {}: quality={:.4}", index + 1, quality);
            }
            LeidenEvent::Terminated {
                iterations, reason, ..
            } => {
                let reason_str = match reason {
                    TerminationReason::Converged => "converged",
                    TerminationReason::IterationCap => "iteration_cap",
                    TerminationReason::DegenerateInput => "degenerate_input",
                };
                tracing::info!("terminated after {iterations} iterations: {reason_str}");
            }
            _ => {}
        }
    }

    let mut stdout = io::stdout().lock();
    if args.format == "json" {
        let json_str = render_json_output(&params, &result).map_err(|err| {
            (
                CliError::Leiden(LeidenError::Graph {
                    message: err.to_string(),
                    line: None,
                }),
                path.clone(),
            )
        })?;
        writeln!(stdout, "{json_str}").map_err(|err| (CliError::Io(err), path.clone()))?;
    } else {
        let text_str = render_text_output(&result);
        write!(stdout, "{text_str}").map_err(|err| (CliError::Io(err), path.clone()))?;
    }

    Ok(())
}

fn main() -> ExitCode {
    match run_cli() {
        Ok(()) => ExitCode::SUCCESS,
        Err((err, path)) => {
            print_cli_error(&err, &path);
            let code = err.exit_code();
            ExitCode::from(u8::try_from(code).unwrap_or(1))
        }
    }
}
