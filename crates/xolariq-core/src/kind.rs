use serde::{Deserialize, Serialize};

use crate::format::Format;

/// Coarse-grained classification of a file by its expected handler.
///
/// Every [`crate::format::Format`] maps to exactly one [`FileKind`]. The
/// kind drives converter dispatch and the dynamic context-menu submenu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileKind {
    Audio,
    Video,
    Image,
    Document,
    Archive,
}

impl FileKind {
    pub const ALL: &'static [FileKind] = &[
        FileKind::Audio,
        FileKind::Video,
        FileKind::Image,
        FileKind::Document,
        FileKind::Archive,
    ];

    pub fn label(self) -> &'static str {
        match self {
            FileKind::Audio => "Audio",
            FileKind::Video => "Video",
            FileKind::Image => "Image",
            FileKind::Document => "Document",
            FileKind::Archive => "Archive",
        }
    }

    /// Stable verb-name suffix used when registering this kind in the
    /// Windows Shell. Kept ASCII and CamelCase to keep regedit-friendly.
    pub fn verb_suffix(self) -> &'static str {
        match self {
            FileKind::Audio => "Audio",
            FileKind::Video => "Video",
            FileKind::Image => "Image",
            FileKind::Document => "Document",
            FileKind::Archive => "Archive",
        }
    }

    /// Windows Search Conditions string used as the `AppliesTo` registry
    /// value when registering a context-menu verb under
    /// `HKCU\Software\Classes\*\shell\<verb>`.
    ///
    /// Audio / Video / Image / Document use the semantic `System.Kind`
    /// property which Explorer evaluates without needing a file-type
    /// association: `music`, `video`, `picture`, `document` are all
    /// well-known values.
    ///
    /// Archive has no `System.Kind` value (Explorer's `compressed`
    /// `System.PerceivedType` is unreliable across Windows builds — see
    /// PR #1 diagnostics), so we fall back to listing the supported
    /// archive extensions explicitly via `System.FileExtension := …`.
    /// The explicit `:=` operator is required: the implicit colon-style
    /// `System.FileExtension:".zip"` doesn't filter on Windows 11 25H2.
    pub fn applies_to_query(self) -> String {
        match self {
            FileKind::Audio => "System.Kind:=\"music\"".into(),
            FileKind::Video => "System.Kind:=\"video\"".into(),
            FileKind::Image => "System.Kind:=\"picture\"".into(),
            FileKind::Document => "System.Kind:=\"document\"".into(),
            FileKind::Archive => Format::ALL
                .iter()
                .filter(|f| f.kind() == FileKind::Archive)
                .map(|f| format!("System.FileExtension:=\".{}\"", f.extension()))
                .collect::<Vec<_>>()
                .join(" OR "),
        }
    }
}
