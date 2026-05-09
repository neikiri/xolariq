use std::path::Path;

use crate::error::{Error, Result};
use crate::format::Format;

/// Detect the source [`Format`] of a file from its extension.
///
/// Xolariq Free relies on the filename extension only — content sniffing is
/// reserved for a later iteration since it would force a synchronous read on
/// every right-click and slow down the context menu.
pub fn detect_format(path: &Path) -> Result<Format> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .ok_or_else(|| Error::UnsupportedSource(path.to_path_buf()))?;

    Format::from_extension(ext).ok_or_else(|| Error::UnsupportedSource(path.to_path_buf()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn detects_mp3() {
        assert_eq!(
            detect_format(&PathBuf::from("song.mp3")).unwrap(),
            Format::Mp3
        );
    }

    #[test]
    fn rejects_unknown() {
        assert!(detect_format(&PathBuf::from("foo.xyz")).is_err());
    }

    #[test]
    fn rejects_missing_extension() {
        assert!(detect_format(&PathBuf::from("README")).is_err());
    }
}
