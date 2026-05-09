// On Windows, suppress the console window when launched via the context menu.
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod cli;
mod commands;
mod notify;
mod state;

fn main() {
    if let Err(err) = run() {
        // Last-ditch error surfacing — at this point neither tracing nor the
        // notification plugin are guaranteed to be initialised.
        eprintln!("xolariq: fatal: {err:#}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    init_tracing();

    let args = cli::parse_args(std::env::args_os());

    // CLI-only paths: install/uninstall the shell integration and exit
    // without ever spinning up Tauri. Useful for CI and the bundled
    // `xolariq-shell-install`/`-uninstall` companion binaries.
    if let Some(cli::Command::InstallShell) = args.command.as_ref() {
        let exe = std::env::current_exe()?;
        return install_shell(&exe);
    }
    if let Some(cli::Command::UninstallShell) = args.command.as_ref() {
        return uninstall_shell();
    }

    state::launch_tauri(args)
}

fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};
    let filter = EnvFilter::try_from_env("XOLARIQ_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = fmt().with_env_filter(filter).with_target(false).try_init();
}

fn install_shell(exe: &std::path::Path) -> anyhow::Result<()> {
    let integration = xolariq_shell::default_integration();
    integration
        .install(exe)
        .map_err(|e| anyhow::anyhow!("install shell integration: {e}"))?;
    println!("Xolariq context-menu integration installed.");
    Ok(())
}

fn uninstall_shell() -> anyhow::Result<()> {
    let integration = xolariq_shell::default_integration();
    integration
        .uninstall()
        .map_err(|e| anyhow::anyhow!("uninstall shell integration: {e}"))?;
    println!("Xolariq context-menu integration removed.");
    Ok(())
}
