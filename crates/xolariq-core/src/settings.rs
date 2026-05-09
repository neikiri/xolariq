use std::fs;
use std::path::PathBuf;

use directories::ProjectDirs;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::tools::{resolve_tool, Tool};

/// What to do when the resolved output path already exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutputMode {
    /// Replace the existing file at the resolved path.
    Overwrite,
    /// Append `(1)`, `(2)`, ... to the file stem until a free path is found.
    #[default]
    Rename,
}

/// Persisted user settings. Lives in `<config_dir>/Xolariq/settings.json`.
///
/// Settings are deliberately small and forwards-compatible: unknown fields
/// are tolerated by serde defaults so that adding fields in a future version
/// does not break older config files.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// When `None`, the converted file is written next to the input.
    pub output_folder: Option<PathBuf>,
    pub output_mode: OutputMode,
    /// When true, container-level metadata (tags, EXIF, etc.) is preserved
    /// when the target format supports it.
    pub preserve_metadata: bool,
    /// Whether the Windows context-menu entry is currently registered.
    pub context_menu_enabled: bool,
    /// Optional override for the `ffmpeg` executable. Falls back to PATH.
    pub ffmpeg_path: Option<PathBuf>,
    /// Optional override for the `pandoc` executable. Falls back to PATH.
    pub pandoc_path: Option<PathBuf>,
    /// Optional override for the `7z` executable. Falls back to PATH.
    pub seven_zip_path: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            output_folder: None,
            output_mode: OutputMode::Rename,
            preserve_metadata: true,
            context_menu_enabled: true,
            ffmpeg_path: None,
            pandoc_path: None,
            seven_zip_path: None,
        }
    }
}

impl Settings {
    /// Resolved path of the settings file. Always under the user's
    /// per-application config directory.
    pub fn config_path() -> Result<PathBuf> {
        let dirs = ProjectDirs::from("dev", "Xolariq", "Xolariq")
            .ok_or_else(|| Error::Settings("could not resolve config directory".into()))?;
        let dir = dirs.config_dir().to_path_buf();
        Ok(dir.join("settings.json"))
    }

    /// Load settings, creating defaults if the file is missing or unreadable.
    /// A read failure is logged and falls back to defaults so a corrupt file
    /// never blocks the app from starting.
    pub fn load() -> Settings {
        match Self::try_load() {
            Ok(s) => s,
            Err(err) => {
                tracing::warn!(?err, "failed to load settings; using defaults");
                Settings::default()
            }
        }
    }

    pub fn try_load() -> Result<Settings> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(Settings::default());
        }
        let raw = fs::read_to_string(&path)?;
        let parsed: Settings = serde_json::from_str(&raw)?;
        Ok(parsed)
    }

    /// Persist settings atomically (write to a temp file, then rename).
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("json.tmp");
        let serialized = serde_json::to_string_pretty(self)?;
        fs::write(&tmp, serialized)?;
        fs::rename(&tmp, &path)?;
        Ok(())
    }

    /// Resolved path of the `ffmpeg` binary.
    ///
    /// Uses [`crate::tools::resolve_tool`] so that the bundled sidecar
    /// (`ffmpeg.exe` next to `xolariq.exe`) is preferred over a `PATH`
    /// lookup, falling back to a bare `"ffmpeg"` command name when no
    /// sidecar is present.
    pub fn ffmpeg_executable(&self) -> PathBuf {
        resolve_tool(Tool::Ffmpeg, self.ffmpeg_path.as_deref())
    }

    /// Resolved path of the `pandoc` binary. See [`Self::ffmpeg_executable`].
    pub fn pandoc_executable(&self) -> PathBuf {
        resolve_tool(Tool::Pandoc, self.pandoc_path.as_deref())
    }

    /// Resolved path of the `p7za` (7-Zip standalone) binary. See
    /// [`Self::ffmpeg_executable`].
    pub fn seven_zip_executable(&self) -> PathBuf {
        resolve_tool(Tool::SevenZip, self.seven_zip_path.as_deref())
    }
}

/// Thread-safe, in-memory settings cache. The Tauri layer holds one of these
/// in its managed state so that every queue worker reads consistent settings
/// without performing disk I/O on the hot path.
#[derive(Debug, Default)]
pub struct SettingsStore {
    inner: RwLock<Settings>,
}

impl SettingsStore {
    pub fn new(initial: Settings) -> Self {
        Self {
            inner: RwLock::new(initial),
        }
    }

    pub fn load_or_default() -> Self {
        Self::new(Settings::load())
    }

    pub fn snapshot(&self) -> Settings {
        self.inner.read().clone()
    }

    pub fn update<F>(&self, mutate: F) -> Result<Settings>
    where
        F: FnOnce(&mut Settings),
    {
        let mut guard = self.inner.write();
        mutate(&mut guard);
        guard.save()?;
        Ok(guard.clone())
    }

    pub fn replace(&self, new_settings: Settings) -> Result<Settings> {
        let mut guard = self.inner.write();
        *guard = new_settings;
        guard.save()?;
        Ok(guard.clone())
    }

    pub fn reset(&self) -> Result<Settings> {
        self.replace(Settings::default())
    }
}
