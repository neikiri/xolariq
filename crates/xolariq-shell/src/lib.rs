//! Windows shell integration for Xolariq.
//!
//! This crate is split into two layers:
//!
//! * A platform-agnostic [`ShellIntegration`] trait describing what the rest
//!   of the app needs (install / uninstall / status).
//! * A Windows-only implementation in [`windows`] that writes the actual
//!   registry keys for the cascading "Convert with Xolariq" submenu.
//!
//! On non-Windows targets the crate still compiles, but every method on the
//! integration returns [`ShellError::Unsupported`]. This lets the workspace
//! be checked / formatted / tested on developer Linux/macOS machines without
//! conditional-compilation noise leaking into the rest of the app.

use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShellError {
    #[error("shell integration is not supported on this platform")]
    Unsupported,

    #[error("registry error: {0}")]
    Registry(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, ShellError>;

/// What the rest of the app needs from a shell integration backend.
///
/// Implementations install / uninstall the OS-level context-menu hooks that
/// route right-click events back into the running Xolariq binary.
pub trait ShellIntegration {
    /// Install the context-menu entries for the currently running user.
    /// Per-user installs do not require admin privileges.
    fn install(&self, exe_path: &std::path::Path) -> Result<()>;

    /// Remove every context-menu entry created by [`Self::install`].
    fn uninstall(&self) -> Result<()>;

    /// Return `true` if the context-menu entries are currently installed.
    fn is_installed(&self) -> Result<bool>;
}

/// The default integration for the host platform. On Windows this returns
/// the registry-based implementation; everywhere else it returns a stub
/// that fails fast with [`ShellError::Unsupported`].
pub fn default_integration() -> Box<dyn ShellIntegration + Send + Sync> {
    #[cfg(windows)]
    {
        Box::new(windows::WindowsShellIntegration::new())
    }
    #[cfg(not(windows))]
    {
        Box::new(stub::StubShellIntegration)
    }
}

/// Convenience wrapper used by the bundled CLI installer/uninstaller.
pub fn current_exe() -> Result<PathBuf> {
    Ok(std::env::current_exe()?)
}

#[cfg(windows)]
pub mod windows;

#[cfg(not(windows))]
mod stub {
    use super::*;

    pub struct StubShellIntegration;

    impl ShellIntegration for StubShellIntegration {
        fn install(&self, _exe_path: &std::path::Path) -> Result<()> {
            Err(ShellError::Unsupported)
        }
        fn uninstall(&self) -> Result<()> {
            Err(ShellError::Unsupported)
        }
        fn is_installed(&self) -> Result<bool> {
            Ok(false)
        }
    }
}

/// Re-exported so callers don't need to know which file kinds Xolariq supports
/// when building UI menus.
pub use xolariq_core::{FileKind, Format, FormatList};
