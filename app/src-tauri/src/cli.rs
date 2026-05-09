//! Command-line argument parsing for the Tauri binary.
//!
//! The binary supports two intentionally narrow modes:
//!
//! * Conversion mode (the common case, invoked by Explorer): one or more
//!   `--input` paths plus a single `--target` format.
//! * Subcommands for installing or removing the shell integration. These
//!   are the same operations the Settings UI exposes — having a CLI version
//!   keeps CI and headless deployments scriptable.
//!
//! When the binary is launched with no arguments at all (e.g. via the Start
//! menu) we fall through to the GUI's settings window.

use std::ffi::OsString;
use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

use xolariq_core::Format;

#[derive(Debug, Parser, Clone)]
#[command(name = "xolariq", about = "Right-click file conversion for Windows.")]
pub struct Cli {
    /// Target output format (e.g. mp3, mp4, png).
    #[arg(long, value_parser = parse_format)]
    pub target: Option<Format>,

    /// Input file(s). May be supplied multiple times. Each `--input` adds
    /// one job to the queue; the queue runs them sequentially.
    #[arg(long = "input", value_name = "PATH")]
    pub inputs: Vec<PathBuf>,

    /// Skip the progress window — write to stderr only. Mostly useful for
    /// scripted invocations and tests.
    #[arg(long, default_value_t = false)]
    pub headless: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand, Clone)]
pub enum Command {
    /// Register the Windows context-menu entries for the current user.
    InstallShell,
    /// Remove the Windows context-menu entries for the current user.
    UninstallShell,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum FormatArg {
    // Used only to keep clap derive happy when we want completion. In practice
    // we use `parse_format` so the user can pass any format spelling we accept.
}

fn parse_format(raw: &str) -> Result<Format, String> {
    Format::from_extension(raw)
        .ok_or_else(|| format!("unknown target format '{raw}'. Try one of mp3, mp4, png, ..."))
}

/// Public parse entry point. Falls back to a fully-defaulted [`Cli`] on
/// failure so the GUI still launches when a mistyped command is supplied.
pub fn parse_args<I, T>(args: I) -> Cli
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    Cli::try_parse_from(args).unwrap_or_else(|err| {
        // Keep the diagnostic visible on the console for power users while
        // still allowing the GUI to come up.
        eprintln!("{err}");
        Cli {
            target: None,
            inputs: Vec::new(),
            headless: false,
            command: None,
        }
    })
}

impl Cli {
    pub fn has_jobs(&self) -> bool {
        self.target.is_some() && !self.inputs.is_empty()
    }
}
