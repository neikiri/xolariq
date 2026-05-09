# Windows Shell Integration

Xolariq is designed around Windows Explorer. The shell integration is therefore a core product surface, not an optional shortcut.

## Goals

The shell integration must:

- Show a clear **Convert with Xolariq** entry for supported files.
- Route selected files into `xolariq.exe` with the requested target format.
- Work without administrator rights where possible.
- Support modern Windows 11 context menus when the COM extension is available.
- Provide a fallback path for Windows 10 and development builds.

## Preferred path: COM shell extension

The preferred integration is a COM `IExplorerCommand` handler implemented as a Windows DLL.

This path is important because modern Windows 11 context menus do not always surface classic registry verbs in the primary menu. A COM command handler can appear in the modern menu and provide a better user experience.

The COM extension is responsible for:

- Registering itself under the current user.
- Exposing a root command such as **Convert with Xolariq**.
- Collecting selected shell items.
- Launching the Xolariq executable with the selected paths.

During installation, the shell integration attempts to locate the shell extension DLL next to the application executable. If it exists, registration uses the COM path.

## Fallback path: registry verbs

When the COM DLL is unavailable, Xolariq uses a registry-only fallback.

This fallback writes per-user keys under `HKCU\Software\Classes`, avoiding machine-wide installation and administrator requirements.

The fallback creates cascading verbs grouped by file kind. Each leaf command launches Xolariq with arguments similar to:

```powershell
xolariq.exe --target <format> --input "%1"
```

The fallback remains useful for:

- Windows 10.
- Portable or development builds.
- Environments where the COM DLL is missing.
- Situations where COM registration is not available.

## File-kind filtering

The fallback uses Windows shell filtering to avoid showing irrelevant menu entries for every file.

The registry integration scopes menus by file kind where possible and uses explicit extension lists for archive formats where Windows kind metadata is not sufficient.

This design avoids registering individual verbs under every extension, which is less reliable on recent Windows versions.

## Installation lifecycle

Shell integration can be installed through:

- First launch best-effort auto-enable behavior.
- Settings window action.
- CLI subcommand.
- Release installer behavior.

It can be removed through:

- Settings window action.
- CLI subcommand.
- Uninstaller cleanup.

Uninstall attempts to clean both the preferred COM path and legacy/fallback registry paths.

## CLI operations

The Tauri binary exposes shell management subcommands:

```powershell
xolariq install-shell
xolariq uninstall-shell
```

These commands run without starting the full Tauri UI and are useful for scripts, CI checks, and installer workflows.

## Windows 11 notes

On Windows 11, classic registry verbs may appear only under **Show more options**. This is expected for the fallback path and is the reason the COM extension exists.

If the menu does not appear immediately after registration, common fixes include:

- Restarting Explorer.
- Checking **Show more options**.
- Verifying that the Windows Search service is running.
- Confirming that registry keys exist under the current user.

## Safety model

The integration is per-user and best-effort.

Shell failures should not prevent the main application from starting. If registration fails, the user can still run conversions through direct CLI invocation or by fixing the shell integration from settings.
