use serde::{Deserialize, Serialize};

use crate::kind::FileKind;

/// Every output/input format Xolariq Free knows about.
///
/// Adding a new format is a three-step change:
/// 1. Add a variant here.
/// 2. Update [`Format::kind`], [`Format::extension`], and [`Format::from_extension`].
/// 3. Teach the relevant module under [`crate::convert`] how to produce/consume it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    // Audio
    Mp3,
    Wav,
    Flac,
    Aac,
    Ogg,
    Opus,
    // Video
    Mp4,
    Mkv,
    Webm,
    Mov,
    Avi,
    Gif,
    // Image
    Png,
    Jpg,
    Webp,
    Avif,
    Heic,
    Ico,
    // Document
    Pdf,
    Docx,
    Epub,
    Txt,
    Html,
    Markdown,
    // Archive
    Zip,
    SevenZ,
    Tar,
    Gz,
    Rar,
}

impl Format {
    /// Canonical file extension without the leading dot.
    pub fn extension(self) -> &'static str {
        match self {
            Format::Mp3 => "mp3",
            Format::Wav => "wav",
            Format::Flac => "flac",
            Format::Aac => "aac",
            Format::Ogg => "ogg",
            Format::Opus => "opus",
            Format::Mp4 => "mp4",
            Format::Mkv => "mkv",
            Format::Webm => "webm",
            Format::Mov => "mov",
            Format::Avi => "avi",
            Format::Gif => "gif",
            Format::Png => "png",
            Format::Jpg => "jpg",
            Format::Webp => "webp",
            Format::Avif => "avif",
            Format::Heic => "heic",
            Format::Ico => "ico",
            Format::Pdf => "pdf",
            Format::Docx => "docx",
            Format::Epub => "epub",
            Format::Txt => "txt",
            Format::Html => "html",
            Format::Markdown => "md",
            Format::Zip => "zip",
            Format::SevenZ => "7z",
            Format::Tar => "tar",
            Format::Gz => "gz",
            Format::Rar => "rar",
        }
    }

    pub fn kind(self) -> FileKind {
        match self {
            Format::Mp3 | Format::Wav | Format::Flac | Format::Aac | Format::Ogg | Format::Opus => {
                FileKind::Audio
            }
            Format::Mp4 | Format::Mkv | Format::Webm | Format::Mov | Format::Avi | Format::Gif => {
                FileKind::Video
            }
            Format::Png
            | Format::Jpg
            | Format::Webp
            | Format::Avif
            | Format::Heic
            | Format::Ico => FileKind::Image,
            Format::Pdf
            | Format::Docx
            | Format::Epub
            | Format::Txt
            | Format::Html
            | Format::Markdown => FileKind::Document,
            Format::Zip | Format::SevenZ | Format::Tar | Format::Gz | Format::Rar => {
                FileKind::Archive
            }
        }
    }

    /// Map a (case-insensitive) extension to a [`Format`]. Accepts both
    /// canonical extensions (`mp3`, `7z`, `md`) and a few common aliases
    /// (`jpeg`, `markdown`, `tgz`, `htm`).
    pub fn from_extension(ext: &str) -> Option<Format> {
        let ext = ext.trim_start_matches('.').to_ascii_lowercase();
        Some(match ext.as_str() {
            "mp3" => Format::Mp3,
            "wav" => Format::Wav,
            "flac" => Format::Flac,
            "aac" | "m4a" => Format::Aac,
            "ogg" | "oga" => Format::Ogg,
            "opus" => Format::Opus,
            "mp4" | "m4v" => Format::Mp4,
            "mkv" => Format::Mkv,
            "webm" => Format::Webm,
            "mov" | "qt" => Format::Mov,
            "avi" => Format::Avi,
            "gif" => Format::Gif,
            "png" => Format::Png,
            "jpg" | "jpeg" => Format::Jpg,
            "webp" => Format::Webp,
            "avif" => Format::Avif,
            "heic" | "heif" => Format::Heic,
            "ico" => Format::Ico,
            "pdf" => Format::Pdf,
            "docx" => Format::Docx,
            "epub" => Format::Epub,
            "txt" | "text" => Format::Txt,
            "html" | "htm" => Format::Html,
            "md" | "markdown" => Format::Markdown,
            "zip" => Format::Zip,
            "7z" => Format::SevenZ,
            "tar" => Format::Tar,
            "gz" | "tgz" => Format::Gz,
            "rar" => Format::Rar,
            _ => return None,
        })
    }

    pub fn label(self) -> &'static str {
        match self {
            Format::Mp3 => "MP3",
            Format::Wav => "WAV",
            Format::Flac => "FLAC",
            Format::Aac => "AAC",
            Format::Ogg => "OGG",
            Format::Opus => "Opus",
            Format::Mp4 => "MP4",
            Format::Mkv => "MKV",
            Format::Webm => "WebM",
            Format::Mov => "MOV",
            Format::Avi => "AVI",
            Format::Gif => "GIF",
            Format::Png => "PNG",
            Format::Jpg => "JPG",
            Format::Webp => "WebP",
            Format::Avif => "AVIF",
            Format::Heic => "HEIC",
            Format::Ico => "ICO",
            Format::Pdf => "PDF",
            Format::Docx => "DOCX",
            Format::Epub => "EPUB",
            Format::Txt => "TXT",
            Format::Html => "HTML",
            Format::Markdown => "Markdown",
            Format::Zip => "ZIP",
            Format::SevenZ => "7z",
            Format::Tar => "TAR",
            Format::Gz => "GZ",
            Format::Rar => "RAR",
        }
    }

    pub const ALL: &'static [Format] = &[
        Format::Mp3,
        Format::Wav,
        Format::Flac,
        Format::Aac,
        Format::Ogg,
        Format::Opus,
        Format::Mp4,
        Format::Mkv,
        Format::Webm,
        Format::Mov,
        Format::Avi,
        Format::Gif,
        Format::Png,
        Format::Jpg,
        Format::Webp,
        Format::Avif,
        Format::Heic,
        Format::Ico,
        Format::Pdf,
        Format::Docx,
        Format::Epub,
        Format::Txt,
        Format::Html,
        Format::Markdown,
        Format::Zip,
        Format::SevenZ,
        Format::Tar,
        Format::Gz,
        Format::Rar,
    ];
}

/// Helper for filtering [`Format::ALL`] by [`FileKind`].
pub struct FormatList;

impl FormatList {
    pub fn for_kind(kind: FileKind) -> Vec<Format> {
        Format::ALL
            .iter()
            .copied()
            .filter(|f| f.kind() == kind)
            .collect()
    }

    /// Returns the list of plausible target formats for an input format.
    /// We intentionally exclude the source format from the suggestion list
    /// so the context menu never offers a no-op conversion.
    pub fn targets_for(source: Format) -> Vec<Format> {
        FormatList::for_kind(source.kind())
            .into_iter()
            .filter(|f| *f != source)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_roundtrip() {
        for &fmt in Format::ALL {
            let ext = fmt.extension();
            assert_eq!(Format::from_extension(ext), Some(fmt), "ext: {ext}");
        }
    }

    #[test]
    fn aliases_resolve() {
        assert_eq!(Format::from_extension("JPEG"), Some(Format::Jpg));
        assert_eq!(Format::from_extension(".markdown"), Some(Format::Markdown));
        assert_eq!(Format::from_extension("tgz"), Some(Format::Gz));
    }

    #[test]
    fn targets_for_excludes_source() {
        let targets = FormatList::targets_for(Format::Mp3);
        assert!(!targets.contains(&Format::Mp3));
        assert!(targets.contains(&Format::Wav));
    }
}
