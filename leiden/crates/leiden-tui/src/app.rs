//! Application state and transition logic for `leiden-tui`.

use std::collections::{HashMap, HashSet};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use leiden::{CsrGraph, LeidenEvent, LeidenParameters, RunResult, TerminationReason};

use crate::logging::LogRing;
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
    ConfirmQuit(Box<AppState>),
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
            focus: FocusPanel::CommunityList,
            selected_community: 0,
            rx: None,
            worker_handle: None,
        }
    }

    /// Set the receiver channel for worker events.
    pub fn with_receiver(&mut self, rx: Receiver<LeidenEvent>) {
        self.rx = Some(rx);
    }

    /// Process a received `LeidenEvent`.
    pub fn push(&mut self, event: LeidenEvent) {
        event.emit();
        match &event {
            LeidenEvent::IterationStarted { .. } => {
                self.state = AppState::Running {
                    iteration: self.iterations + 1,
                };
            }
            LeidenEvent::IterationFinished { index, quality, partition, .. } => {
                self.iterations = *index;
                self.quality = *quality;
                self.state = AppState::Running { iteration: *index };
                
                if let Some(p) = partition {
                    if let Some(ref graph) = self.graph {
                        let n = graph.node_count();
                        let mut next_partition = Vec::with_capacity(n);
                        for i in 0..n {
                            if let Ok(u_idx) = u32::try_from(i) {
                                if let Some(id) = graph.node_id(u_idx) {
                                    let comm = p.community_of(u_idx);
                                    next_partition.push((id.clone(), comm));
                                }
                            }
                        }
                        next_partition.sort_by(|a, b| a.0.cmp(&b.0));
                        self.partition = next_partition;
                    }
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
        self.events.push(event);
    }

    /// Drain all pending events from the worker receiver channel.
    pub fn drain(&mut self) {
        if let Some(ref rx) = self.rx {
            let mut pending = Vec::new();
            while let Ok(event) = rx.try_recv() {
                pending.push(event);
            }
            for event in pending {
                self.push(event);
            }
        }

        if let Some(handle) = self.worker_handle.take() {
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

    /// Handle a keyboard event.
    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            if matches!(self.state, AppState::Running { .. }) {
                self.state = AppState::ConfirmQuit(Box::new(self.state.clone()));
                return;
            }
            self.control.should_quit = true;
            self.control.abort.store(true, Ordering::SeqCst);
            self.control.paused.store(false, Ordering::SeqCst);
            return;
        }

        // If in Error state, any key returns to Idle (preserving LogRing)
        if matches!(self.state, AppState::Error(_)) {
            self.state = AppState::Idle;
            return;
        }

        if let AppState::ConfirmQuit(ref prev) = self.state {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    self.control.should_quit = true;
                    self.control.abort.store(true, Ordering::SeqCst);
                    self.control.paused.store(false, Ordering::SeqCst);
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    self.state = *prev.clone();
                }
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Char('q') => {
                if matches!(self.state, AppState::Running { .. }) {
                    self.state = AppState::ConfirmQuit(Box::new(self.state.clone()));
                } else {
                    self.control.should_quit = true;
                    self.control.abort.store(true, Ordering::SeqCst);
                    self.control.paused.store(false, Ordering::SeqCst);
                }
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
                let current = self.control.paused.load(Ordering::SeqCst);
                self.control.paused.store(!current, Ordering::SeqCst);
            }
            KeyCode::Char('s') => {
                self.control.paused.store(true, Ordering::SeqCst);
                self.control.step.store(true, Ordering::SeqCst);
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
                    if let Some(ref graph) = self.graph {
                        self.control.abort.store(false, Ordering::SeqCst);
                        let (rx, worker) = spawn_leiden_worker(graph.clone(), self.params.clone(), self.control.paused.clone(), self.control.step.clone(), self.control.abort.clone());
                        self.rx = Some(rx);
                        self.worker_handle = Some(worker);
                        self.state = AppState::Running { iteration: 0 };
                        self.iterations = 0;
                        self.quality = 0.0;
                        self.events.clear();
                    } else {
                        self.state = AppState::Running { iteration: 0 };
                        self.iterations = 0;
                        self.quality = 0.0;
                        self.events.clear();
                    }
                }
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

    #[test]
    fn pause_and_step_key_bindings() {
        let mut app = App::new_idle();
        
        // Start running
        app.handle_key(KeyEvent::from(KeyCode::Char('r')));
        assert_eq!(app.state, AppState::Running { iteration: 0 });
        assert!(!app.control.paused.load(Ordering::SeqCst), "Should start unpaused");
        assert!(!app.control.step.load(Ordering::SeqCst), "Should start with step disabled");

        // 1. Pressing 's' while running continuously
        app.handle_key(KeyEvent::from(KeyCode::Char('s')));
        assert!(app.control.paused.load(Ordering::SeqCst), "Pressing 's' while running must switch to paused mode");
        assert!(app.control.step.load(Ordering::SeqCst), "Pressing 's' must request a step");

        // Reset step manually for testing the next transition
        app.control.step.store(false, Ordering::SeqCst);

        // 2. Unpause using 'p'
        app.handle_key(KeyEvent::from(KeyCode::Char('p')));
        assert!(!app.control.paused.load(Ordering::SeqCst), "Pressing 'p' while paused must unpause");

        // 3. Pause using 'p'
        app.handle_key(KeyEvent::from(KeyCode::Char('p')));
        assert!(app.control.paused.load(Ordering::SeqCst), "Pressing 'p' while running must pause");

        // 4. Pressing 's' while already paused
        app.handle_key(KeyEvent::from(KeyCode::Char('s')));
        assert!(app.control.paused.load(Ordering::SeqCst), "Pressing 's' while paused must keep app paused");
        assert!(app.control.step.load(Ordering::SeqCst), "Pressing 's' while paused must request a step");
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
}
