//! Resolution of external conversion tools (ffmpeg, pandoc, 7-Zip).
//!
//! Lookup order:
//!
//! 1. **Explicit user override** in [`crate::settings::Settings`] (e.g.
//!    `ffmpeg_path`).
//! 2. **`XOLARIQ_TOOLS_DIR` env var** — handy for dev shells and CI runs
//!    that don't sit in the bundle layout.
//! 3. **Sidecar directory** next to the running executable. The Tauri MSI
//!    bundle places `ffmpeg.exe`, `pandoc.exe`, `p7za.exe` directly next to
//!    `xolariq.exe` via `bundle.windows.externalBin`, so this is the
//!    common case for installed end-users.
//! 4. **`PATH` fallback** — the bare tool name is returned (e.g.
//!    `"ffmpeg"`) and `tokio::process::Command` performs the lookup.
//!
//! The resolver is intentionally I/O-light: it does at most one
//! `try_exists` per call so it can run on every conversion without
//! noticeable overhead.

use std::path::{Path, PathBuf};

/// Logical name for a bundled tool. We keep this enum small and explicit so
/// that adding a new tool requires touching exactly one match arm here and
/// one Wix payload entry.
#[derive(Debug, Clone, Copy)]
pub enum Tool {
    Ffmpeg,
    Pandoc,
    SevenZip,
}

impl Tool {
    /// File-stem under which the tool ships in the sidecar directory and
    /// under which it's typically discoverable on `PATH`.
    pub fn stem(self) -> &'static str {
        match self {
            Tool::Ffmpeg => "ffmpeg",
            Tool::Pandoc => "pandoc",
            // p7za = the standalone command-line build of 7-Zip, prefixed
            // with 'p' so the WiX component ID is a valid identifier
            // (identifiers may not start with a digit).
            Tool::SevenZip => "p7za",
        }
    }
}

/// Resolve the path for a tool, given an optional user override. The
/// returned path may be an absolute path on disk or a bare command name
/// suitable for `PATH` lookup — callers should pass it directly to
/// `tokio::process::Command::new` either way.
pub fn resolve_tool(tool: Tool, override_path: Option<&Path>) -> PathBuf {
    if let Some(p) = override_path {
        return p.to_path_buf();
    }
    if let Some(p) = lookup_in_env_dir(tool) {
        return p;
    }
    // For 7-Zip, prefer the system-installed `7z.exe` over the bundled
    // `p7za.exe` sidecar because the bundled binary may carry an
    // elevation manifest (OS error 740) that prevents non-admin usage.
    #[cfg(windows)]
    if matches!(tool, Tool::SevenZip) {
        if let Some(p) = lookup_system_7zip() {
            return p;
        }
    }
    if let Some(p) = lookup_next_to_exe(tool) {
        return p;
    }
    // Fall back to a bare command name and let the OS resolve via PATH.
    PathBuf::from(tool.stem())
}

/// Check well-known 7-Zip installation directories on Windows.
#[cfg(windows)]
fn lookup_system_7zip() -> Option<PathBuf> {
    for env_var in ["ProgramFiles", "ProgramFiles(x86)", "ProgramW6432"] {
        if let Ok(dir) = std::env::var(env_var) {
            let candidate = Path::new(&dir).join("7-Zip").join("7z.exe");
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn lookup_in_env_dir(tool: Tool) -> Option<PathBuf> {
    let dir = std::env::var_os("XOLARIQ_TOOLS_DIR")?;
    candidate_in(Path::new(&dir), tool)
}

fn lookup_next_to_exe(tool: Tool) -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    candidate_in(dir, tool)
}

fn candidate_in(dir: &Path, tool: Tool) -> Option<PathBuf> {
    let stem = tool.stem();
    let candidate = if cfg!(windows) {
        dir.join(format!("{stem}.exe"))
    } else {
        dir.join(stem)
    };
    if candidate.is_file() {
        Some(candidate)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn override_takes_priority() {
        let override_path = PathBuf::from("/opt/custom/ffmpeg");
        let resolved = resolve_tool(Tool::Ffmpeg, Some(&override_path));
        assert_eq!(resolved, override_path);
    }

    #[test]
    fn falls_back_to_bare_name() {
        // No override, no XOLARIQ_TOOLS_DIR set in this test, exe dir
        // unlikely to contain a "ffmpeg" binary in a CI runner.
        let prev = std::env::var_os("XOLARIQ_TOOLS_DIR");
        // SAFETY: tests run sequentially per process by default; we restore
        // the previous value at the end. If you run with --test-threads
        // greater than 1 and another test also pokes this var, this test
        // is brittle — but no other test in the crate does.
        std::env::remove_var("XOLARIQ_TOOLS_DIR");

        let resolved = resolve_tool(Tool::Pandoc, None);
        // Either the bare name (PATH fallback) or an absolute path next to
        // the test runner exe is acceptable; we only verify the file stem
        // matches.
        let stem = resolved
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        assert_eq!(stem, "pandoc");

        if let Some(prev) = prev {
            std::env::set_var("XOLARIQ_TOOLS_DIR", prev);
        }
    }
}
