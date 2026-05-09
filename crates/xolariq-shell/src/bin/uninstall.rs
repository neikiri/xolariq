//! Standalone uninstaller. Removes every registry key created by
//! `xolariq-shell-install`. Per-user only, so no admin elevation is required.

use std::process::ExitCode;

use xolariq_shell::{default_integration, ShellError};

fn main() -> ExitCode {
    let integration = default_integration();
    match integration.uninstall() {
        Ok(()) => {
            println!("Removed Xolariq context-menu entries for the current user.");
            ExitCode::SUCCESS
        }
        Err(ShellError::Unsupported) => {
            eprintln!("xolariq-shell-uninstall: this platform does not support shell integration.");
            ExitCode::from(3)
        }
        Err(e) => {
            eprintln!("xolariq-shell-uninstall: {e}");
            ExitCode::FAILURE
        }
    }
}
