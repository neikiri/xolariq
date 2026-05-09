<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="./assets/logo-dark.png">
    <source media="(prefers-color-scheme: light)" srcset="./assets/logo-light.png">
    <img alt="logo" src="./assets/logo-light.png" width="620">
  </picture>
</p>

<h1 align="center">Right-click file conversion for Windows.</h1>

<p align="center">
  Convert media, documents and archives directly from Explorer.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/tauri-%2324C8D8.svg?style=for-the-badge&logo=tauri&logoColor=white" alt="Tauri">
  <img src="https://img.shields.io/badge/javascript-%23323330.svg?style=for-the-badge&logo=javascript&logoColor=%23F7DF1E" alt="JavaScript">
  <img src="https://img.shields.io/badge/windows-%230078D6.svg?style=for-the-badge&logo=windows&logoColor=white" alt="Windows">
  <br>
  <img src="https://img.shields.io/badge/License-MPL%202.0-2563EB?style=for-the-badge&logo=open-source-initiative&logoColor=white&labelColor=000F15&logoWidth=20" alt="License">
  <img src="https://img.shields.io/badge/Version-0.1.0-2563EB?style=for-the-badge&logo=semantic-release&logoColor=white&labelColor=000F15&logoWidth=20" alt="Version">
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Formats-30%2B%20Supported-3b82f6?style=flat&labelColor=383C43" />
  <img src="https://img.shields.io/badge/UI-Context%20Menu-8b5cf6?style=flat&labelColor=383C43" />
  <img src="https://img.shields.io/badge/Cloud-None-22c55e?style=flat&labelColor=383C43" />
  <img src="https://img.shields.io/badge/Installer-MSI%20%2B%20EXE-f97316?style=flat&labelColor=383C43" />
</p>

---

## 📖 About

Xolariq is a Windows desktop tool that lives entirely inside the Explorer **context menu**. Right-click any audio, video, image, document or archive file, pick a target format, and Xolariq runs the conversion in the background — no main window, no account, no cloud upload. It's a thin, fast shell over a small set of well-trusted local tools: **FFmpeg** for audio/video/images, **pandoc** for documents, and **7-Zip** for archives.

The installer bundles everything needed — no extra downloads, no `PATH` configuration, no dependencies to manage. Install once, convert forever.

---

## ✨ Highlights

- **Shell-first UX** — the primary surface is the Windows context menu. The settings window is just a small dialog you rarely have to open.
- **Batch-aware queue** — right-click multiple files, pick a target — they process sequentially with a single progress window and one finish toast.
- **Two-tier shell integration** — a native COM `IExplorerCommand` handler (`xolariq-shellext`) powers the entry in Windows 11's default context menu; a pure-registry fallback covers Windows 10 and per-user installs.
- **Pluggable converters** — audio/video/image route through FFmpeg, documents through pandoc, archives through 7-Zip. Adding a new format is a one-file change.
- **Tools bundled with the installer** — the MSI ships FFmpeg, pandoc and 7-Zip next to `xolariq.exe`, so users don't need to install anything extra.
- **Fully local** — no AI, no cloud, no plugin marketplace, no premium tier.

---

## 📂 Supported formats

| Kind     | Formats                                              |
| -------- | ---------------------------------------------------- |
| Audio    | mp3, wav, flac, aac, ogg, opus                       |
| Video    | mp4, mkv, webm, mov, avi, gif                        |
| Image    | png, jpg, webp, avif, heic, ico                      |
| Document | pdf*, docx, epub, txt, html, markdown                |
| Archive  | zip, 7z, tar, gz, rar†                               |

\*PDF output requires a working LaTeX install (e.g. MiKTeX); PDF input is not supported in the free version.
†RAR is read-only — Xolariq can convert *from* `.rar` but not *to* `.rar` (proprietary `Rar.exe` required).

---

## 🚀 Quick start

1. Head to [**Releases**](https://github.com/neikiri/xolariq-dev/releases) and download the latest installer — `.msi` (recommended) or `.exe`.
2. Run the installer. Done — no extra setup needed.
3. Right-click any supported file in Explorer → **"Convert with Xolariq"** → pick a target format.

## 📚 Documentation

For a deeper user, administrator and developer guide, see [`WIKI.md`](./WIKI.md).

### Building from source

If you prefer to build locally, you will need a Windows machine, Rust and the Tauri CLI. See [`BUILD.md`](./BUILD.md) for the full step-by-step guide.

```powershell
cargo install tauri-cli --version "^2"
# download sidecar binaries into app/src-tauri/external/ (see BUILD.md)
cargo tauri build
```

---

## 📄 License

This project is licensed under the **Mozilla Public License 2.0** — see the [LICENSE](LICENSE) file for details.

The bundled conversion tools each ship under their own license — see [`app/src-tauri/external/LICENSES/`](./app/src-tauri/external/LICENSES/) (FFmpeg LGPL/GPL, pandoc GPLv2+, 7-Zip LGPL/unRAR-restricted).

---

## 👨‍💻 Author

**neikiri**
GitHub: https://github.com/neikiri

---

## 📬 Contact

📧 Email: dev@neiki.eu
