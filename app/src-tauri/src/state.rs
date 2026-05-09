//! Tauri bootstrap and shared application state.
//!
//! Responsibilities:
//!
//! * Spin up the Tauri runtime with the right plugins (notification,
//!   dialog, single-instance, opener).
//! * Build a [`xolariq_core::QueueHandle`] whose progress sink fans out to
//!   the progress window via Tauri events and to native Windows toasts.
//! * Decide which window to show on startup based on parsed CLI args.
//! * Handle the single-instance callback so context-menu invocations made
//!   while Xolariq is already running enqueue more work into the existing
//!   queue instead of spawning a second process.

use std::sync::Arc;

use parking_lot::Mutex;
use tauri::{AppHandle, Emitter, Manager, WebviewWindow};
use tauri_plugin_single_instance::init as single_instance;

use xolariq_core::{settings::SettingsStore, ConvertOptions, Job, ProgressEvent, QueueHandle};

use crate::cli::{self, Cli};
use crate::commands;
use crate::notify;

pub struct AppState {
    pub queue: QueueHandle,
    pub settings: Arc<SettingsStore>,
    /// Cached most-recent progress so that the progress window, when it is
    /// (re-)opened, can immediately render the live state instead of
    /// waiting for the next event.
    pub last_progress: Mutex<Option<ProgressEvent>>,
}

pub fn launch_tauri(args: Cli) -> anyhow::Result<()> {
    let initial_args = args.clone();

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(single_instance(|app, argv, _cwd| {
            // Re-parse argv from the second instance and forward to the queue.
            let cli = cli::parse_args(argv.into_iter());
            if let Err(err) = enqueue_from_cli(app, &cli) {
                tracing::warn!(?err, "failed to enqueue jobs from secondary instance");
            }
            // Bring an existing window to the foreground.
            if let Some(window) = pick_visible_window(app) {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::update_settings,
            commands::reset_settings,
            commands::pick_output_folder,
            commands::install_shell_integration,
            commands::uninstall_shell_integration,
            commands::shell_integration_status,
            commands::cancel_current_job,
            commands::supported_targets_for_extension,
            commands::open_settings_window,
            commands::open_external_url,
        ])
        .setup(move |app| {
            // Detect first launch: the settings file does not exist yet.
            let first_launch = xolariq_core::settings::Settings::config_path()
                .map(|p| !p.exists())
                .unwrap_or(false);

            let settings = Arc::new(SettingsStore::load_or_default());
            let app_handle = app.handle().clone();
            let progress_sink = build_progress_sink(app_handle.clone(), settings.clone());
            // The queue worker runs on Tauri's bundled Tokio runtime so that
            // converters can use tokio::process / tokio::sync without a
            // separate runtime — and so that we don't panic with
            // "no reactor running" when `setup` runs outside an entered
            // Tokio context.
            let (queue, worker) = QueueHandle::new(settings.clone(), progress_sink);
            tauri::async_runtime::spawn(worker);

            let state = AppState {
                queue,
                settings: settings.clone(),
                last_progress: Mutex::new(None),
            };
            app.manage(state);

            // Auto-enable the Windows context-menu integration on first
            // launch so that newly installed users get the right-click
            // workflow out of the box. Also re-register when settings say
            // the integration should be active but the registry keys are
            // missing (e.g. after a reinstall that preserved settings.json
            // but cleared the old registry tree).
            let should_register = first_launch || {
                let want = settings.snapshot().context_menu_enabled;
                let have = xolariq_shell::default_integration()
                    .is_installed()
                    .unwrap_or(false);
                want && !have
            };
            if should_register {
                if let Err(err) = auto_enable_shell_integration(&settings) {
                    tracing::warn!(?err, "auto-enable shell integration failed");
                }
            }

            choose_initial_window(&app_handle, &initial_args)?;
            if initial_args.has_jobs() {
                enqueue_from_cli(&app_handle, &initial_args)?;
            }

            Ok(())
        });

    builder
        .run(tauri::generate_context!())
        .map_err(|e| anyhow::anyhow!(e))
}

fn build_progress_sink(app: AppHandle, settings: Arc<SettingsStore>) -> xolariq_core::ProgressSink {
    Arc::new(move |event: ProgressEvent| {
        // Cache so a freshly opened progress window can hydrate state.
        if let Some(state) = app.try_state::<AppState>() {
            *state.last_progress.lock() = Some(event.clone());
        }

        // Forward to the progress window. Errors are swallowed because the
        // window may not exist yet (headless mode) or may be closed while
        // a job is finishing — neither case is fatal.
        if let Err(err) = app.emit("xolariq:progress", &event) {
            tracing::debug!(?err, "failed to emit xolariq:progress");
        }

        // Native toast for terminal events only. A toast per-progress would
        // spam the action centre.
        notify::handle_event(&app, &settings, &event);
    })
}

fn choose_initial_window(app: &AppHandle, args: &Cli) -> tauri::Result<()> {
    if args.has_jobs() {
        if let Some(window) = app.get_webview_window("progress") {
            window.show()?;
            window.set_focus()?;
        }
    } else {
        if let Some(window) = app.get_webview_window("settings") {
            window.show()?;
            window.set_focus()?;
        }
    }
    Ok(())
}

fn pick_visible_window(app: &AppHandle) -> Option<WebviewWindow> {
    if let Some(w) = app.get_webview_window("progress") {
        if matches!(w.is_visible(), Ok(true)) {
            return Some(w);
        }
    }
    app.get_webview_window("settings")
}

/// Best-effort auto-install of the shell integration on first launch.
/// Persists the `context_menu_enabled` flag so the Settings UI reflects
/// the actual state.
fn auto_enable_shell_integration(settings: &Arc<SettingsStore>) -> anyhow::Result<()> {
    let exe = std::env::current_exe()?;
    xolariq_shell::default_integration()
        .install(&exe)
        .map_err(|e| anyhow::anyhow!("install shell integration: {e}"))?;
    let _ = settings.update(|s| {
        s.context_menu_enabled = true;
    });
    Ok(())
}

pub fn enqueue_from_cli(app: &AppHandle, args: &Cli) -> anyhow::Result<()> {
    let target = match args.target {
        Some(t) => t,
        None => return Ok(()),
    };
    if args.inputs.is_empty() {
        return Ok(());
    }

    let state = app
        .try_state::<AppState>()
        .ok_or_else(|| anyhow::anyhow!("app state not initialised"))?;
    let settings = state.settings.snapshot();
    let options = ConvertOptions {
        overwrite: matches!(settings.output_mode, xolariq_core::OutputMode::Overwrite),
        preserve_metadata: settings.preserve_metadata,
    };

    let jobs = args
        .inputs
        .iter()
        .cloned()
        .map(|input| Job::new(input, target, options.clone()))
        .collect::<Vec<_>>();

    state.queue.submit(jobs);

    // Bring the progress window to the foreground for new batches submitted
    // by a context-menu invocation made while Xolariq was already running.
    if let Some(window) = app.get_webview_window("progress") {
        let _ = window.show();
        let _ = window.set_focus();
    }
    Ok(())
}
