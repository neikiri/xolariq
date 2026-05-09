use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::queue::JobId;

/// All events emitted by the queue. The Tauri layer forwards these to the UI
/// over a single window event channel; the same enum is reused for any future
/// CLI/headless subscribers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProgressEvent {
    QueueStarted {
        total: usize,
    },
    JobStarted {
        id: JobId,
        index: usize,
        total: usize,
        input: PathBuf,
        output: PathBuf,
    },
    JobProgress {
        id: JobId,
        /// Fraction of completion in `[0.0, 1.0]`. May be `None` when the
        /// underlying tool does not expose progress information.
        percent: Option<f32>,
    },
    JobFinished {
        id: JobId,
        output: PathBuf,
    },
    JobFailed {
        id: JobId,
        error: String,
    },
    JobCancelled {
        id: JobId,
    },
    QueueFinished {
        successes: usize,
        failures: usize,
        cancelled: bool,
    },
}

/// Type-erased progress emitter passed into individual converters.
///
/// Wrapped in `Arc` so it can be cloned cheaply across async boundaries.
pub type ProgressSink = Arc<dyn Fn(ProgressEvent) + Send + Sync + 'static>;

pub fn noop_sink() -> ProgressSink {
    Arc::new(|_| {})
}
