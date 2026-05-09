# Icons

Drop application icons here before building the bundle:

| File              | Purpose                                              |
| ----------------- | ---------------------------------------------------- |
| `32x32.png`       | Small bundle icon                                    |
| `128x128.png`     | Medium bundle icon                                   |
| `icon.ico`        | Multi-resolution Windows icon (used by exe + bundle) |

The Tauri CLI ships a generator that takes a single source PNG and emits
every size/format the bundler needs:

```powershell
cargo tauri icon path\to\source.png
```

Run that command **once** from the repository root after providing your own
source image. The generator writes its outputs into this directory.
