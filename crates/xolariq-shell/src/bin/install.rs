//! Standalone installer for Xolariq's Windows context-menu entries.
//!
//! Useful for CI smoke tests and for power users who want to wire the
//! integration without launching the Tauri UI. Equivalent to clicking the
//! "Enable context menu" toggle in the Settings window.
//!
//! Usage: `xolariq-shell-install <path-to-xolariq.exe>`

use std::path::PathBuf;
use std::process::ExitCode;

use xolariq_shell::{default_integration, ShellError};

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let exe = match args.next() {
        Some(p) => PathBuf::from(p),
        None => match std::env::current_exe() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("xolariq-shell-install: cannot determine exe path: {e}");
                return ExitCode::from(2);
            }
        },
    };

    if !exe.exists() {
        eprintln!(
            "xolariq-shell-install: exe path does not exist: {}",
            exe.display()
        );
        return ExitCode::from(2);
    }

    let integration = default_integration();
    match integration.install(&exe) {
        Ok(()) => {
            println!("Installed Xolariq context-menu entries for the current user.");
            ExitCode::SUCCESS
        }
        Err(ShellError::Unsupported) => {
            eprintln!("xolariq-shell-install: this platform does not support shell integration.");
            ExitCode::from(3)
        }
        Err(e) => {
            eprintln!("xolariq-shell-install: {e}");
            ExitCode::FAILURE
        }
    }
}
