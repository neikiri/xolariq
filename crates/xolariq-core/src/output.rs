use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::format::Format;
use crate::settings::{OutputMode, Settings};

/// Resolve the absolute output path for a conversion job.
///
/// Resolution rules:
/// 1. The directory is `settings.output_folder` if set, otherwise the input's
///    parent directory.
/// 2. The base filename is the input's stem with the target extension.
/// 3. If a file already exists at that path, [`OutputMode`] decides what to do:
///    - `Overwrite` returns the original path (caller is responsible for the
///      destructive write).
///    - `Rename` walks `name.ext`, `name (1).ext`, `name (2).ext`, ... until
///      a non-existent path is found.
pub fn resolve_output_path(input: &Path, target: Format, settings: &Settings) -> Result<PathBuf> {
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| Error::UnsupportedSource(input.to_path_buf()))?;

    let dir = match &settings.output_folder {
        Some(p) => p.clone(),
        None => input
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from(".")),
    };

    let filename = format!("{stem}.{ext}", ext = target.extension());
    let candidate = dir.join(filename);

    if !candidate.exists() {
        return Ok(candidate);
    }

    match settings.output_mode {
        OutputMode::Overwrite => Ok(candidate),
        OutputMode::Rename => {
            for n in 1..u32::MAX {
                let renamed = dir.join(format!("{stem} ({n}).{ext}", ext = target.extension()));
                if !renamed.exists() {
                    return Ok(renamed);
                }
            }
            Err(Error::Internal(
                "exhausted rename attempts; this should never happen".into(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn places_output_next_to_input_by_default() {
        let dir = tempdir().unwrap();
        let input = dir.path().join("clip.wav");
        std::fs::write(&input, b"fake").unwrap();
        let settings = Settings::default();
        let out = resolve_output_path(&input, Format::Mp3, &settings).unwrap();
        assert_eq!(out, dir.path().join("clip.mp3"));
    }

    #[test]
    fn rename_mode_avoids_collisions() {
        let dir = tempdir().unwrap();
        let input = dir.path().join("clip.wav");
        std::fs::write(&input, b"fake").unwrap();
        std::fs::write(dir.path().join("clip.mp3"), b"existing").unwrap();
        let settings = Settings::default();
        let out = resolve_output_path(&input, Format::Mp3, &settings).unwrap();
        assert_eq!(out, dir.path().join("clip (1).mp3"));
    }

    #[test]
    fn overwrite_mode_returns_existing_path() {
        let dir = tempdir().unwrap();
        let input = dir.path().join("clip.wav");
        std::fs::write(&input, b"fake").unwrap();
        std::fs::write(dir.path().join("clip.mp3"), b"existing").unwrap();
        let settings = Settings {
            output_mode: OutputMode::Overwrite,
            ..Settings::default()
        };
        let out = resolve_output_path(&input, Format::Mp3, &settings).unwrap();
        assert_eq!(out, dir.path().join("clip.mp3"));
    }
}
