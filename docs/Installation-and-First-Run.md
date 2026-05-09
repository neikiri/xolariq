# Installation and First Run

This page explains how to install Xolariq, what the first launch does, and how to verify that the Windows context-menu workflow is ready.

## Requirements

For normal users, the release installer is intended to include everything needed to run conversions locally.

For development or source builds, the machine should have:

- **Windows 10 or Windows 11** with WebView2 available.
- **Rust stable** with `rustfmt` and `clippy`.
- **Microsoft C++ build tools** for the MSVC linker used by Tauri on Windows.
- **Tauri CLI v2** installed through Cargo.
- **FFmpeg, pandoc, and 7-Zip** either installed globally or available as bundled sidecar binaries.

## Installing from a release

1. Download the latest `.msi` or `.exe` installer from the project releases.
2. Run the installer.
3. Launch Xolariq once from the Start menu or by opening the installed executable.
4. Confirm that the settings window opens and that shell integration is enabled.
5. Right-click a supported file and choose **Convert with Xolariq**.

The MSI/NSIS release configuration is designed to ship Xolariq as a Windows utility and bundle external conversion tools next to the application binary.

## First launch behavior

On first launch, Xolariq initializes its settings and attempts to enable the Windows context-menu integration when appropriate.

The application stores settings in the user configuration directory under an Xolariq-specific `settings.json`. If the settings file is missing or unreadable, the application falls back to safe defaults instead of blocking startup.

Default behavior includes:

- **Context menu enabled** by default.
- **Metadata preservation** enabled by default.
- **Output conflict mode** set to rename rather than overwrite.
- **Output folder** unset, meaning converted files are written next to the source file.

## Verifying shell integration

After installation, open File Explorer and right-click a supported file such as `.mp3`, `.png`, `.docx`, or `.zip`.

Expected behavior:

- On modern Windows 11 builds with the COM extension registered, the Xolariq entry should appear in the primary context menu.
- If the COM extension is not available, the registry fallback may appear under **Show more options**.
- Selecting a target format should start `xolariq.exe` with the selected input path and target format.
- A compact progress window should appear while conversion is running.

## Running from source

Install the Tauri CLI:

```powershell
cargo install tauri-cli --version "^2"
```

Run the app in development mode:

```powershell
cargo tauri dev
```

Trigger a conversion manually without using the context menu:

```powershell
cargo run --bin xolariq -- --target mp3 --input "C:\path\to\song.wav"
```

Register the context menu manually:

```powershell
cargo run --bin xolariq -- install-shell
```

Remove it again:

```powershell
cargo run --bin xolariq -- uninstall-shell
```

## External tools

Xolariq resolves tools in this priority order:

1. Explicit user override from settings.
2. `XOLARIQ_TOOLS_DIR` environment variable.
3. Tool sidecar next to the running executable.
4. Normal `PATH` lookup.

This allows the same code to work for bundled releases, portable development setups, and CI environments.

## Common first-run issues

| Symptom | Likely cause | Fix |
| --- | --- | --- |
| Context menu is missing | Explorer has not refreshed or only the fallback is available | Restart Explorer or check **Show more options** |
| FFmpeg cannot be spawned | FFmpeg is not bundled and not on `PATH` | Install FFmpeg or set the tool path in settings |
| PDF output fails | pandoc can call LaTeX, but LaTeX is missing | Install MiKTeX or choose a non-PDF target |
| 7-Zip conversions fail | `7z.exe` or `p7za.exe` cannot be resolved | Install 7-Zip or set `XOLARIQ_TOOLS_DIR` |
