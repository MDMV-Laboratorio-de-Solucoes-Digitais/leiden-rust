//! Interactive Terminal UI library components for Leiden community detection.

pub mod app;
pub mod error;
pub mod event;
pub mod logging;
pub mod presets;
pub mod simulation;
pub mod ui;
pub mod worker;

pub use app::{App, AppState, FocusPanel};
pub use error::TuiError;
pub use event::{AppAction, map_key_event};
pub use logging::{LogPaneLayer, LogRing};
pub use presets::{PresetDataset, PresetId};
pub use simulation::{ForceSimulation, Point2D};
