# Building Xolariq on Windows

This guide walks through every step required to build, run and package
Xolariq from a fresh Windows 10 / Windows 11 machine. The instructions
are intentionally conservative — they prefer official installers and
avoid steps that need administrator rights wherever possible.

## 1. Install prerequisites

### 1.1 Rust

Download `rustup` from <https://rustup.rs> and run the default install.
Restart your shell so the freshly added `cargo` is on `PATH`, then:

```powershell
rustup default stable
rustup component add rustfmt clippy
```

### 1.2 Microsoft C++ build tools

Tauri's Windows backend (WebView2) needs the MSVC linker. Install the
*Build Tools for Visual Studio* (or full Visual Studio with the
"Desktop development with C++" workload).

Download: <https://visualstudio.microsoft.com/downloads/>

### 1.3 WebView2 runtime

Already present on Windows 11 and most Windows 10 systems. If missing,
fetch the *Evergreen Standalone Installer* from
<https://developer.microsoft.com/microsoft-edge/webview2/>.

### 1.4 Tauri CLI

```powershell
cargo install tauri-cli --version "^2"
```

### 1.5 Runtime conversion tools

There are two ways to satisfy the FFmpeg / pandoc / 7-Zip dependency at
build time. Pick whichever fits how you plan to *run* Xolariq.

#### Option A — bundle them with the installer (recommended for releases)

The default Tauri config (`bundle.externalBin` in
`tauri.conf.json`) ships sidecar binaries next to `xolariq.exe`. Drop
the following files into `app/src-tauri/external/` before running
`cargo tauri build`:

| Sidecar name | Source                                                                         |
| ------------ | ------------------------------------------------------------------------------ |
| `ffmpeg.exe` | <https://www.gyan.dev/ffmpeg/builds/> (essentials build is sufficient)         |
| `pandoc.exe` | <https://github.com/jgm/pandoc/releases> (Windows ZIP, take only `pandoc.exe`) |
| `7za.exe`    | <https://www.7-zip.org/download.html> ("7-Zip Extra" → `7za.exe`)              |

Tauri 2's `externalBin` mechanism appends a target triple to each path
at build time, so the actual files Tauri looks for are
`ffmpeg-x86_64-pc-windows-msvc.exe`, `pandoc-x86_64-pc-windows-msvc.exe`
and `7za-x86_64-pc-windows-msvc.exe`. Either rename the files
accordingly *or* set up a small PowerShell helper to copy them with the
suffix:

```powershell
$tools = "ffmpeg","pandoc","7za"
foreach ($t in $tools) {
  Copy-Item "external\$t.exe" "external\$t-x86_64-pc-windows-msvc.exe"
}
```

Also drop each tool's upstream `LICENSE` file into
`app/src-tauri/external/LICENSES/` (the bundle's `resources` glob
ships them inside the installer; see
[`LICENSES/README.md`](./app/src-tauri/external/LICENSES/README.md) for
exact filenames). The repo deliberately does **not** commit the
binaries or license texts — they are large, lightly versioned, and the
upstream sources are the canonical home.

The backend resolves tools through the priority order:
**explicit Settings override → `XOLARIQ_TOOLS_DIR` env var → sidecar
next to `xolariq.exe` → bare command on `PATH`** (see
`crates/xolariq-core/src/tools.rs`). This means the bundle in (A) just
works, and the dev shell in (B) below also works without any config.

#### Option B — install the tools globally (recommended for development)

Pick whichever package manager you already use; the names below assume
`winget`. Substitute `scoop` / `choco` freely.

| Tool   | Purpose                              | Command                                              |
| ------ | ------------------------------------ | ---------------------------------------------------- |
| FFmpeg | Audio / video / image conversions    | `winget install Gyan.FFmpeg`                         |
| pandoc | Document conversions                 | `winget install JohnMacFarlane.Pandoc`               |
| 7-Zip  | Archive conversions                  | `winget install 7zip.7zip`                           |
| LaTeX  | *Optional.* Required for PDF output. | `winget install MiKTeX.MiKTeX`                       |

After installation, confirm everything is on `PATH`:

```powershell
ffmpeg -version
pandoc --version
7z
```

If `7z` is not on `PATH` after installing 7-Zip, add
`C:\Program Files\7-Zip` to your user `PATH` or set the override in
Xolariq's Settings window once it is built. You can also point Xolariq
at a directory containing all three tools via the `XOLARIQ_TOOLS_DIR`
environment variable — handy for CI / portable installs.

## 2. Clone the repository

```powershell
git clone https://github.com/neikiri/xolariq-dev.git
cd xolariq-dev
```

## 3. Provide application icons

The Tauri bundler needs at least `icons/icon.ico`, `icons/32x32.png` and
`icons/128x128.png` under `app/src-tauri/icons/`. The fastest way is to
generate them all from a single source PNG using Tauri's helper:

```powershell
cargo tauri icon path\to\source.png
```

The generated files land in `app/src-tauri/icons/`.

## 4. Run in development

```powershell
cargo tauri dev
```

The first run builds the workspace from scratch and launches the
Settings window. To trigger a conversion without registering the
context menu, run from another terminal:

```powershell
cargo run --bin xolariq -- --target mp3 --input "C:\path\to\song.wav"
```

A separate progress window opens, the queue runs, and you'll get a
Windows toast when the file is ready.

## 5. Register the Windows context menu

The first time you click **Enable** in the Settings window, Xolariq
writes per-user registry keys under `HKCU\Software\Classes`. No admin
rights required. You can do the same thing from the command line:

```powershell
cargo run --bin xolariq -- install-shell
```

To remove the integration:

```powershell
cargo run --bin xolariq -- uninstall-shell
```

The companion binaries `xolariq-shell-install` and
`xolariq-shell-uninstall` (built from the `xolariq-shell` crate) are
equivalent and safe to script from CI.

## 6. Build a release bundle

```powershell
cargo tauri build
```

This produces:

- `app/src-tauri/target/release/xolariq.exe` — the standalone binary.
- `app/src-tauri/target/release/bundle/msi/Xolariq_*.msi` — Windows MSI.
- `app/src-tauri/target/release/bundle/nsis/Xolariq_*-setup.exe` — NSIS installer.

The MSI / NSIS installers register the context menu on first launch.

## 7. Run the test suite

```powershell
cargo test -p xolariq-core
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

The Tauri app itself does not ship Rust unit tests — its logic is
glue code over `xolariq-core` and `xolariq-shell`, which both have
their own tests.

## 8. Troubleshooting

| Symptom                                            | Likely cause / fix                                                                        |
| -------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| `failed to spawn ffmpeg`                           | FFmpeg not on `PATH`. Re-open the shell after install, or set the path in Settings.       |
| `pandoc` errors when producing PDF                 | LaTeX is missing. Install MiKTeX (`winget install MiKTeX.MiKTeX`) or pick a non-PDF target.|
| Context-menu entry doesn't appear after enabling    | **Windows 11:** classic registry verbs (incl. ours) only appear under **"Show more options"** or `Shift + Right-click`. After (re)enabling, restart Explorer with `Stop-Process -Name explorer -Force; Start-Process explorer`. Verify keys exist at `HKCU\Software\Classes\*\shell\Xolariq.Audio` (and `Xolariq.Video`, `Xolariq.Image`, `Xolariq.Document`, `Xolariq.Archive`) in `regedit`. The `AppliesTo` filter on each verb relies on the **Windows Search service** — make sure it is running (`Get-Service WSearch`). |
| HEIC / AVIF conversions fail                        | The Windows FFmpeg build from gyan.dev includes `libheif` and `libaom`; confirm `ffmpeg -encoders` lists them. |
| MSI fails to sign during `cargo tauri build`        | Disable signing in `tauri.conf.json` (`bundle.windows.wix`) for unsigned local builds.    |

## 9. Uninstall

1. Open the Settings window and click **Disable** under "Enable Windows context menu".
2. Run the platform uninstaller (Windows → Apps → Xolariq) or simply delete the install folder.
3. Per-user settings live under `%APPDATA%\Xolariq\Xolariq\settings.json` and can be removed manually.
