//! Background worker thread for executing Leiden runs asynchronously.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, SyncSender, TrySendError, sync_channel};
use std::thread::{self, JoinHandle};

use leiden::{CsrGraph, Leiden, LeidenEvent, LeidenParameters, RunResult};

/// Spawn a worker thread executing `Leiden::run` with bounded event communication.
#[must_use]
pub fn spawn_leiden_worker(
    graph: CsrGraph<String>,
    params: LeidenParameters,
    paused: Arc<AtomicBool>,
    step: Arc<AtomicBool>,
    abort: Arc<AtomicBool>,
) -> (
    Receiver<LeidenEvent>,
    JoinHandle<Result<RunResult<String>, leiden::LeidenError>>,
) {
    let (tx, rx) = sync_channel::<LeidenEvent>(1024);
    let (proxy_tx, proxy_rx) = std::sync::mpsc::channel::<LeidenEvent>();

    // Spawn an adapter/proxy forwarder that bridges mpsc to bounded sync_channel with Throttled emission
    let forwarder_handle = thread::spawn(move || {
        let mut dropped_count: u64 = 0;
        while let Ok(event) = proxy_rx.recv() {
            match tx.try_send(event) {
                Ok(()) => {
                    dropped_count = 0;
                }
                Err(TrySendError::Full(_)) => {
                    dropped_count += 1;
                    let _ = tx.try_send(LeidenEvent::Throttled {
                        dropped: dropped_count,
                    });
                }
                Err(TrySendError::Disconnected(_)) => {
                    tracing::warn!("TUI event receiver disconnected; dropping event");
                    break;
                }
            }
        }
    });

    let control_flags = Arc::new(leiden::orchestrator::ControlFlags {
        paused,
        step,
        abort,
    });

    let worker_handle = thread::spawn(move || {
        let result = Leiden::new()
            .with_parameters(params)
            .with_event_sink(proxy_tx)
            .with_control_flags(control_flags)
            .run(&graph);

        let _ = forwarder_handle.join();
        result
    });

    (rx, worker_handle)
}

/// A wrapper struct for sending events safely across threads with throttling.
#[derive(Debug, Clone)]
pub struct ThrottledSender {
    tx: SyncSender<LeidenEvent>,
}

impl ThrottledSender {
    /// Create a new `ThrottledSender`.
    #[must_use]
    pub const fn new(tx: SyncSender<LeidenEvent>) -> Self {
        Self { tx }
    }

    /// Send an event, falling back to `LeidenEvent::Throttled` if full, or logging on disconnect.
    pub fn send(&self, event: LeidenEvent) {
        match self.tx.try_send(event) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
                let _ = self.tx.try_send(LeidenEvent::Throttled { dropped: 1 });
            }
            Err(TrySendError::Disconnected(_)) => {
                tracing::warn!("TUI receiver disconnected");
            }
        }
    }
}
