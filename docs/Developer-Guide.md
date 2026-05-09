# Developer Guide

This guide explains how to work on Xolariq safely and where to make common changes.

## Development philosophy

Xolariq favors explicit, maintainable code over dynamic plugin systems.

Important principles:

- Keep conversion logic in the core engine.
- Keep Tauri commands thin.
- Keep shell behavior isolated from conversion behavior.
- Prefer deterministic settings snapshots for queued jobs.
- Avoid frontend complexity unless the UI genuinely needs it.
- Treat external tools as replaceable process adapters.

## Adding a new format

For a format within an existing file kind:

1. Add the new format to the central format enum.
2. Add its canonical extension.
3. Add aliases if needed.
4. Assign the correct file kind.
5. Add a display label.
6. Update the relevant converter module.
7. Verify the context menu offers the target only for compatible source formats.
8. Add or update tests for extension parsing and target filtering.

If the external tool already supports the format, the converter change may only involve command-line arguments.

## Adding a new file kind

A new file kind is a larger change.

You should expect to update:

- File kind definitions.
- Format grouping.
- Converter dispatch.
- A new converter module.
- Shell menu generation.
- UI target listing if assumptions change.
- Documentation and tests.

Examples of possible future kinds include fonts, e-books as a separate category, or 3D model formats.

## Adding a new external tool

A new tool should be added through the existing resolver pattern.

Recommended steps:

1. Add a logical tool variant.
2. Define the sidecar/PATH stem.
3. Add settings override fields if users need custom paths.
4. Update release bundle configuration.
5. Update installer payloads and license documentation.
6. Keep invocation details inside the converter that needs the tool.

Do not scatter direct `Command::new("tool")` calls across unrelated modules.

## Tauri command guidelines

Tauri commands should be small adapters.

A good command:

- Accepts frontend-friendly data.
- Calls a core or shell API.
- Converts errors into user-visible strings.
- Does not duplicate conversion logic.
- Does not perform broad filesystem or process operations unless that is its explicit purpose.

If a command grows complex, move the logic into the appropriate Rust crate first.

## Queue and cancellation guidelines

The queue currently runs sequentially by design.

Before changing queue behavior, consider:

- External tools may be CPU and I/O heavy.
- Parallel conversions could overload low-end machines.
- Progress UI assumes a coherent active job state.
- Cancellation currently targets the active job and remaining batch.
- Settings snapshots avoid mid-batch surprises.

Parallel execution would require a deliberate redesign of progress aggregation, cancellation, and resource limits.

## Shell integration guidelines

Shell changes should be tested on real Windows versions.

Pay special attention to:

- Windows 11 modern context menu behavior.
- Classic **Show more options** behavior.
- Per-user registry paths.
- Uninstall cleanup.
- COM registration and unregistration.
- Explorer refresh behavior.

Avoid assuming that registry keys visible in `regedit` automatically appear in the modern context menu.

## Frontend guidelines

The frontend is intentionally simple.

When editing UI code:

- Keep state derived from backend events.
- Avoid adding a bundler unless the project explicitly adopts one.
- Keep Tauri invokes centralized and easy to audit.
- Avoid arbitrary external URL opening.
- Preserve the compact utility-app feel.

## Testing strategy

Recommended local checks:

```powershell
cargo test -p xolariq-core
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

Manual tests should cover:

- One conversion per file kind.
- Batch conversion.
- Cancellation.
- Output rename mode.
- Output overwrite mode.
- Settings persistence.
- Shell install and uninstall.
- Missing external tool behavior.

## Common development problems

| Problem | Cause | Fix |
| --- | --- | --- |
| Conversion works in terminal but not from Explorer | Shell command arguments or registration are wrong | Inspect registered command and run it manually |
| Tool works on one machine only | Dependency is available through `PATH` but not bundled | Use sidecars or `XOLARIQ_TOOLS_DIR` |
| UI does not update | Progress event is not emitted or listener is not attached | Check backend event emission and frontend event subscription |
| Settings appear ignored | Job was already queued with an older snapshot | Submit a new job after changing settings |
