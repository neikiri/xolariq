use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::error::Result;
use crate::ffmpeg::{probe_duration, run_ffmpeg};
use crate::format::Format;
use crate::queue::ConvertOptions;
use crate::settings::Settings;

/// Video → video conversion via ffmpeg.
///
/// We choose codec defaults that balance quality and compatibility and avoid
/// experimental features. CPU-only encoders are used so the free version has
/// no GPU dependency (premium GPU acceleration is intentionally out of scope).
pub async fn convert(
    input: &Path,
    output: &Path,
    _source: Format,
    target: Format,
    options: &ConvertOptions,
    settings: &Settings,
    cancel: Arc<AtomicBool>,
    on_progress: impl Fn(Option<f32>) + Send + Sync + Clone + 'static,
) -> Result<()> {
    let ffmpeg = settings.ffmpeg_executable();
    let duration = probe_duration(&ffmpeg, input).await.unwrap_or(None);

    let input_str = input.to_string_lossy();
    let output_str = output.to_string_lossy();
    let metadata_arg = if options.preserve_metadata { "0" } else { "-1" };

    let codec_args: Vec<&str> = match target {
        Format::Mp4 => vec![
            "-c:v",
            "libx264",
            "-preset",
            "medium",
            "-crf",
            "23",
            "-c:a",
            "aac",
            "-b:a",
            "192k",
            "-movflags",
            "+faststart",
        ],
        Format::Mkv => vec![
            "-c:v", "libx264", "-preset", "medium", "-crf", "23", "-c:a", "aac", "-b:a", "192k",
        ],
        Format::Webm => vec![
            "-c:v",
            "libvpx-vp9",
            "-b:v",
            "0",
            "-crf",
            "32",
            "-c:a",
            "libopus",
            "-b:a",
            "128k",
        ],
        Format::Mov => vec![
            "-c:v", "libx264", "-preset", "medium", "-crf", "23", "-c:a", "aac", "-b:a", "192k",
        ],
        Format::Avi => vec![
            "-c:v",
            "mpeg4",
            "-q:v",
            "5",
            "-c:a",
            "libmp3lame",
            "-q:a",
            "4",
        ],
        Format::Gif => vec![
            // GIF: drop audio, keep modest fps/scale to avoid huge files.
            "-vf",
            "fps=15,scale=480:-1:flags=lanczos",
            "-loop",
            "0",
        ],
        _ => unreachable!("video::convert called with non-video target {:?}", target),
    };

    let mut args: Vec<&str> = vec!["-i", &input_str, "-map_metadata", metadata_arg];
    if matches!(target, Format::Gif) {
        args.push("-an");
    }
    args.extend(codec_args);
    args.push(&output_str);

    run_ffmpeg(&ffmpeg, &args, duration, cancel, on_progress).await
}
