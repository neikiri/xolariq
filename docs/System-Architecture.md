# System Architecture

Xolariq is split into clear layers: Windows shell integration, the Tauri desktop process, the conversion engine, and external conversion tools.

The most important design decision is that the core conversion logic is platform-agnostic while Windows-specific behavior is kept in dedicated integration layers.

## High-level flow

A normal conversion follows this path:

1. The user selects a file in Windows Explorer.
2. The context-menu integration launches Xolariq with an input path and target format.
3. The Tauri process parses CLI arguments.
4. Jobs are submitted to the conversion queue.
5. The queue resolves output paths and dispatches the right converter.
6. The converter runs FFmpeg, pandoc, or 7-Zip as a child process.
7. Progress events are emitted to the frontend and notifications layer.
8. The converted file is written to the resolved output path.

## Shell integration layer

The Windows shell layer has two implementations:

- **COM `IExplorerCommand` handler** for modern Windows context menus.
- **Registry-based fallback** for classic context menus and development environments.

The COM path is preferred when the shell extension DLL is available next to the application. The registry fallback remains important for Windows 10, portable developer builds, and environments where COM registration is not available.

## Tauri application layer

The Tauri process is responsible for:

- CLI argument parsing.
- Startup decision between settings and progress windows.
- Single-instance behavior.
- Exposing command handlers to the frontend.
- Forwarding queue progress events to the UI.
- Emitting native notifications for terminal events.

The frontend is intentionally static and does not require a JavaScript bundler.

## Core engine layer

The core engine owns:

- Format definitions.
- Extension-based format detection.
- Output path resolution.
- Persistent settings model.
- Sequential queue processing.
- Cancellation state.
- Converter dispatch.
- External tool resolution.

This layer is designed so it could be reused by a future CLI, service, or alternative frontend.

## External tool layer

Xolariq delegates actual conversion work to mature command-line tools:

| Tool | Responsibility |
| --- | --- |
| FFmpeg | Audio, video, and image conversion |
| pandoc | Document conversion |
| 7-Zip | Archive extraction and archive creation |

Tool resolution supports settings overrides, an environment variable, bundled sidecars, and `PATH` fallback.

## Single-instance behavior

Xolariq uses a single-instance plugin so repeated Explorer invocations do not create competing application processes.

When a second invocation occurs:

1. The existing process receives the new arguments.
2. Arguments are parsed again.
3. New jobs are submitted into the existing queue.
4. The progress window is focused if available.

This matters for real Explorer usage, where users may right-click multiple files or invoke conversions quickly.

## Key invariants

- **One active conversion job at a time** in the current queue implementation.
- **Settings are snapshotted per job** so mid-conversion changes do not alter active work unexpectedly.
- **Source and target kinds must match** before conversion starts.
- **Tool execution is isolated** behind converter modules.
- **Shell registration is best-effort** and should not prevent the app from starting.
- **The UI consumes events** rather than directly controlling conversion internals.

## Extension philosophy

Xolariq favors explicit extension points over dynamic plugin loading.

This keeps the application predictable, easier to package, and easier to audit. New capabilities are added by extending the format model, dispatcher, shell menus, and specific converter modules as needed.
