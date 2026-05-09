//! Static format / file-kind table used by the shell extension.
//!
//! Kept deliberately separate from `xolariq-core::format` so this DLL
//! does not pull tokio, serde, directories, etc. into every Explorer
//! process that loads it. The list **must** stay in sync with
//! `crates/xolariq-core/src/format.rs` — when adding a format there,
//! mirror the entry here.

/// Logical kind that maps onto the cascade groups Explorer renders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FileKind {
    Audio,
    Video,
    Image,
    Document,
    Archive,
}

/// One supported format.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Format {
    /// Lowercase extension without the leading dot (e.g. `"mp3"`).
    pub ext: &'static str,
    /// Human-readable label used as the verb title (e.g. `"To MP3"`).
    pub label: &'static str,
    pub kind: FileKind,
}

/// Full table of supported source/target formats. Mirrors the table in
/// `xolariq-core::format`.
pub(crate) const ALL: &[Format] = &[
    Format {
        ext: "mp3",
        label: "MP3",
        kind: FileKind::Audio,
    },
    Format {
        ext: "wav",
        label: "WAV",
        kind: FileKind::Audio,
    },
    Format {
        ext: "flac",
        label: "FLAC",
        kind: FileKind::Audio,
    },
    Format {
        ext: "aac",
        label: "AAC",
        kind: FileKind::Audio,
    },
    Format {
        ext: "ogg",
        label: "OGG",
        kind: FileKind::Audio,
    },
    Format {
        ext: "opus",
        label: "Opus",
        kind: FileKind::Audio,
    },
    Format {
        ext: "mp4",
        label: "MP4",
        kind: FileKind::Video,
    },
    Format {
        ext: "mkv",
        label: "MKV",
        kind: FileKind::Video,
    },
    Format {
        ext: "webm",
        label: "WebM",
        kind: FileKind::Video,
    },
    Format {
        ext: "mov",
        label: "MOV",
        kind: FileKind::Video,
    },
    Format {
        ext: "avi",
        label: "AVI",
        kind: FileKind::Video,
    },
    Format {
        ext: "gif",
        label: "GIF",
        kind: FileKind::Video,
    },
    Format {
        ext: "png",
        label: "PNG",
        kind: FileKind::Image,
    },
    Format {
        ext: "jpg",
        label: "JPG",
        kind: FileKind::Image,
    },
    Format {
        ext: "webp",
        label: "WebP",
        kind: FileKind::Image,
    },
    Format {
        ext: "avif",
        label: "AVIF",
        kind: FileKind::Image,
    },
    Format {
        ext: "heic",
        label: "HEIC",
        kind: FileKind::Image,
    },
    Format {
        ext: "ico",
        label: "ICO",
        kind: FileKind::Image,
    },
    Format {
        ext: "pdf",
        label: "PDF",
        kind: FileKind::Document,
    },
    Format {
        ext: "docx",
        label: "DOCX",
        kind: FileKind::Document,
    },
    Format {
        ext: "epub",
        label: "EPUB",
        kind: FileKind::Document,
    },
    Format {
        ext: "txt",
        label: "TXT",
        kind: FileKind::Document,
    },
    Format {
        ext: "html",
        label: "HTML",
        kind: FileKind::Document,
    },
    Format {
        ext: "md",
        label: "Markdown",
        kind: FileKind::Document,
    },
    Format {
        ext: "zip",
        label: "ZIP",
        kind: FileKind::Archive,
    },
    Format {
        ext: "7z",
        label: "7z",
        kind: FileKind::Archive,
    },
    Format {
        ext: "tar",
        label: "TAR",
        kind: FileKind::Archive,
    },
    Format {
        ext: "gz",
        label: "GZIP",
        kind: FileKind::Archive,
    },
];

/// Look up a format by its (case-insensitive) extension.
pub(crate) fn from_extension(ext: &str) -> Option<Format> {
    let ext = ext.trim_start_matches('.');
    ALL.iter()
        .copied()
        .find(|f| f.ext.eq_ignore_ascii_case(ext))
}

/// All formats that share a kind, excluding `from_self`. Used to build
/// the cascade — converting MP3 should not offer "To MP3" as a target.
pub(crate) fn targets_for(source: Format) -> impl Iterator<Item = Format> {
    let kind = source.kind;
    let from_ext = source.ext;
    ALL.iter()
        .copied()
        .filter(move |f| f.kind == kind && !f.ext.eq_ignore_ascii_case(from_ext))
}
