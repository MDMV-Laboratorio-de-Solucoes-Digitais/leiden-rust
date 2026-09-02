//! Application state and transition logic for `leiden-tui`.

use std::collections::{HashMap, HashSet};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use leiden::{CsrGraph, LeidenEvent, LeidenParameters, RunResult, TerminationReason};

use crate::logging::LogRing;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ControlState {
    /// Whether the application should terminate.
    pub should_quit: bool,
    /// Whether auto-iteration is paused.
    pub paused: bool,
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
    /// Handle to the background worker thread executing Leiden.
    pub worker: Option<std::thread::JoinHandle<Result<RunResult<String>, leiden::LeidenError>>>,
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
            log_ring: Arc::new(Mutex::new(LogRing::default())),
            partition: Vec::new(),
            quality: 0.0,
            iterations: 0,
            termination_reason: None,
            visibility: PanelVisibility::default(),
            control: ControlState::default(),
            worker: None,
            focus: FocusPanel::CommunityList,
            selected_community: 0,
            rx: None,
        }
    }

    /// Set the receiver channel for worker events.
    pub fn with_receiver(&mut self, rx: Receiver<LeidenEvent>) {
        self.rx = Some(rx);
    }

    /// Process a received `LeidenEvent`.
    pub fn push(&mut self, event: LeidenEvent) {
        match &event {
            LeidenEvent::IterationStarted { .. } => {
                self.state = AppState::Running {
                    iteration: self.iterations + 1,
                };
            }
            LeidenEvent::IterationFinished { index, quality } => {
                self.iterations = *index;
                self.quality = *quality;
                self.state = AppState::Running { iteration: *index };
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
        self.events.push(event);
    }

    /// Drain all pending events from the worker receiver channel.
    pub fn drain(&mut self) {
        // First, drain any pending events from the worker receiver.
        if let Some(ref rx) = self.rx {
            let mut pending = Vec::new();
            while let Ok(event) = rx.try_recv() {
                pending.push(event);
            }
            for event in pending {
                self.push(event);
            }
        }

        // After processing events, check if the worker thread has finished.
        if let Some(handle) = self.worker.as_ref() {
            if handle.is_finished() {
                // Take ownership of the handle to join it.
                if let Some(handle) = self.worker.take() {
                    match handle.join() {
                        Ok(Ok(run_result)) => {
                            // Update partition with final community assignments.
                            self.partition = run_result.partition;
                        }
                        Ok(Err(e)) => {
                            tracing::error!("Leiden worker returned error: {e:?}");
                            self.state = AppState::Error(format!("Leiden error: {e:?}"));
                        }
                        Err(panic) => {
                            tracing::error!("Leiden worker panicked: {panic:?}");
                            self.state = AppState::Error("Leiden worker panicked".to_string());
                        }
                    }
                }
            }
        }
    }

    /// Handle a keyboard event.
    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.control.should_quit = true;
            return;
        }

        // If in Error state, any key returns to Idle (preserving LogRing)
        if matches!(self.state, AppState::Error(_)) {
            self.state = AppState::Idle;
            return;
        }

        match key.code {
            KeyCode::Char('q') => {
                self.control.should_quit = true;
            }
            KeyCode::Char('?') => {
                self.visibility.help_open = !self.visibility.help_open;
            }
            KeyCode::Char('g') => {
                self.visibility.show_graph = !self.visibility.show_graph;
            }
            KeyCode::Char('l') => {
                self.visibility.show_log = !self.visibility.show_log;
            }
            KeyCode::Char('p') => {
                self.control.paused = !self.control.paused;
            }
            KeyCode::Tab => {
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
                AppState::Done { .. } | AppState::Idle => {
                    self.state = AppState::Running { iteration: 0 };
                    self.iterations = 0;
                    self.quality = 0.0;
                    self.events.clear();
                }
                AppState::Error(_) => {
                    self.state = AppState::Idle;
                }
                AppState::Running { .. } => {}
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
