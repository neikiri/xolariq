# Conversion Engine

The conversion engine is the platform-agnostic heart of Xolariq. It defines supported formats, validates conversion requests, resolves output paths, manages the queue, and dispatches work to the correct converter.

## Format model

Every known format is represented by a central `Format` enum. Each format has:

- A canonical extension.
- A display label.
- A file kind.
- Optional accepted aliases through extension parsing.

`FormatList` provides helper methods for grouping formats by kind and returning target formats for a given source format.

## Detection

Format detection is extension-based.

This is intentional for the current product shape:

- Right-click workflows should stay fast.
- Content sniffing would require reading files during menu construction or queue preparation.
- Extension-based detection is predictable and easy to explain.

A future version could add content sniffing for ambiguous or unsafe extensions, but that would need careful performance and UX design.

## Dispatch pipeline

The main conversion entry point performs the following checks and actions:

1. Confirm the input path exists.
2. Detect the source format from the input extension.
3. Ensure source and target belong to the same file kind.
4. Resolve the output path.
5. Create the output directory if needed.
6. Dispatch to the converter for the target file kind.
7. Return the produced output path.

This means validation happens before an expensive external tool is started.

## Queue model

The queue is sequential and asynchronous.

A `QueueHandle` accepts submitted jobs and forwards them to a worker task through a channel. Each batch produces queue-level events around per-job events.

Important properties:

- Queue handles are cheap to clone.
- Jobs contain a stable UUID.
- Conversion options are derived from settings when the job is enqueued.
- The worker snapshots settings before running each job.
- Failed jobs do not crash the queue; they emit failure events and the batch continues unless cancellation is requested.

## Cancellation

Cancellation uses shared atomic state.

The active job receives an `AtomicBool` cancel flag. When cancellation is requested:

- The current job is signaled.
- Remaining jobs in the active batch are skipped.
- Converters poll the flag and terminate child processes where supported.

This keeps cancellation cooperative while still allowing the backend to kill long-running external tools.

## Output resolution

Output resolution decides where the converted file should be written.

Inputs include:

- Original input path.
- Target format.
- Optional configured output folder.
- Output conflict mode.

If the user did not configure an output folder, Xolariq writes the output next to the input file. If the target already exists, rename mode appends a numeric suffix while overwrite mode writes to the target directly.

## Converter responsibilities

Each converter module owns one file kind.

| Converter | External tool | Notes |
| --- | --- | --- |
| Audio | FFmpeg | Supports progress when duration can be probed |
| Video | FFmpeg | Uses FFmpeg progress output |
| Image | FFmpeg | Often quick; progress may be limited |
| Document | pandoc | PDF output may require LaTeX |
| Archive | 7-Zip | RAR output is not supported |

The dispatcher stays intentionally declarative. Tool-specific flags and edge cases belong in the relevant converter module.

## Error handling

Errors are surfaced as structured engine errors and converted to user-visible messages by higher layers.

Common error categories include:

- Missing input file.
- Unknown extension.
- Unsupported conversion pair.
- Failed output path resolution.
- External tool spawn failure.
- External tool non-zero exit.
- User cancellation.

The queue reports errors through progress events rather than panicking.

## Tool resolution

The engine resolves external tools in this order:

1. Explicit path configured by the user.
2. `XOLARIQ_TOOLS_DIR` environment variable.
3. Sidecar binary next to the running executable.
4. Bare command name resolved by `PATH`.

This order supports both packaged releases and developer environments.
