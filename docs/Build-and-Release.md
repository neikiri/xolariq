# Build and Release

This page describes how to build Xolariq locally and prepare release bundles.

## Build prerequisites

A Windows build machine should have:

- Rust stable.
- `rustfmt` and `clippy` components.
- Microsoft C++ Build Tools or Visual Studio with Desktop C++ workload.
- WebView2 runtime.
- Tauri CLI v2.
- FFmpeg, pandoc, and 7-Zip for runtime conversion testing.

Install the Tauri CLI:

```powershell
cargo install tauri-cli --version "^2"
```

## Development build

Run the desktop app in development mode:

```powershell
cargo tauri dev
```

The first run may take longer because Rust dependencies and Tauri components need to compile.

To test conversion without installing shell integration, run the binary directly with conversion arguments:

```powershell
cargo run --bin xolariq -- --target mp3 --input "C:\path\to\song.wav"
```

## Tool sidecars

Release builds are designed to bundle external tools through Tauri sidecars.

Required tools:

| Tool | Purpose |
| --- | --- |
| FFmpeg | Audio, video, and image conversions |
| pandoc | Document conversions |
| 7-Zip / p7za | Archive conversions |

For local development, tools can also be installed globally and resolved through `PATH`.

For release bundles, sidecar binaries should be placed where Tauri expects external binaries and named according to Tauri's target-triple sidecar convention.

## Icons

Tauri requires application icons for packaging. Icons can be generated from a source image with:

```powershell
cargo tauri icon path\to\source.png
```

Generated icons are used by the app window, installer, and Windows shell integration.

## Release build

Build release bundles with:

```powershell
cargo tauri build
```

The configured bundle targets are:

- **MSI** installer.
- **NSIS** installer.

The bundle metadata identifies Xolariq as a Windows utility for local file conversion.

## Shell extension packaging

When the COM shell extension DLL is included next to the executable, the installer can register it so Xolariq appears in the modern Windows context menu.

If the DLL is missing, Xolariq can still use the registry fallback path. This is common for development builds.

## Quality checks

Recommended checks before release:

```powershell
cargo test -p xolariq-core
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

The core engine owns most unit-testable behavior. The Tauri application is mainly glue code over the core and shell crates.

## Release checklist

Before publishing a release:

- Confirm version numbers are consistent.
- Confirm bundled external tools are present and executable.
- Include license files for bundled tools.
- Verify MSI and NSIS installers build successfully.
- Test first launch on a clean Windows user profile.
- Test context-menu registration and removal.
- Test representative audio, video, image, document, and archive conversions.
- Verify that uninstall removes shell integration.

## Troubleshooting release builds

| Symptom | Likely cause | Fix |
| --- | --- | --- |
| Tauri build cannot find sidecars | External binaries are missing or incorrectly named | Rename/copy them according to the target triple convention |
| MSI signing fails | Local signing configuration is not available | Disable signing for local builds or configure a valid signing identity |
| FFmpeg works in dev but not installer | Tool is on developer `PATH` but not bundled | Add the sidecar to the release bundle |
| Context menu works in dev but not release | COM DLL or registry registration did not run | Verify shell extension packaging and installer custom actions |
