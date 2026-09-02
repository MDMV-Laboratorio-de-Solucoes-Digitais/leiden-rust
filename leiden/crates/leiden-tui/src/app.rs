//! Application state and transition logic for `leiden-tui`.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use leiden::{CsrGraph, Edge, LeidenEvent, LeidenParameters, RunResult, TerminationReason};

use crate::explanation::ExplanationState;
use crate::logging::LogRing;
use crate::presets::{PresetDataset, PresetId};
use crate::simulation::ForceSimulation;
use crate::worker::spawn_leiden_worker;

/// High-level lifecycle state of the TUI application.
#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    /// Initial idle state before a run starts.
    Idle,
    /// Leiden algorithm is actively running.
    Running {
        /// Current iteration index.
        iteration: u32,
    },
    /// Leiden algorithm has completed.
    Done {
        /// Total iterations completed.
        iterations: u32,
        /// Final modularity quality score.
        quality: f64,
    },
    /// An error occurred during graph loading or execution.
    Error(String),
    /// Prompting for quit confirmation.
    ConfirmQuit(Box<Self>),
}

/// Identifies which panel currently holds keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusPanel {
    /// Community table panel.
    #[default]
    CommunityList,
    /// Graph visualization panel.
    GraphView,
    /// Tracing log viewer panel.
    LogPane,
}

/// Aggregated community statistics for the UI community panel.
#[derive(Debug, Clone, PartialEq)]
pub struct CommunitySummary {
    /// Community identifier.
    pub id: u32,
    /// Number of nodes in community.
    pub size: usize,
    /// Total weight of internal edges within community.
    pub internal_weight: f64,
    /// Total degree / incident edge weight of nodes in community.
    pub total_degree: f64,
}

/// Panel visibility toggle flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PanelVisibility {
    /// Whether the graph visualization panel is visible.
    pub show_graph: bool,
    /// Whether the log viewer panel is visible.
    pub show_log: bool,
    /// Whether the help modal overlay is active.
    pub help_open: bool,
}

/// Stepping granularity mode (FR-005, Data Model §2.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GranularityMode {
    /// Pauses at major algorithm phases (Local Moving, Refinement,
    /// Aggregation).
    #[default]
    PhaseLevel,
    /// Pauses after individual node migrations and sub-steps.
    MicroStep,
}

/// Controls interactive playback and stepping state (Data Model §2.5).
#[derive(Debug, Clone)]
pub struct PlaybackController {
    /// Whether auto-play is actively running.
    pub is_playing: bool,
    /// Auto-play tick speed in milliseconds (fixed: 200ms).
    pub tick_speed_ms: u64,
    /// Single manual step requested flag.
    pub step_requested: bool,
    /// Active granularity mode (persists across preset switches).
    pub granularity: GranularityMode,
}

impl Default for PlaybackController {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaybackController {
    /// Create a default paused controller in `PhaseLevel` mode.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            is_playing: false,
            tick_speed_ms: 200,
            step_requested: false,
            granularity: GranularityMode::PhaseLevel,
        }
    }

    /// Toggle play/pause state.
    pub const fn toggle_play(&mut self) {
        self.is_playing = !self.is_playing;
        if self.is_playing {
            self.step_requested = false;
        }
    }

    /// Request a single step forward (auto-pauses if playing).
    pub const fn request_step(&mut self) {
        self.is_playing = false;
        self.step_requested = true;
    }

    /// Toggle between `PhaseLevel` and `MicroStep` granularity.
    pub const fn toggle_granularity(&mut self) {
        self.granularity = match self.granularity {
            GranularityMode::PhaseLevel => GranularityMode::MicroStep,
            GranularityMode::MicroStep => GranularityMode::PhaseLevel,
        };
    }

    /// Handle preset switch: resets state to Step 1, auto-pauses, and
    /// preserves the user's selected granularity (Contract §2.2).
    pub const fn on_preset_switch(&mut self) {
        self.is_playing = false;
        self.step_requested = false;
    }
}

impl Default for PanelVisibility {
    fn default() -> Self {
        Self {
            show_graph: true,
            show_log: true,
            help_open: false,
        }
    }
}

/// Runtime execution control flags.
#[derive(Debug, Clone, Default)]
pub struct ControlState {
    /// Whether the application should terminate.
    pub should_quit: bool,
    /// Whether auto-iteration is paused.
    pub paused: Arc<AtomicBool>,
    /// Whether the application should advance by exactly one iteration.
    pub step: Arc<AtomicBool>,
    /// Whether the application should abort execution.
    pub abort: Arc<AtomicBool>,
}

/// Central state model for `leiden-tui`.
#[derive(Debug)]
pub struct App {
    /// Current lifecycle state.
    pub state: AppState,
    /// Leiden algorithm parameters.
    pub params: LeidenParameters,
    /// Path to input graph file if provided.
    pub graph_path: Option<String>,
    /// Loaded input graph if available.
    pub graph: Option<CsrGraph<String>>,
    /// Received Leiden events.
    pub events: Vec<LeidenEvent>,
    /// Events received from the worker but not yet applied to the UI state
    /// (FR-005). Manual stepping drains this playhead buffer one event at a
    /// time (`MicroStep`) or up to the next phase boundary (`PhaseLevel`),
    /// while auto-play flushes it completely.
    pub pending_events: VecDeque<LeidenEvent>,
    /// Shared log ring buffer for log pane.
    pub log_ring: Arc<Mutex<LogRing>>,
    /// Current node-to-community partition assignments.
    pub partition: Vec<(String, u32)>,
    /// Current modularity score.
    pub quality: f64,
    /// Current iterations completed.
    pub iterations: u32,
    /// Termination reason if terminated.
    pub termination_reason: Option<TerminationReason>,
    /// Panel visibility configuration.
    pub visibility: PanelVisibility,
    /// Runtime control state.
    pub control: ControlState,
    /// Currently focused panel.
    pub focus: FocusPanel,
    /// Currently selected community row in table.
    pub selected_community: usize,
    /// Optional receiver for incoming Leiden events from worker thread.
    pub rx: Option<Receiver<LeidenEvent>>,
    /// Background worker join handle.
    pub worker_handle: Option<JoinHandle<Result<RunResult<String>, leiden::LeidenError>>>,
    /// Currently active demo preset (FR-006).
    pub preset: PresetId,
    /// Display title of the active dataset (custom files show the file name).
    pub dataset_title: String,
    /// Edges of the active dataset used by the graph canvas and physics.
    pub dataset_edges: Vec<(String, String)>,
    /// 2D force-directed layout simulation (FR-003).
    pub simulation: ForceSimulation,
    /// Current 3-tier plain-English explanation state (FR-004).
    pub explanation: ExplanationState,
    /// Playback and stepping controller (FR-005).
    pub playback: PlaybackController,
}

impl App {
    /// Construct a new `App` in `Idle` state.
    #[must_use]
    pub fn new_idle() -> Self {
        Self {
            state: AppState::Idle,
            params: LeidenParameters::default(),
            graph_path: None,
            graph: None,
            events: Vec::new(),
            pending_events: VecDeque::new(),
            log_ring: Arc::new(Mutex::new(LogRing::default())),
            partition: Vec::new(),
            quality: 0.0,
            iterations: 0,
            termination_reason: None,
            visibility: PanelVisibility::default(),
            control: ControlState::default(),
            focus: FocusPanel::CommunityList,
            selected_community: 0,
            rx: None,
            worker_handle: None,
            preset: PresetId::KarateClub,
            dataset_title: PresetId::KarateClub.title().to_string(),
            dataset_edges: Vec::new(),
            simulation: ForceSimulation::new(&[]),
            explanation: ExplanationState::initial_unclustered(0, 0),
            playback: PlaybackController::new(),
        }
    }

    /// Load a curated preset dataset and reset the explanation to Step 1.
    ///
    /// Per Contract §2.2: switching presets ALWAYS resets the explanation
    /// state machine, auto-pauses playback, and reloads the graph topology.
    pub fn load_preset(&mut self, id: PresetId) {
        let dataset = PresetDataset::get(id);
        self.load_dataset(&dataset);
    }

    /// Load a custom dataset from a CLI file path (FR-006, CHK001).
    ///
    /// Errors surface as `AppState::Error` rather than panicking.
    pub fn load_file(&mut self, path: &std::path::Path) {
        match PresetDataset::from_cli_path(path) {
            Ok(dataset) => self.load_dataset(&dataset),
            Err(err) => self.state = AppState::Error(err.to_string()),
        }
    }

    /// Load a dataset (built-in or custom), rebuilding topology, physics,
    /// and the explanation state.
    ///
    /// Any running worker is aborted; a fresh worker is spawned in the
    /// paused state so playback only starts on user request.
    pub fn load_dataset(&mut self, dataset: &PresetDataset) {
        // Abort any running worker; auto-pause per Contract §2.2 while
        // preserving the user's granularity mode.
        self.control.abort.store(true, Ordering::SeqCst);
        self.control.paused.store(true, Ordering::SeqCst);
        self.control.step.store(false, Ordering::SeqCst);
        self.playback.on_preset_switch();

        let edges = dataset.edges.clone();
        let leiden_edges: Vec<Edge<String>> = edges
            .iter()
            .map(|(s, t)| Edge {
                source: s.clone(),
                target: t.clone(),
                weight: 1.0,
            })
            .collect();

        match CsrGraph::from_edges(leiden_edges) {
            Ok(graph) => {
                let node_count = graph.node_count();
                let mut nodes = Vec::with_capacity(node_count);
                let mut init_partition = Vec::with_capacity(node_count);
                for i in 0..node_count {
                    if let Ok(u) = u32::try_from(i)
                        && let Some(id) = graph.node_id(u)
                    {
                        nodes.push(id.clone());
                        init_partition.push((id.clone(), u));
                    }
                }
                init_partition.sort_by(|a, b| a.0.cmp(&b.0));

                self.preset = dataset.id;
                self.dataset_title = dataset.title.to_string();
                self.dataset_edges = edges;
                self.partition = init_partition;
                self.iterations = 0;
                self.quality = 0.0;
                self.termination_reason = None;
                self.events.clear();
                self.pending_events.clear();
                self.state = AppState::Idle;

                self.simulation = ForceSimulation::new(&nodes);
                self.simulation.reset(&nodes);
                self.explanation =
                    ExplanationState::initial_unclustered(node_count, self.dataset_edges.len());

                let worker_graph = graph.clone();
                self.graph = Some(graph);

                let (rx, worker) = spawn_leiden_worker(
                    worker_graph,
                    self.params.clone(),
                    self.control.paused.clone(),
                    self.control.step.clone(),
                    self.control.abort.clone(),
                );
                self.rx = Some(rx);
                self.worker_handle = Some(worker);
            }
            Err(err) => {
                self.state = AppState::Error(err.to_string());
            }
        }
    }

    /// Set the receiver channel for worker events.
    pub fn with_receiver(&mut self, rx: Receiver<LeidenEvent>) {
        self.rx = Some(rx);
    }

    /// Process a received `LeidenEvent`.
    ///
    /// Updates the lifecycle state machine, the explanation panel (FR-004,
    /// US2/AC1), and the event log. `Terminated` additionally renders the
    /// completion summary via [`ExplanationState::completed`] (US3/AC1).
    pub fn push(&mut self, event: LeidenEvent) {
        event.emit();
        match &event {
            LeidenEvent::IterationStarted { .. } => {
                self.state = AppState::Running {
                    iteration: self.iterations + 1,
                };
            }
            LeidenEvent::IterationFinished {
                index,
                quality,
                partition,
                ..
            } => {
                self.iterations = *index;
                self.quality = *quality;
                self.state = AppState::Running { iteration: *index };

                if let Some(p) = partition
                    && let Some(ref graph) = self.graph
                {
                    let n = graph.node_count();
                    let mut next_partition = Vec::with_capacity(n);
                    for i in 0..n {
                        if let Ok(u_idx) = u32::try_from(i)
                            && let Some(id) = graph.node_id(u_idx)
                        {
                            let comm = p.community_of(u_idx);
                            next_partition.push((id.clone(), comm));
                        }
                    }
                    next_partition.sort_by(|a, b| a.0.cmp(&b.0));
                    self.partition = next_partition;
                }
            }
            LeidenEvent::Terminated {
                iterations,
                reason,
                quality,
            } => {
                self.iterations = *iterations;
                self.quality = *quality;
                self.termination_reason = Some(*reason);
                self.state = AppState::Done {
                    iterations: *iterations,
                    quality: *quality,
                };
            }
            _ => {}
        }
        self.update_explanation(&event);
        self.events.push(event);
    }

    /// Refresh the 3-tier explanation panel from `event` (FR-004).
    ///
    /// `Terminated` renders the completion summary using the distinct
    /// community count of the current partition and the final quality;
    /// every other event maps to its phase narrative with the live
    /// community count attached.
    fn update_explanation(&mut self, event: &LeidenEvent) {
        let communities = distinct_community_count(&self.partition);
        self.explanation = if matches!(event, LeidenEvent::Terminated { .. }) {
            ExplanationState::completed(communities, self.quality)
        } else {
            ExplanationState::from_leiden_event(event, communities)
        };
    }

    /// Set the explanation panel to the final completion summary (US3/AC1),
    /// derived from the current partition and quality.
    fn complete_explanation(&mut self) {
        let communities = distinct_community_count(&self.partition);
        self.explanation = ExplanationState::completed(communities, self.quality);
    }

    /// Drain all pending events from the worker receiver channel.
    ///
    /// Incoming events are first buffered in [`App::pending_events`] and
    /// then applied to the UI state according to the active playback mode
    /// (FR-005): auto-play flushes the whole playhead, while a manual step
    /// applies exactly one event (`MicroStep`) or all events up to and
    /// including the next phase boundary (`PhaseLevel`). The worker is
    /// reaped only after every event has been applied so the playhead
    /// never jumps ahead of the rendered narrative.
    pub fn drain(&mut self) {
        if let Some(ref rx) = self.rx {
            while let Ok(event) = rx.try_recv() {
                self.pending_events.push_back(event);
            }
        }
        self.apply_pending_events();

        if self.pending_events.is_empty()
            && let Some(handle) = self.worker_handle.take()
        {
            if handle.is_finished() {
                match handle.join() {
                    Ok(Ok(run_result)) => {
                        self.partition = run_result.partition;
                        self.quality = run_result.quality;
                        self.iterations = run_result.iterations;
                        self.termination_reason = Some(run_result.termination_reason);
                        self.state = AppState::Done {
                            iterations: run_result.iterations,
                            quality: run_result.quality,
                        };
                        self.complete_explanation();
                    }
                    Ok(Err(err)) => {
                        self.state = AppState::Error(err.to_string());
                    }
                    Err(_) => {
                        self.state = AppState::Error("Worker thread panicked".to_string());
                    }
                }
            } else {
                self.worker_handle = Some(handle);
            }
        }
    }

    /// Apply buffered events to the UI state per the playback mode (FR-005).
    ///
    /// While auto-playing, every buffered event is applied immediately.
    /// While paused with a pending step request, `MicroStep` applies one
    /// event and `PhaseLevel` advances the playhead to the next phase
    /// boundary event; the step request only clears once an event has
    /// actually been applied, so a request waits for the worker to emit.
    fn apply_pending_events(&mut self) {
        if self.playback.is_playing {
            while let Some(event) = self.pending_events.pop_front() {
                self.push(event);
            }
            return;
        }

        if !self.playback.step_requested {
            return;
        }

        match self.playback.granularity {
            GranularityMode::MicroStep => {
                if let Some(event) = self.pending_events.pop_front() {
                    self.push(event);
                    self.playback.step_requested = false;
                }
            }
            GranularityMode::PhaseLevel => {
                let mut applied = 0;
                while let Some(event) = self.pending_events.pop_front() {
                    applied += 1;
                    let boundary = is_phase_boundary(&event);
                    self.push(event);
                    if boundary {
                        break;
                    }
                }
                if applied > 0 {
                    self.playback.step_requested = false;
                }
            }
        }
    }

    /// Terminate the app immediately, aborting any running worker.
    fn request_quit(&mut self) {
        self.control.should_quit = true;
        self.control.abort.store(true, Ordering::SeqCst);
        self.control.paused.store(false, Ordering::SeqCst);
    }

    /// Handle a key press while the quit-confirmation prompt is open:
    /// `y`/`Y` confirms quit, `n`/`N`/`Esc` restores the previous state.
    fn handle_confirm_quit(&mut self, key: KeyEvent) {
        if let AppState::ConfirmQuit(ref prev) = self.state {
            match key.code {
                KeyCode::Char('y' | 'Y') => {
                    self.request_quit();
                }
                KeyCode::Char('n' | 'N') | KeyCode::Esc => {
                    self.state = *prev.clone();
                }
                _ => {}
            }
        }
    }

    /// Cycle keyboard focus to the next visible panel (Tab).
    const fn focus_next_visible(&mut self) {
        self.focus = match self.focus {
            FocusPanel::CommunityList => {
                if self.visibility.show_graph {
                    FocusPanel::GraphView
                } else if self.visibility.show_log {
                    FocusPanel::LogPane
                } else {
                    FocusPanel::CommunityList
                }
            }
            FocusPanel::GraphView => {
                if self.visibility.show_log {
                    FocusPanel::LogPane
                } else {
                    FocusPanel::CommunityList
                }
            }
            FocusPanel::LogPane => FocusPanel::CommunityList,
        };
    }

    /// Restart the run from `Idle`/`Done` (r): respawn the worker when a
    /// graph is loaded, then reset the run state.
    fn restart_run(&mut self) {
        if let Some(ref graph) = self.graph {
            self.control.abort.store(false, Ordering::SeqCst);
            let (rx, worker) = spawn_leiden_worker(
                graph.clone(),
                self.params.clone(),
                self.control.paused.clone(),
                self.control.step.clone(),
                self.control.abort.clone(),
            );
            self.rx = Some(rx);
            self.worker_handle = Some(worker);
        }
        self.state = AppState::Running { iteration: 0 };
        self.iterations = 0;
        self.quality = 0.0;
        self.events.clear();
        self.pending_events.clear();
        let node_count = self.graph.as_ref().map_or(0, CsrGraph::node_count);
        self.explanation =
            ExplanationState::initial_unclustered(node_count, self.dataset_edges.len());
    }

    /// Handle a keyboard event.
    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            if matches!(self.state, AppState::Running { .. }) {
                self.state = AppState::ConfirmQuit(Box::new(self.state.clone()));
                return;
            }
            self.request_quit();
            return;
        }

        // If in Error state, any key returns to Idle (preserving LogRing)
        if matches!(self.state, AppState::Error(_)) {
            self.state = AppState::Idle;
            return;
        }

        if matches!(self.state, AppState::ConfirmQuit(_)) {
            self.handle_confirm_quit(key);
            return;
        }

        match key.code {
            KeyCode::Char('q') => {
                if matches!(self.state, AppState::Running { .. }) {
                    self.state = AppState::ConfirmQuit(Box::new(self.state.clone()));
                } else {
                    self.request_quit();
                }
            }
            KeyCode::Char('1') => self.load_preset(PresetId::KarateClub),
            KeyCode::Char('2') => self.load_preset(PresetId::TwoCliques),
            KeyCode::Char('3') => self.load_preset(PresetId::RandomMess),
            KeyCode::Char(' ') => {
                // Space: toggle play/pause auto-stepping (Contract §2.1)
                self.playback.toggle_play();
                let paused = !self.playback.is_playing;
                self.control.paused.store(paused, Ordering::SeqCst);
            }
            KeyCode::Char('n') | KeyCode::Right => {
                // n / Right Arrow: advance exactly one step, auto-pausing.
                // Buffered events are consumed locally first; the worker is
                // only unblocked when the playhead has caught up (FR-005).
                self.playback.request_step();
                self.control.paused.store(true, Ordering::SeqCst);
                if self.pending_events.is_empty() {
                    self.control.step.store(true, Ordering::SeqCst);
                }
            }
            KeyCode::Char('t') => {
                // t: toggle PhaseLevel / MicroStep granularity
                self.playback.toggle_granularity();
            }
            KeyCode::Char('?') => self.visibility.help_open = !self.visibility.help_open,
            KeyCode::Char('g') => self.visibility.show_graph = !self.visibility.show_graph,
            KeyCode::Char('l') => self.visibility.show_log = !self.visibility.show_log,
            KeyCode::Char('p') => {
                let current = self.control.paused.load(Ordering::SeqCst);
                self.control.paused.store(!current, Ordering::SeqCst);
            }
            KeyCode::Char('s') => {
                self.control.paused.store(true, Ordering::SeqCst);
                self.control.step.store(true, Ordering::SeqCst);
            }
            KeyCode::Tab => self.focus_next_visible(),
            KeyCode::Up => {
                if self.selected_community > 0 {
                    self.selected_community -= 1;
                }
            }
            KeyCode::Down => {
                let summaries = self.community_summaries();
                if !summaries.is_empty() && self.selected_community + 1 < summaries.len() {
                    self.selected_community += 1;
                }
            }
            KeyCode::Char('r') => match self.state {
                AppState::Done { .. } | AppState::Idle => self.restart_run(),
                AppState::Error(_) => {
                    self.state = AppState::Idle;
                }
                AppState::Running { .. } | AppState::ConfirmQuit(_) => {}
            },
            _ => {}
        }
    }

    /// Compute sorted community summaries for display in the community panel.
    #[must_use]
    pub fn community_summaries(&self) -> Vec<CommunitySummary> {
        let mut comm_members: HashMap<u32, Vec<String>> = HashMap::new();
        for (node, comm) in &self.partition {
            comm_members.entry(*comm).or_default().push(node.clone());
        }

        let mut summaries = Vec::new();
        for (comm, members) in comm_members {
            let size = members.len();
            let mut internal_weight = 0.0;
            let mut total_degree = 0.0;

            if let Some(ref graph) = self.graph {
                let member_set: HashSet<_> = members.iter().collect();
                for node in &members {
                    if let Some(u) = graph.internal_id(node) {
                        total_degree += graph.degree_of(u);
                        let nbrs = graph.neighbours_of(u);
                        let weights = graph.weights_of(u);
                        for (idx, &v) in nbrs.iter().enumerate() {
                            if let Some(target_id) = graph.node_id(v)
                                && member_set.contains(target_id)
                                && let Some(&w) = weights.get(idx)
                            {
                                internal_weight += w;
                            }
                        }
                    }
                }
            }

            summaries.push(CommunitySummary {
                id: comm,
                size,
                internal_weight: internal_weight / 2.0,
                total_degree,
            });
        }

        summaries.sort_by(|a, b| b.size.cmp(&a.size).then_with(|| a.id.cmp(&b.id)));
        summaries
    }
}

/// Count the distinct community identifiers in a node-to-community
/// partition.
fn distinct_community_count(partition: &[(String, u32)]) -> usize {
    partition
        .iter()
        .map(|&(_, community)| community)
        .collect::<HashSet<u32>>()
        .len()
}

/// Whether `event` marks a major phase boundary for `PhaseLevel` stepping
/// (FR-005): graph load, entry into a new algorithm phase, iteration
/// completion, or termination. Micro events (progress deltas, quality
/// updates, refinement merges, aggregation details, throttling notices)
/// are not boundaries.
const fn is_phase_boundary(event: &LeidenEvent) -> bool {
    matches!(
        event,
        LeidenEvent::GraphLoaded { .. }
            | LeidenEvent::IterationStarted { .. }
            | LeidenEvent::IterationFinished { .. }
            | LeidenEvent::Terminated { .. }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use leiden::events::Phase;

    #[test]
    fn app_state_transitions() {
        let mut app = App::new_idle();
        assert_eq!(app.state, AppState::Idle);

        // Idle -> Running
        app.handle_key(KeyEvent::from(KeyCode::Char('r')));
        assert_eq!(app.state, AppState::Running { iteration: 0 });

        // Running -> Done
        app.push(LeidenEvent::Terminated {
            iterations: 3,
            reason: TerminationReason::Converged,
            quality: 0.45,
        });
        assert_eq!(
            app.state,
            AppState::Done {
                iterations: 3,
                quality: 0.45
            }
        );

        // Done -> Running (restart)
        app.handle_key(KeyEvent::from(KeyCode::Char('r')));
        assert_eq!(app.state, AppState::Running { iteration: 0 });

        // Running -> Error
        app.state = AppState::Error("Something failed".to_string());
        assert_eq!(app.state, AppState::Error("Something failed".to_string()));

        // Error -> Idle
        app.handle_key(KeyEvent::from(KeyCode::Char('x')));
        assert_eq!(app.state, AppState::Idle);
    }

    #[test]
    fn error_to_idle_recovery_preserves_log() {
        let mut app = App::new_idle();
        if let Ok(mut ring) = app.log_ring.lock() {
            ring.push_back("diagnostic entry 1".to_string());
            ring.push_back("diagnostic entry 2".to_string());
        }

        app.state = AppState::Error("File corrupt".to_string());

        // Key press restores to Idle
        app.handle_key(KeyEvent::from(KeyCode::Char(' ')));
        assert_eq!(app.state, AppState::Idle);

        // LogRing preserved
        let Ok(ring) = app.log_ring.lock() else {
            return;
        };
        assert_eq!(ring.len(), 2);
        assert_eq!(
            ring.entries().front().map(String::as_str),
            Some("diagnostic entry 1")
        );
    }

    #[test]
    fn pause_and_step_key_bindings() {
        let mut app = App::new_idle();

        // Start running
        app.handle_key(KeyEvent::from(KeyCode::Char('r')));
        assert_eq!(app.state, AppState::Running { iteration: 0 });
        assert!(
            !app.control.paused.load(Ordering::SeqCst),
            "Should start unpaused"
        );
        assert!(
            !app.control.step.load(Ordering::SeqCst),
            "Should start with step disabled"
        );

        // 1. Pressing 's' while running continuously
        app.handle_key(KeyEvent::from(KeyCode::Char('s')));
        assert!(
            app.control.paused.load(Ordering::SeqCst),
            "Pressing 's' while running must switch to paused mode"
        );
        assert!(
            app.control.step.load(Ordering::SeqCst),
            "Pressing 's' must request a step"
        );

        // Reset step manually for testing the next transition
        app.control.step.store(false, Ordering::SeqCst);

        // 2. Unpause using 'p'
        app.handle_key(KeyEvent::from(KeyCode::Char('p')));
        assert!(
            !app.control.paused.load(Ordering::SeqCst),
            "Pressing 'p' while paused must unpause"
        );

        // 3. Pause using 'p'
        app.handle_key(KeyEvent::from(KeyCode::Char('p')));
        assert!(
            app.control.paused.load(Ordering::SeqCst),
            "Pressing 'p' while running must pause"
        );

        // 4. Pressing 's' while already paused
        app.handle_key(KeyEvent::from(KeyCode::Char('s')));
        assert!(
            app.control.paused.load(Ordering::SeqCst),
            "Pressing 's' while paused must keep app paused"
        );
        assert!(
            app.control.step.load(Ordering::SeqCst),
            "Pressing 's' while paused must request a step"
        );
    }

    #[test]
    fn quit_confirmation_transitions() {
        let mut app = App::new_idle();

        // Quitting from Idle quits immediately
        app.handle_key(KeyEvent::from(KeyCode::Char('q')));
        assert!(app.control.should_quit);

        let mut app = App::new_idle();
        app.handle_key(KeyEvent::from(KeyCode::Char('r'))); // Now Running
        assert!(matches!(app.state, AppState::Running { .. }));

        // Quitting from Running shows confirm prompt
        app.handle_key(KeyEvent::from(KeyCode::Char('q')));
        assert!(!app.control.should_quit);
        assert!(matches!(app.state, AppState::ConfirmQuit(..)));

        // Pressing 'n' cancels quit
        app.handle_key(KeyEvent::from(KeyCode::Char('n')));
        assert!(!app.control.should_quit);
        assert!(matches!(app.state, AppState::Running { .. }));

        // Quitting again
        app.handle_key(KeyEvent::from(KeyCode::Char('q')));
        assert!(matches!(app.state, AppState::ConfirmQuit(..)));

        // Pressing 'y' confirms quit
        app.handle_key(KeyEvent::from(KeyCode::Char('y')));
        assert!(app.control.should_quit);
    }

    // --- T041: explanation panel wiring (FR-004, US2/AC1, US3/AC1) ---

    #[test]
    fn leiden_events_update_explanation_panel() {
        let mut app = App::new_idle();

        app.push(LeidenEvent::IterationStarted {
            index: 0,
            phase: Phase::LocalMoving,
        });
        assert_eq!(app.explanation.phase_name, "Local Moving");

        app.push(LeidenEvent::IterationStarted {
            index: 0,
            phase: Phase::Refinement,
        });
        assert_eq!(app.explanation.phase_name, "Refinement");

        app.push(LeidenEvent::IterationStarted {
            index: 0,
            phase: Phase::Aggregation,
        });
        assert_eq!(app.explanation.phase_name, "Aggregation");
    }

    #[test]
    fn terminated_event_sets_completed_explanation() {
        let mut app = App::new_idle();
        app.partition = vec![
            ("a".to_string(), 0),
            ("b".to_string(), 1),
            ("c".to_string(), 0),
        ];

        app.push(LeidenEvent::IterationFinished {
            index: 0,
            quality: 0.42,
            partition: None,
        });
        app.push(LeidenEvent::Terminated {
            iterations: 1,
            reason: TerminationReason::Converged,
            quality: 0.42,
        });

        assert_eq!(app.explanation, ExplanationState::completed(2, 0.42));
        assert_eq!(app.explanation.community_count, 2);
        assert_eq!(app.explanation.phase_name, "Finished");
    }

    // --- T042: granularity stepping semantics (FR-005, US2/AC2) ---

    #[test]
    fn micro_step_applies_one_event_per_press() {
        let mut app = App::new_idle();
        app.playback.granularity = GranularityMode::MicroStep;
        app.pending_events.push_back(LeidenEvent::GraphLoaded {
            nodes: 3,
            edges: 2,
            total_weight: 4.0,
        });
        app.pending_events.push_back(LeidenEvent::IterationStarted {
            index: 0,
            phase: Phase::LocalMoving,
        });

        app.handle_key(KeyEvent::from(KeyCode::Char('n')));
        assert!(
            !app.control.step.load(Ordering::SeqCst),
            "'n' with a buffered backlog must not unblock the worker"
        );

        app.drain();
        assert_eq!(
            app.events.len(),
            1,
            "MicroStep must apply exactly one event per step"
        );
        assert!(
            matches!(app.events.first(), Some(LeidenEvent::GraphLoaded { .. })),
            "the first buffered event must be applied first"
        );
        assert!(
            !app.playback.step_requested,
            "the step request must clear once an event is applied"
        );
        assert_eq!(app.pending_events.len(), 1, "the backlog must be preserved");
    }

    #[test]
    fn phase_level_step_advances_to_next_phase_boundary() {
        let mut app = App::new_idle();
        app.playback.granularity = GranularityMode::PhaseLevel;
        for event in [
            LeidenEvent::IterationStarted {
                index: 0,
                phase: Phase::LocalMoving,
            },
            LeidenEvent::LocalMovingProgress {
                iteration: 0,
                moved_nodes: 2,
            },
            LeidenEvent::IterationStarted {
                index: 0,
                phase: Phase::Refinement,
            },
            LeidenEvent::QualityComputed {
                iteration: 0,
                quality: 0.3,
            },
        ] {
            app.pending_events.push_back(event);
        }

        app.handle_key(KeyEvent::from(KeyCode::Right));
        app.drain();
        assert_eq!(
            app.events.len(),
            1,
            "a phase-level step lands on the next boundary event"
        );
        assert!(matches!(
            app.events.last(),
            Some(LeidenEvent::IterationStarted {
                phase: Phase::LocalMoving,
                ..
            })
        ));

        app.handle_key(KeyEvent::from(KeyCode::Right));
        app.drain();
        assert_eq!(
            app.events.len(),
            3,
            "the next phase-level step sweeps micro events up to the following boundary"
        );
        assert!(matches!(
            app.events.last(),
            Some(LeidenEvent::IterationStarted {
                phase: Phase::Refinement,
                ..
            })
        ));
    }

    #[test]
    fn auto_play_applies_all_buffered_events() {
        let mut app = App::new_idle();
        app.playback.granularity = GranularityMode::MicroStep;
        app.pending_events.push_back(LeidenEvent::GraphLoaded {
            nodes: 2,
            edges: 1,
            total_weight: 2.0,
        });
        app.pending_events.push_back(LeidenEvent::Terminated {
            iterations: 1,
            reason: TerminationReason::Converged,
            quality: 0.5,
        });

        app.handle_key(KeyEvent::from(KeyCode::Char(' ')));
        assert!(app.playback.is_playing, "Space must start auto-play");

        app.drain();
        assert_eq!(
            app.events.len(),
            2,
            "auto-play must flush the whole playhead backlog"
        );
        assert!(app.pending_events.is_empty());
        assert!(
            matches!(app.state, AppState::Done { .. }),
            "Terminated must complete the app lifecycle state"
        );
        assert_eq!(app.explanation.phase_name, "Finished");
    }

    #[test]
    fn restart_clears_pending_event_backlog() {
        let mut app = App::new_idle();
        app.pending_events
            .push_back(LeidenEvent::Throttled { dropped: 1 });

        app.handle_key(KeyEvent::from(KeyCode::Char('r')));
        assert!(
            app.pending_events.is_empty(),
            "restart must drop stale buffered events"
        );
        assert_eq!(
            app.explanation.phase_name, "Initial State",
            "restart must reset the explanation narrative"
        );
    }

    #[test]
    fn preset_switch_clears_pending_event_backlog() {
        let mut app = App::new_idle();
        app.pending_events
            .push_back(LeidenEvent::Throttled { dropped: 1 });

        app.handle_key(KeyEvent::from(KeyCode::Char('2')));
        assert!(
            app.pending_events.is_empty(),
            "preset switch must drop stale buffered events"
        );
    }
}
