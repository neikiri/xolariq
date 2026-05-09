//! Converter dispatch.
//!
//! [`convert_one`] is the single entry point used by the queue. It looks up
//! the source format from the input path, validates the source/target pair
//! belong to the same [`crate::FileKind`], and forwards to the kind-specific
//! converter.
//!
//! Each per-kind module is responsible for actually invoking the right
//! external tool (ffmpeg / pandoc / 7z). The dispatcher itself remains
//! purely declarative so adding a new kind is a one-line match arm.
//!
//! All per-kind `convert` functions share the same wide signature on
//! purpose — the dispatcher needs to forward every argument verbatim, and
//! introducing a `ConvertContext` struct just to silence one clippy lint
//! would push complexity into every caller without changing behaviour.
#![allow(clippy::too_many_arguments)]

use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::detect::detect_format;
use crate::error::{Error, Result};
use crate::format::Format;
use crate::kind::FileKind;
use crate::output::resolve_output_path;
use crate::queue::ConvertOptions;
use crate::settings::Settings;

pub mod archive;
pub mod audio;
pub mod document;
pub mod image;
pub mod video;

/// Result of a single conversion: the absolute path of the produced file.
#[derive(Debug, Clone)]
pub struct ConvertOutcome {
    pub output: std::path::PathBuf,
}

/// Dispatch a single conversion. The caller is expected to have already
/// gated this behind queue/cancel logic; this function does not enforce
/// concurrency limits itself.
pub async fn convert_one(
    input: &Path,
    target: Format,
    options: &ConvertOptions,
    settings: &Settings,
    cancel: Arc<AtomicBool>,
    on_progress: impl Fn(Option<f32>) + Send + Sync + Clone + 'static,
) -> Result<ConvertOutcome> {
    if !input.exists() {
        return Err(Error::InputNotFound(input.to_path_buf()));
    }

    let source = detect_format(input)?;
    if source.kind() != target.kind() {
        return Err(Error::unsupported_conversion(
            source.label(),
            target.label(),
        ));
    }

    let output = resolve_output_path(input, target, settings)?;

    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let progress = on_progress.clone();
    match target.kind() {
        FileKind::Audio => {
            audio::convert(
                input, &output, source, target, options, settings, cancel, progress,
            )
            .await?;
        }
        FileKind::Video => {
            video::convert(
                input, &output, source, target, options, settings, cancel, progress,
            )
            .await?;
        }
        FileKind::Image => {
            image::convert(
                input, &output, source, target, options, settings, cancel, progress,
            )
            .await?;
        }
        FileKind::Document => {
            document::convert(
                input, &output, source, target, options, settings, cancel, progress,
            )
            .await?;
        }
        FileKind::Archive => {
            archive::convert(
                input, &output, source, target, options, settings, cancel, progress,
            )
            .await?;
        }
    }

    Ok(ConvertOutcome { output })
}
