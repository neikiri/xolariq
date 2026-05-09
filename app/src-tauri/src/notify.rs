//! Native Windows notifications for terminal queue events.
//!
//! Only the events the user actually cares about — successful conversion of
//! a single file, failure, or completion of a multi-file batch — generate a
//! toast. Per-file progress events are intentionally silent.

use std::sync::Arc;

use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

use xolariq_core::settings::SettingsStore;
use xolariq_core::ProgressEvent;

pub fn handle_event(app: &AppHandle, _settings: &Arc<SettingsStore>, event: &ProgressEvent) {
    match event {
        ProgressEvent::JobFinished { output, .. } => {
            let body = format!("Saved to {}", output.display());
            send(app, "Xolariq — Conversion complete", &body);
        }
        ProgressEvent::JobFailed { error, .. } => {
            send(app, "Xolariq — Conversion failed", error);
        }
        ProgressEvent::QueueFinished {
            successes,
            failures,
            cancelled,
        } if *successes + *failures > 1 => {
            let title = if *cancelled {
                "Xolariq — Batch cancelled"
            } else if *failures == 0 {
                "Xolariq — Batch complete"
            } else {
                "Xolariq — Batch finished with errors"
            };
            let body = format!("{successes} succeeded, {failures} failed");
            send(app, title, &body);
        }
        _ => {}
    }
}

fn send(app: &AppHandle, title: &str, body: &str) {
    if let Err(err) = app.notification().builder().title(title).body(body).show() {
        tracing::debug!(?err, "failed to show notification");
    }
}
