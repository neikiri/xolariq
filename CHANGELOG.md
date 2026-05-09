# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0] - 2025-05-09

### Added

- Windows Explorer context-menu integration with COM `IExplorerCommand` handler (Windows 11) and registry fallback (Windows 10).
- Batch-aware conversion queue with progress window and native toast notifications.
- Audio conversion via FFmpeg: mp3, wav, flac, aac, ogg, opus.
- Video conversion via FFmpeg: mp4, mkv, webm, mov, avi, gif.
- Image conversion via FFmpeg: png, jpg, webp, avif, heic, ico.
- Document conversion via pandoc: pdf, docx, epub, txt, html, markdown.
- Archive conversion via 7-Zip: zip, 7z, tar, gz, rar (read-only).
- Settings dialog for configuration.
- Bundled FFmpeg, pandoc and 7-Zip sidecars in the installer (MSI + NSIS).
- Tauri 2 application shell with vanilla-JS frontend.
