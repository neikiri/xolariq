# Supported Formats

Xolariq groups formats by file kind. Conversions are allowed within the same kind and rejected across incompatible kinds.

## Format matrix

| Kind | Supported formats | Primary tool |
| --- | --- | --- |
| Audio | `mp3`, `wav`, `flac`, `aac`, `ogg`, `opus` | FFmpeg |
| Video | `mp4`, `mkv`, `webm`, `mov`, `avi`, `gif` | FFmpeg |
| Image | `png`, `jpg`, `webp`, `avif`, `heic`, `ico` | FFmpeg |
| Document | `pdf`, `docx`, `epub`, `txt`, `html`, `md` | pandoc |
| Archive | `zip`, `7z`, `tar`, `gz`, `rar` | 7-Zip |

## Extension aliases

The format detector is currently extension-based. It accepts canonical extensions and selected aliases.

Common aliases include:

| Alias | Resolved format |
| --- | --- |
| `jpeg` | `jpg` |
| `markdown` | `md` |
| `htm` | `html` |
| `m4a` | `aac` |
| `m4v` | `mp4` |
| `qt` | `mov` |
| `oga` | `ogg` |
| `heif` | `heic` |
| `tgz` | `gz` |
| `text` | `txt` |

## Conversion rules

Xolariq validates the source format and target format before dispatching a converter.

The key rule is:

> Source and target must belong to the same file kind.

Examples:

| Input | Allowed targets | Rejected targets |
| --- | --- | --- |
| `song.wav` | `mp3`, `flac`, `aac`, `ogg`, `opus` | `pdf`, `zip`, `png` |
| `photo.heic` | `png`, `jpg`, `webp`, `avif`, `ico` | `mp3`, `docx`, `7z` |
| `book.html` | `pdf`, `docx`, `epub`, `txt`, `md` | `mp4`, `zip`, `jpg` |
| `archive.rar` | archive targets except unsupported outputs | audio/video/document targets |

The context menu excludes the source format from target suggestions to avoid no-op conversions.

## PDF behavior

PDF output is supported through pandoc, but PDF generation typically requires a LaTeX installation such as MiKTeX.

Important notes:

- PDF output depends on the local pandoc/LaTeX toolchain.
- PDF input is not treated as a general-purpose free document input path.
- When PDF output fails, the most common cause is missing LaTeX rather than Xolariq itself.

## RAR behavior

RAR is read-only in the practical Xolariq workflow.

Xolariq can extract or convert from `.rar` archives when the configured 7-Zip tool supports reading them. Creating `.rar` output is not supported because it requires proprietary RAR tooling.

## Adding formats

Adding a new format usually requires changes in three areas:

1. Add the format variant to the central format definition.
2. Map the extension, label, and file kind.
3. Teach the relevant converter how to produce or consume it.

If the format belongs to an existing kind and the external tool already supports it, the change is usually small. If it introduces a new kind, the dispatcher and shell integration must also be extended.
