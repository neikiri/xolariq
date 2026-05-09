# Architecture

Xolariq is split into four layers that map cleanly to crates:

```
+------------------------------------------------------------------+
|  Windows Explorer (right-click)                                  |
|     |                                                            |
|     +-- preferred ---> COM IExplorerCommand handler              |
|     |                  (xolariq-shellext, dll)                   |
|     +-- fallback --->  classic verb under                        |
|                        HKCU\Software\Classes\*\shell\Xolariq.*   |
|                        (xolariq-shell, registry-only)            |
|     |                                                            |
|     v                                                            |
|  xolariq.exe --target <ext> --input "<path>"                     |
+--------------------------------|---------------------------------+
                                 |
                                 v
+------------------------------------------------------------------+
|  Tauri app  (crate: xolariq-app)                                 |
|    + CLI parser  (cli.rs)                                        |
|    + Single-instance bridge -> existing queue                    |
|    + Tauri commands  (commands.rs)                               |
|    + Native toasts   (notify.rs)                                 |
|    + Vanilla-JS UIs  (progress.html, settings.html)              |
+--------------------------------|---------------------------------+
                                 |
                                 v
+------------------------------------------------------------------+
|  Conversion engine  (crate: xolariq-core)                        |
|    detect -> resolve_output_path -> queue ->                     |
|        convert::{audio, video, image, document, archive}         |
|    tools::resolve_tool dispatches each binary lookup             |
|    queue is sequential, cancellable, sink-based                  |
+--------------------------------|---------------------------------+
                                 |
                                 v
                +-----------------------------------+
                | external tools (sidecar or PATH)  |
                |   ffmpeg, pandoc, 7za             |
                +-----------------------------------+
```

## Layer 1 — Shell integration

Two crates cover this layer; both are Windows-only.

### Layer 1a — COM `IExplorerCommand` handler (`crates/xolariq-shellext`)

A `cdylib` (`xolariq_shellext.dll`) that exposes a single root
`IExplorerCommand` — *"Convert with Xolariq"* — through the official
COM in-process server contract:

- `DllMain` captures the DLL `HMODULE` so registration can resolve its
  own filesystem path.
- `DllGetClassObject` returns an `IClassFactory` that builds
  [`RootCommand`](crates/xolariq-shellext/src/root.rs).
- `DllRegisterServer` / `DllUnregisterServer` write under
  `HKCU\Software\Classes\CLSID\{a4f1d8e2-…}` and a wildcard verb under
  `HKCU\Software\Classes\*\shell\XolariqRoot`. Per-user — no admin.
- `RootCommand::Invoke` collects every selected `IShellItem` path and
  spawns `xolariq.exe` (looked up next to the DLL) with those paths as
  arguments; the running Tauri app then chooses the target format.

This handler is what surfaces the entry in Windows 11's modern context
menu *without* the *"Show more options"* detour. The crate is
intentionally light on dependencies — it links against
[`windows`](https://crates.io/crates/windows) and the standard library
only, so the DLL is small and loads quickly inside every Explorer
process.

#### Registration paths

The Wix bundle (`app/src-tauri/wix/shellext.wxs`) drops the DLL next to
`xolariq.exe` and runs `regsvr32 /s` as a Windows Installer custom
action on install/uninstall. Outside of the bundle, the
`xolariq install-shell` / `uninstall-shell` CLI commands also drive
`regsvr32` against the DLL when it is colocated with the running exe;
if it is not present (developer builds, portable runs) the registry
fallback alone is registered.

### Layer 1b — Pure-registry fallback (`crates/xolariq-shell`)

A tiny Windows-only crate that owns every classic verb Xolariq writes.
It exposes a [`ShellIntegration`](crates/xolariq-shell/src/lib.rs)
trait with `install` / `uninstall` / `is_installed` and a Windows
implementation backed by [`winreg`](https://docs.rs/winreg).

For each [`FileKind`] we register a single cascading verb under the
**wildcard `*` class** filtered by an `AppliesTo` Search Conditions
query:

- `HKCU\Software\Classes\*\shell\Xolariq.<Kind>` with empty
  `SubCommands` (the documented opt-in for the nested-shell cascade
  pattern), `MUIVerb`, `Icon`, `AppliesTo`.
- `HKCU\Software\Classes\*\shell\Xolariq.<Kind>\shell\NN_To<Fmt>` with
  the per-format leaf verb and a `command` subkey holding the actual
  `xolariq.exe --target <fmt> --input "%1"` invocation.

Wildcard classes are required because Windows 11 25H2 silently ignores
verbs registered at `HKCU\Software\Classes\.<ext>\shell\<verb>` even
though `HKEY_CLASSES_ROOT` shows them correctly. The `AppliesTo` filter
is what keeps the verb scoped to relevant kinds — `System.Kind:="…"`
for music/video/picture/document and an explicit `System.FileExtension`
list for archives (which have no `System.Kind` value). See the module
docs of `crates/xolariq-shell/src/windows.rs` for the full diagnostic
trail.

Classic verbs only show under *"Show more options"* on Win11 25H2,
which is why Layer 1a exists — but they are a fully functional
fallback for Windows 10, sandboxed user environments, and developers
running `cargo tauri dev` without a registered COM DLL.

The crate also ships standalone `xolariq-shell-install` /
`xolariq-shell-uninstall` binaries for CI and headless environments.

## Layer 2 — Tauri app (`app/src-tauri`)

The Tauri 2 binary is the user-facing process. It plays three roles:

1. **Conversion launcher.** Parses CLI arguments
   (`--target`, `--input`, subcommands) and submits jobs to the queue.
2. **UI host.** Two pre-declared windows live in `tauri.conf.json`:
   `progress` (modal popup that streams queue events from
   `xolariq:progress`) and `settings` (small dialog to toggle the
   integration and pick the output folder).
3. **Single-instance arbiter.** Backed by
   `tauri-plugin-single-instance`. If the user right-clicks several
   files in quick succession, the second invocation forwards its argv
   to the first instance, which in turn drops the new jobs into the
   already-running queue.

The frontend is intentionally vanilla JS. There is no bundler, no
package.json — just three static files (`progress.html`, `settings.html`,
`styles.css`) loaded directly from `app/src/`.

### Commands

| Command                            | Purpose                                            |
| ---------------------------------- | -------------------------------------------------- |
| `get_settings`                     | Read current persisted settings.                   |
| `update_settings` / `reset_settings` | Persist updated settings or fall back to defaults. |
| `pick_output_folder`               | Native folder picker.                              |
| `install_shell_integration`        | Register context-menu entries.                     |
| `uninstall_shell_integration`      | Remove context-menu entries.                       |
| `shell_integration_status`         | Probe whether registry keys exist.                 |
| `cancel_current_job`               | Stop the active job.                               |
| `supported_targets_for_extension`  | Return the dynamic submenu for a file extension.   |
| `open_settings_window`             | Show the settings window from the progress window. |

### Notifications

`notify::handle_event` consumes the same `ProgressEvent` stream the UI
sees and emits a Windows toast on terminal events only:

- `JobFinished` → "Conversion complete"
- `JobFailed`   → "Conversion failed" (with the error message)
- `QueueFinished` for batches of 2+ → summary of successes/failures.

Per-job progress events deliberately produce no toasts to avoid spamming
the action centre.

## Layer 3 — Conversion engine (`crates/xolariq-core`)

The only platform-agnostic crate. Its public surface is small enough to
embed in a CLI, a service, or any future Xolariq surface.

### Format & detection

[`format::Format`](crates/xolariq-core/src/format.rs) lists every
supported format and groups them by [`FileKind`](crates/xolariq-core/src/kind.rs).
[`detect::detect_format`](crates/xolariq-core/src/detect.rs) is
extension-based for now — content sniffing is reserved for a future
release because it would require a synchronous read on every right-click.

### Settings

[`settings::Settings`](crates/xolariq-core/src/settings.rs) lives in
`<config_dir>/Xolariq/settings.json` and is loaded once into a
[`SettingsStore`](crates/xolariq-core/src/settings.rs). The store is
held in Tauri's managed state so every queue worker reads the same
snapshot without re-parsing the file.

### Output resolution

[`output::resolve_output_path`](crates/xolariq-core/src/output.rs)
applies the user's overwrite/rename mode and returns an absolute path.
It is fully unit-tested.

### Queue

[`queue::QueueHandle`](crates/xolariq-core/src/queue.rs) spawns one
tokio worker that drains an unbounded channel. Each batch submitted via
`QueueHandle::submit` produces a single `QueueStarted` /
`QueueFinished` pair surrounding any number of `JobStarted` /
`JobProgress` / `JobFinished` events.

Cancellation works via a shared `AtomicBool`: `cancel_current` flips
the flag, the per-format converters poll it between progress callbacks
and kill the underlying child process when set.

### Converters

`crates/xolariq-core/src/convert/{audio,video,image,document,archive}.rs`
each implement the same async signature:

```rust
pub async fn convert(
    input: &Path,
    output: &Path,
    source: Format,
    target: Format,
    options: &ConvertOptions,
    settings: &Settings,
    cancel: Arc<AtomicBool>,
    on_progress: impl Fn(Option<f32>) + Send + Sync + Clone + 'static,
) -> Result<()>;
```

[`convert::convert_one`](crates/xolariq-core/src/convert/mod.rs)
dispatches by `target.kind()` so adding a new file kind (e.g. fonts,
3D models) means adding one module + one match arm.

### FFmpeg wrapper

[`ffmpeg::run_ffmpeg`](crates/xolariq-core/src/ffmpeg.rs) spawns
`ffmpeg -progress pipe:1 …`, parses `out_time_us` for percent progress,
honours the cancel flag, and surfaces non-zero exits as
`Error::ToolFailed { tool: "ffmpeg", … }` with the captured stderr
attached. Duration is probed once via a separate non-encoding ffmpeg
call so audio/video jobs can show meaningful percentages.

## Extensibility

Every layer is built around a single clear extension point:

| Layer        | "Add a new …" extension point                                      |
| ------------ | ------------------------------------------------------------------ |
| Engine       | New `Format` variant + new converter module                        |
| Engine       | New `FileKind` (e.g. fonts) + new dispatcher arm                   |
| Tauri app    | New command in `commands.rs` + matching `invoke()` in JS           |
| Shell crate  | New `ShellIntegration` impl (e.g. for macOS Services / Linux DEs)  |

Future, deliberately-out-of-scope features (AI helpers, cloud sync,
plugin marketplace, GPU encoding) all fit cleanly into one of these
extension points without disturbing the others.
