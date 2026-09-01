//! In-memory ring buffer and custom `tracing_subscriber::Layer` for the TUI log pane.

use std::collections::VecDeque;
use std::fmt::Write as _;
use std::sync::{Arc, Mutex};
use tracing::Subscriber;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;

/// Fixed-capacity FIFO ring buffer storing up to 500 log lines.
#[derive(Debug, Clone)]
pub struct LogRing {
    capacity: usize,
    entries: VecDeque<String>,
}

impl Default for LogRing {
    fn default() -> Self {
        Self::new(500)
    }
}

impl LogRing {
    /// Create a new `LogRing` with the specified capacity.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            entries: VecDeque::with_capacity(capacity),
        }
    }

    /// Push an entry into the ring buffer, evicting the oldest if capacity is exceeded.
    pub fn push_back(&mut self, entry: String) {
        if self.capacity == 0 {
            return;
        }
        if self.entries.len() >= self.capacity {
            let _ = self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    /// Return entries as a slice-like iterator or collection.
    #[must_use]
    pub const fn entries(&self) -> &VecDeque<String> {
        &self.entries
    }

    /// Return the count of current entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Return `true` if the ring buffer contains no entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Visitor that formats event fields into ` name=value`.
struct FieldVisitor<'a>(&'a mut String);

impl tracing::field::Visit for FieldVisitor<'_> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        let _ = write!(&mut self.0, " {}={value:?}", field.name());
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        let _ = write!(&mut self.0, " {}={value}", field.name());
    }
}

/// A `tracing_subscriber::Layer` that formats tracing events and pushes them to a `LogRing`.
#[derive(Debug, Clone)]
pub struct LogPaneLayer {
    sink: Arc<Mutex<LogRing>>,
}

impl LogPaneLayer {
    /// Construct a new `LogPaneLayer` writing to the given shared `LogRing`.
    #[must_use]
    pub const fn new(sink: Arc<Mutex<LogRing>>) -> Self {
        Self { sink }
    }
}

impl<S> Layer<S> for LogPaneLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        let mut buf = String::with_capacity(128);
        let metadata = event.metadata();
        let _ = write!(
            &mut buf,
            "[{}] {}: {}",
            metadata.level(),
            metadata.target(),
            metadata.name(),
        );

        let mut visitor = FieldVisitor(&mut buf);
        event.record(&mut visitor);

        if let Ok(mut guard) = self.sink.lock() {
            guard.push_back(buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber::prelude::*;

    #[test]
    fn ring_buffer_eviction_at_500_entries() {
        let mut ring = LogRing::new(500);
        for i in 0..505 {
            ring.push_back(format!("entry {i}"));
        }
        assert_eq!(ring.len(), 500);
        assert_eq!(ring.entries().front().map(String::as_str), Some("entry 5"));
        assert_eq!(ring.entries().back().map(String::as_str), Some("entry 504"));
    }

    #[test]
    fn log_pane_layer_formats_events() {
        let ring = Arc::new(Mutex::new(LogRing::new(10)));
        let layer = LogPaneLayer::new(Arc::clone(&ring));
        let subscriber = tracing_subscriber::registry().with(layer);

        tracing::subscriber::with_default(subscriber, || {
            tracing::info!(node_count = 10, "graph_loaded");
        });

        let Ok(guard) = ring.lock() else {
            return;
        };
        assert_eq!(guard.len(), 1);
        let line = &guard.entries()[0];
        assert!(line.contains("[INFO]"));
        assert!(line.contains("graph_loaded"));
        assert!(line.contains("node_count=10") || line.contains("node_count = 10"));
    }
}
