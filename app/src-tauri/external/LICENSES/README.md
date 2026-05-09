# Bundled tool licenses

When you build a release MSI/NSIS bundle of Xolariq, the installer
ships three external command-line tools alongside `xolariq.exe`:

| Tool   | Sidecar name         | Upstream license            |
|--------|----------------------|-----------------------------|
| FFmpeg | `ffmpeg.exe`         | LGPL-2.1-or-later (default build) / GPL-2.0-or-later (`-gpl` build) |
| Pandoc | `pandoc.exe`         | GPL-2.0-or-later            |
| 7-Zip  | `7za.exe` (standalone) | LGPL-2.1-or-later, with [unRAR exception](https://www.7-zip.org/license.txt) |

The MPL-2.0 license that Xolariq itself ships under explicitly does
**not** sublicense any of these third-party binaries. To stay
compliant, drop each tool's upstream `LICENSE` file into this directory
before running `cargo tauri build`. The Tauri bundler picks them up via
the `bundle.resources` glob in `tauri.conf.json` and copies them into
the installed application directory.

Suggested filenames (keep the upstream content verbatim):

- `LICENSE-ffmpeg.txt`
- `LICENSE-pandoc.txt`
- `LICENSE-7zip.txt`

You may also include the matching `COPYING.LGPL`, `COPYING.GPLv2`, etc.
companion files if upstream ships them — Xolariq does not parse the
contents, only ensures they reach the installed location.

This file is intentionally tracked in git as a placeholder. The actual
license texts are not committed because they are large, lightly
versioned, and trivially obtainable from upstream releases. CI does
**not** download them — that is on whoever runs `cargo tauri build`.
