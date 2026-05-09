# User Guide

Xolariq is built around a right-click workflow. The application is intentionally small: the context menu starts conversions, the progress window reports status, and the settings window controls persistent preferences.

## Converting a file

1. Open File Explorer.
2. Right-click a supported file.
3. Select **Convert with Xolariq**.
4. Choose one of the available target formats.
5. Wait for the progress window to finish.

Xolariq only offers target formats that belong to the same file kind as the input. For example, an audio file can be converted to another audio format, but not directly to a document format.

## Converting multiple files

When multiple files are submitted, Xolariq creates a batch and processes it sequentially.

The queue emits one batch-level start event, one batch-level finish event, and per-job events for each file. This keeps the UI predictable and avoids multiple competing conversion windows.

Important behavior:

- Jobs run one after another, not in parallel.
- Each job receives a snapshot of settings at the moment it is enqueued.
- Cancelling stops the active job and skips the remaining jobs in the current batch.
- New batches submitted later are unaffected by a previous cancellation.

## Progress window

The progress window appears when a conversion is active. It receives live events from the Rust backend through Tauri events.

Typical states include:

- **Queue started** — a batch has been accepted.
- **Job started** — a specific file is being converted.
- **Job progress** — progress is known when the underlying tool can report it.
- **Job finished** — output file was produced successfully.
- **Job failed** — conversion failed with an error message.
- **Queue finished** — the batch completed with success/failure totals.

Some conversions cannot expose exact progress. In those cases, Xolariq may show an indeterminate state rather than a precise percentage.

## Notifications

Xolariq uses Windows notifications only for terminal events.

Notifications are intentionally limited:

- A successful single conversion can show a completion toast.
- A failed conversion can show an error toast.
- A multi-file batch can show a summary toast.
- Per-progress events do not create notifications.

This prevents notification spam during long or multi-file conversions.

## Settings window

The settings window controls persistent behavior:

| Setting | Meaning |
| --- | --- |
| Output folder | Optional folder where converted files should be written |
| Output mode | Choose whether existing output files are overwritten or renamed |
| Preserve metadata | Keep tags, EXIF, or container metadata when supported |
| Context menu integration | Install or remove Explorer integration |
| Tool paths | Optional overrides for FFmpeg, pandoc, or 7-Zip |

If no output folder is configured, converted files are written next to the original input file.

## Output naming

Xolariq resolves output paths before the conversion starts.

When the target file already exists:

- **Rename mode** appends a numeric suffix such as `(1)` or `(2)`.
- **Overwrite mode** writes to the resolved target path directly.

Rename mode is the default because it is safer for non-technical users.

## Privacy model

Xolariq is local-first:

- Files are processed on the local machine.
- No account is required.
- No cloud upload is part of the conversion pipeline.
- External tools run as local child processes.

The main privacy consideration is therefore the trustworthiness of the bundled or configured external conversion tools.
