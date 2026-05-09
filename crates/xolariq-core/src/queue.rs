use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::convert::convert_one;
use crate::error::Error;
use crate::format::Format;
use crate::progress::{ProgressEvent, ProgressSink};
use crate::settings::SettingsStore;

/// Stable identifier for a queued job. Surfaced to the UI so the user can
/// cancel a specific job by id (currently the UI only cancels the active
/// job, but the API leaves room for per-job cancel).
pub type JobId = Uuid;

/// Per-job knobs. These are derived from [`crate::Settings`] at the moment
/// the job is enqueued so that mid-queue settings changes don't surprise
/// jobs that are already in flight.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertOptions {
    pub overwrite: bool,
    pub preserve_metadata: bool,
}

impl Default for ConvertOptions {
    fn default() -> Self {
        Self {
            overwrite: false,
            preserve_metadata: true,
        }
    }
}

/// A unit of work for the queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: JobId,
    pub input: PathBuf,
    pub target: Format,
    pub options: ConvertOptions,
}

impl Job {
    pub fn new(input: PathBuf, target: Format, options: ConvertOptions) -> Self {
        Self {
            id: Uuid::new_v4(),
            input,
            target,
            options,
        }
    }
}

/// Internal control messages sent to the worker task.
enum Control {
    Submit(Vec<Job>),
    Shutdown,
}

/// Public handle held by the Tauri layer. Cloning is cheap (Arc-backed) and
/// safe across threads.
#[derive(Clone)]
pub struct QueueHandle {
    tx: mpsc::UnboundedSender<Control>,
    state: Arc<QueueState>,
}

struct QueueState {
    /// Cancel flag for the *currently running* job.
    current_cancel: Mutex<Option<Arc<AtomicBool>>>,
    /// Whether any pending jobs in the local batch should be skipped.
    cancel_remaining: AtomicBool,
}

impl QueueHandle {
    /// Build a queue handle alongside the worker future that drives it.
    ///
    /// The caller is responsible for spawning the returned future on a Tokio
    /// runtime — typically `tauri::async_runtime::spawn` from the desktop
    /// shell. Keeping the spawn out of `xolariq-core` lets the engine stay
    /// runtime-agnostic and avoids the "no reactor running" panic that
    /// surfaces when `tokio::spawn` is called outside of an entered runtime.
    ///
    /// The returned handle can be cloned freely. The worker terminates when
    /// every clone is dropped *or* [`QueueHandle::shutdown`] is called.
    pub fn new(
        settings: Arc<SettingsStore>,
        sink: ProgressSink,
    ) -> (Self, impl std::future::Future<Output = ()> + Send + 'static) {
        let (tx, rx) = mpsc::unbounded_channel::<Control>();
        let state = Arc::new(QueueState {
            current_cancel: Mutex::new(None),
            cancel_remaining: AtomicBool::new(false),
        });

        let worker_state = state.clone();
        let worker = async move {
            run_worker(rx, settings, sink, worker_state).await;
        };

        (Self { tx, state }, worker)
    }

    /// Enqueue one or more jobs as a single batch. Batches show up as one
    /// `QueueStarted`/`QueueFinished` pair in the progress stream.
    pub fn submit(&self, jobs: Vec<Job>) {
        if jobs.is_empty() {
            return;
        }
        let _ = self.tx.send(Control::Submit(jobs));
    }

    /// Signal the currently running job to stop and skip the remainder of
    /// the active batch. Subsequent batches submitted later are unaffected.
    pub fn cancel_current(&self) {
        if let Some(flag) = self.state.current_cancel.lock().as_ref() {
            flag.store(true, Ordering::Relaxed);
        }
        self.state.cancel_remaining.store(true, Ordering::Relaxed);
    }

    pub fn shutdown(&self) {
        let _ = self.tx.send(Control::Shutdown);
    }
}

async fn run_worker(
    mut rx: mpsc::UnboundedReceiver<Control>,
    settings: Arc<SettingsStore>,
    sink: ProgressSink,
    state: Arc<QueueState>,
) {
    while let Some(msg) = rx.recv().await {
        match msg {
            Control::Shutdown => break,
            Control::Submit(jobs) => {
                state.cancel_remaining.store(false, Ordering::Relaxed);
                run_batch(jobs, &settings, &sink, &state).await;
            }
        }
    }
}

async fn run_batch(
    jobs: Vec<Job>,
    settings: &Arc<SettingsStore>,
    sink: &ProgressSink,
    state: &Arc<QueueState>,
) {
    let total = jobs.len();
    sink(ProgressEvent::QueueStarted { total });

    let mut successes = 0usize;
    let mut failures = 0usize;
    let mut cancelled = false;

    for (index, job) in jobs.into_iter().enumerate() {
        if state.cancel_remaining.load(Ordering::Relaxed) {
            cancelled = true;
            sink(ProgressEvent::JobCancelled { id: job.id });
            failures += 1;
            continue;
        }

        let cancel_flag = Arc::new(AtomicBool::new(false));
        *state.current_cancel.lock() = Some(cancel_flag.clone());

        let snapshot = settings.snapshot();
        let id = job.id;
        let job_sink = sink.clone();
        let job_id = id;

        // Pre-compute the output path so we can include it in JobStarted; the
        // converter will recompute internally but the recomputation is cheap
        // and does not race because the queue is sequential.
        let preview_output =
            match crate::output::resolve_output_path(&job.input, job.target, &snapshot) {
                Ok(p) => p,
                Err(err) => {
                    sink(ProgressEvent::JobFailed {
                        id,
                        error: err.to_string(),
                    });
                    failures += 1;
                    continue;
                }
            };

        sink(ProgressEvent::JobStarted {
            id,
            index,
            total,
            input: job.input.clone(),
            output: preview_output,
        });

        let on_progress = move |percent: Option<f32>| {
            job_sink(ProgressEvent::JobProgress {
                id: job_id,
                percent,
            });
        };

        let result = convert_one(
            &job.input,
            job.target,
            &job.options,
            &snapshot,
            cancel_flag.clone(),
            on_progress,
        )
        .await;

        // Drop the per-job cancel reference so a later cancel_current() does
        // not target a stale job.
        *state.current_cancel.lock() = None;

        match result {
            Ok(outcome) => {
                successes += 1;
                sink(ProgressEvent::JobFinished {
                    id,
                    output: outcome.output,
                });
            }
            Err(Error::Cancelled) => {
                cancelled = true;
                failures += 1;
                sink(ProgressEvent::JobCancelled { id });
            }
            Err(err) => {
                failures += 1;
                sink(ProgressEvent::JobFailed {
                    id,
                    error: err.to_string(),
                });
            }
        }
    }

    sink(ProgressEvent::QueueFinished {
        successes,
        failures,
        cancelled,
    });
}
