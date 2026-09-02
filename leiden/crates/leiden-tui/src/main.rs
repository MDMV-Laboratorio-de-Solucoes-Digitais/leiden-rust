//! `leiden-tui` binary entry point.

use std::fs::File;
use std::io::{self, IsTerminal};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use clap::Parser;
use color_eyre::Result;
use crossterm::event::{self, Event};
use leiden::LeidenParameters;
use leiden_cli::parse_graph_input;
use leiden_tui::app::{App, AppState};
use leiden_tui::logging::{LogPaneLayer, LogRing};
use leiden_tui::ui;
use leiden_tui::worker::spawn_leiden_worker;
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};

#[derive(Debug, Parser)]
#[command(
    name = "leiden-tui",
    about = "Interactive Terminal UI for inspecting Leiden partitions",
    version
)]
struct Args {
    /// Optional starting graph file path.
    #[arg(value_name = "GRAPH_FILE")]
    graph_file: Option<PathBuf>,

    /// Initial resolution parameter gamma.
    #[arg(long, default_value_t = 1.0, value_name = "F")]
    gamma: f64,

    /// Initial randomness seed.
    #[arg(long, value_name = "U")]
    seed: Option<u64>,

    /// Initial iteration cap.
    #[arg(long, default_value_t = 10, value_name = "N")]
    iteration_cap: u32,

    /// Optional file for structured tracing logs.
    #[arg(long, value_name = "PATH")]
    log_file: Option<PathBuf>,

    /// Tracing log level.
    #[arg(long, default_value = "info", value_name = "LVL")]
    log_level: String,
}

fn init_tracing(
    log_level: &str,
    log_file: Option<&PathBuf>,
    log_ring: Arc<Mutex<LogRing>>,
) -> Result<()> {
    let level = Level::from_str(log_level).unwrap_or(Level::INFO);
    let filter = EnvFilter::default().add_directive(level.into());
    let log_layer = LogPaneLayer::new(log_ring);

    if let Some(path) = log_file {
        let file = File::create(path)?;
        let file_layer = fmt::layer().with_writer(file).with_ansi(false);
        tracing_subscriber::registry()
            .with(filter)
            .with(log_layer)
            .with(file_layer)
            .try_init()?;
    } else if !io::stderr().is_terminal() {
        let stderr_layer = fmt::layer().with_writer(io::stderr).with_ansi(false);
        tracing_subscriber::registry()
            .with(filter)
            .with(log_layer)
            .with(stderr_layer)
            .try_init()?;
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(log_layer)
            .try_init()?;
    }

    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    let log_ring = Arc::new(Mutex::new(LogRing::default()));
    init_tracing(
        &args.log_level,
        args.log_file.as_ref(),
        Arc::clone(&log_ring),
    )?;

    let mut app = App::new_idle();
    app.log_ring = log_ring;
    app.params = LeidenParameters {
        gamma: args.gamma,
        seed: args.seed,
        iteration_cap: args.iteration_cap,
    };

    if let Some(ref path) = args.graph_file {
        let path_str = path.to_string_lossy().to_string();
        app.graph_path = Some(path_str.clone());

        match std::fs::read_to_string(path) {
            Ok(content) => match parse_graph_input(&content, &path_str) {
                Ok(graph) => {
                    tracing::info!(
                        nodes = graph.node_count(),
                        edges = graph.edge_count(),
                        "Loaded graph file: {}",
                        path_str
                    );
                    let mut init_partition = Vec::with_capacity(graph.node_count());
                    for i in 0..graph.node_count() {
                        if let Ok(u) = u32::try_from(i)
                            && let Some(id) = graph.node_id(u)
                        {
                            init_partition.push((id.clone(), u));
                        }
                    }
                    init_partition.sort_by(|a, b| a.0.cmp(&b.0));
                    app.partition = init_partition;

                    let (rx, worker) = spawn_leiden_worker(
                        graph.clone(),
                        app.params.clone(),
                        app.control.paused.clone(),
                        app.control.step.clone(),
                        app.control.abort.clone(),
                    );
                    app.graph = Some(graph);
                    app.with_receiver(rx);
                    app.worker_handle = Some(worker);
                    app.state = AppState::Running { iteration: 0 };
                }
                Err(err) => {
                    tracing::error!("Failed to parse graph: {err}");
                    app.state = AppState::Error(err.to_string());
                }
            },
            Err(err) => {
                tracing::error!("Failed to read graph file: {err}");
                app.state = AppState::Error(format!("Failed to read graph file: {err}"));
            }
        }
    }

    let mut terminal = ratatui::init();

    while !app.control.should_quit {
        app.drain();

        let _ = terminal.draw(|f| ui::render(f, &app));

        if event::poll(Duration::from_millis(50))?
            && let Event::Key(key) = event::read()?
        {
            app.handle_key(key);
        }
    }

    ratatui::restore();
    Ok(())
}
