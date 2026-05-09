# Xolariq Wiki

Welcome to the **Xolariq** wiki — the technical and user-facing knowledge base for a Windows-first file converter that lives directly inside the Explorer context menu.

Xolariq is designed as a shell-first desktop app. Users do not need to open a large main window, upload files to a cloud service, or build a complex workflow. The primary flow is simple: right-click a supported file, choose a target format, and monitor the conversion in a compact progress window.

## What Xolariq does

- **Local file conversion** without web services or accounts.
- **Windows Explorer integration** through a modern COM shell extension with a registry-based fallback.
- **Batch-aware processing** with a sequential queue and a single progress stream.
- **Unified conversion engine** for audio, video, images, documents, and archives.
- **Bundled tool support** so releases can ship with FFmpeg, pandoc, and 7-Zip sidecars.

## Who this wiki is for

- **Users** can learn how to install Xolariq, use the context menu, configure settings, and troubleshoot common problems.
- **Developers** can understand the architecture, conversion pipeline, Tauri command bridge, shell integration, and release workflow.
- **Maintainers** can find the key invariants and extension points for adding formats, tools, or platform integrations.

## Recommended reading path

- **New users** should start with **Installation and First Run**.
- **Power users** should continue with **User Guide** and **Supported Formats**.
- **Developers** should read **System Architecture**, **Conversion Engine**, and **Developer Guide**.
- **Windows integration work** should start with **Windows Shell Integration**.

## Project principles

- **Local-first** — user files stay on the machine.
- **Small core, explicit adapters** — conversion logic belongs in `xolariq-core`; platform-specific integration lives outside it.
- **Per-user installation** — shell registration is designed to avoid administrator requirements where possible.
- **Predictable output** — converted files are written next to the input or into a user-selected output folder.
- **Minimal frontend stack** — the UI is static HTML, CSS, and JavaScript hosted by Tauri.

## Current status

Xolariq is currently version **0.1.0** and primarily targets Windows. It uses Rust, Tauri 2, FFmpeg, pandoc, and 7-Zip to provide local file conversion from the Windows context menu.
