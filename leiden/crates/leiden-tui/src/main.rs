//! `leiden-tui` binary entry point.

use std::fs::File;
use std::io::{self, IsTerminal};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use clap::Parser;
use color_eyre::Result;
use crossterm::event::{self, Event};
use leiden::LeidenParameters;
use leiden_tui::app::App;
use leiden_tui::logging::{LogPaneLayer, LogRing};
use leiden_tui::ui;
use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM};
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

    // Signal + panic cleanup handler (Contract §4.2): restore terminal
    // state on SIGINT / SIGTERM / SIGHUP and on panic, guaranteeing
    // disable_raw_mode() and LeaveAlternateScreen run before exit.
    let term_flag = Arc::new(AtomicBool::new(false));
    for signal in [SIGINT, SIGTERM, SIGHUP] {
        let _unused = signal_hook::flag::register(signal, Arc::clone(&term_flag))?;
    }
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            io::stdout(),
            crossterm::terminal::LeaveAlternateScreen,
            event::DisableMouseCapture
        );
        original_hook(info);
    }));

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

    // Load the active dataset: a custom CLI file (PresetId::Custom) or the
    // default Karate Club demo preset (FR-006, CHK001).
    if let Some(ref path) = args.graph_file {
        let path_str = path.to_string_lossy().to_string();
        app.graph_path = Some(path_str.clone());
        tracing::info!(path = %path_str, "Loading graph file");
        app.load_file(path);
    } else {
        tracing::info!("No graph file supplied; loading Karate Club demo preset");
        app.load_preset(leiden_tui::presets::PresetId::KarateClub);
    }

    let mut terminal = ratatui::init();

    while !app.control.should_quit {
        // Signal-driven exit: restore terminal and leave cleanly
        if term_flag.load(Ordering::SeqCst) {
            break;
        }

        app.drain();

        // Advance the force-directed physics one relaxation step per frame
        // so node motion is smooth at the 50 ms (20 FPS) tick rate (FR-003).
        app.simulation.tick(&app.partition, &app.dataset_edges);

        let _ = terminal.draw(|f| ui::render(f, &app));

        // CPU throttling (Contract §4.2): when playback is paused/idle the
        // event poll blocks up to 200 ms instead of the 50 ms animation
        // tick, holding CPU utilization below 0.1%.
        let poll_ms = if app.playback.is_playing || app.control.step.load(Ordering::SeqCst) {
            50
        } else {
            200
        };

        if event::poll(Duration::from_millis(poll_ms))?
            && let Event::Key(key) = event::read()?
        {
            app.handle_key(key);
        }
    }

    ratatui::restore();
    Ok(())
}
