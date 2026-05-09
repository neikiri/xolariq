# Tauri App and UI

The Tauri application is the bridge between Windows shell invocations, the Rust conversion engine, and the user interface.

## Responsibilities

The Tauri layer handles:

- Process startup.
- CLI parsing.
- Shell install/uninstall subcommands.
- Single-instance coordination.
- Managed application state.
- Frontend command handlers.
- Progress event forwarding.
- Native notifications.
- Window selection on startup.

It should stay thin. Most conversion decisions belong in the core engine, while Windows shell details belong in the shell integration crates.

## Startup behavior

At startup, the application parses command-line arguments.

There are three major modes:

| Mode | Trigger | Behavior |
| --- | --- | --- |
| Shell command | `install-shell` or `uninstall-shell` | Run shell operation and exit |
| Conversion mode | `--target` with one or more `--input` values | Open progress window and enqueue jobs |
| Normal launch | No conversion arguments | Open settings window |

This allows the same executable to serve users, scripts, and Explorer integration.

## Windows

Xolariq declares two main windows:

| Window | Purpose |
| --- | --- |
| Settings | Configure output behavior, shell integration, and tool paths |
| Progress | Display active conversion status and cancellation controls |

The settings window is the default when the app is launched normally. The progress window is shown when jobs are submitted.

## Frontend stack

The frontend intentionally avoids a heavy build pipeline.

It uses:

- Static HTML.
- Shared CSS.
- Vanilla JavaScript.
- Tauri global APIs.

This keeps the application easy to package and avoids introducing a Node-based build step for a small utility UI.

## Tauri commands

The backend exposes focused commands to the frontend:

| Command | Purpose |
| --- | --- |
| `get_settings` | Read current settings |
| `update_settings` | Persist new settings |
| `reset_settings` | Restore defaults |
| `pick_output_folder` | Open a native folder picker |
| `install_shell_integration` | Register context-menu integration |
| `uninstall_shell_integration` | Remove context-menu integration |
| `shell_integration_status` | Report whether shell integration is installed and supported |
| `cancel_current_job` | Cancel the active queue batch |
| `supported_targets_for_extension` | Return target formats for a source extension |
| `open_settings_window` | Bring the settings window to the foreground |
| `open_external_url` | Open allow-listed external URLs |

Commands should remain small adapters over engine or shell functionality.

## Progress events

The Rust backend emits `xolariq:progress` events to the frontend.

Events represent queue and job state rather than UI-specific instructions. This lets the UI render state without owning conversion logic.

The application also caches the most recent progress event so a reopened progress window can hydrate immediately instead of waiting for the next update.

## Notifications

Notifications consume the same progress stream as the UI.

Only terminal events produce toasts:

- Job finished.
- Job failed.
- Multi-file queue finished.

This ensures that long conversions do not flood the Windows Action Center.

## URL allow-listing

The `open_external_url` command only accepts specific HTTPS prefixes. This keeps the frontend from becoming a generic shell launcher and reduces accidental abuse of the opener plugin.

## Security posture

The Tauri configuration uses a restrictive content security policy for a local static frontend.

The app should avoid loading remote scripts or dynamic frontend assets. External interaction should happen through explicit backend commands rather than arbitrary frontend shell access.
