//! Tauri command handlers exposed to the frontend.
//!
//! Each command is a thin adapter over either [`xolariq_core`] or
//! [`xolariq_shell`]; we keep them small so the JS side has a clean,
//! discoverable API surface.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use xolariq_core::settings::Settings;
use xolariq_core::{Format, FormatList};

use crate::state::AppState;

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Settings {
    state.settings.snapshot()
}

#[tauri::command]
pub fn update_settings(
    new_settings: Settings,
    state: State<'_, AppState>,
) -> Result<Settings, String> {
    state
        .settings
        .replace(new_settings)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reset_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    state.settings.reset().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn pick_output_folder(app: AppHandle) -> Option<PathBuf> {
    use tauri_plugin_dialog::DialogExt;

    // The dialog plugin's blocking pick_folder() returns Option<PathBuf>.
    app.dialog().file().blocking_pick_folder().map(|p| {
        // FileDialogPath -> PathBuf
        match p {
            tauri_plugin_dialog::FilePath::Path(path) => path,
            tauri_plugin_dialog::FilePath::Url(url) => PathBuf::from(url.to_string()),
        }
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShellStatus {
    pub installed: bool,
    pub supported: bool,
}

#[tauri::command]
pub fn shell_integration_status() -> ShellStatus {
    let integration = xolariq_shell::default_integration();
    match integration.is_installed() {
        Ok(installed) => ShellStatus {
            installed,
            supported: true,
        },
        Err(xolariq_shell::ShellError::Unsupported) => ShellStatus {
            installed: false,
            supported: false,
        },
        Err(err) => {
            tracing::warn!(?err, "failed to read shell integration status");
            ShellStatus {
                installed: false,
                supported: true,
            }
        }
    }
}

#[tauri::command]
pub fn install_shell_integration() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    xolariq_shell::default_integration()
        .install(&exe)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn uninstall_shell_integration() -> Result<(), String> {
    xolariq_shell::default_integration()
        .uninstall()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cancel_current_job(state: State<'_, AppState>) {
    state.queue.cancel_current();
}

#[tauri::command]
pub fn supported_targets_for_extension(extension: String) -> Vec<TargetEntry> {
    let Some(format) = Format::from_extension(&extension) else {
        return Vec::new();
    };
    FormatList::targets_for(format)
        .into_iter()
        .map(|f| TargetEntry {
            extension: f.extension().to_string(),
            label: f.label().to_string(),
        })
        .collect()
}

#[tauri::command]
pub fn open_settings_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("settings window not registered".into())
    }
}

/// Open an external URL in the user's default browser.
///
/// We accept a small allow-list of HTTPS hosts rather than letting the
/// frontend pass arbitrary URLs into the shell plugin — keeps the API
/// surface tight even though the link is currently driven by static UI.
#[tauri::command]
pub fn open_external_url(app: AppHandle, url: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;

    const ALLOWED_PREFIXES: &[&str] = &[
        "https://github.com/neikiri/",
        "https://xolariq.dev/",
        "https://docs.xolariq.dev/",
    ];
    if !ALLOWED_PREFIXES.iter().any(|p| url.starts_with(p)) {
        return Err(format!("URL not in allow-list: {url}"));
    }
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| e.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TargetEntry {
    pub extension: String,
    pub label: String,
}
