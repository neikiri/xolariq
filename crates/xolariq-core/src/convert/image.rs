use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::error::Result;
use crate::ffmpeg::run_ffmpeg;
use crate::format::Format;
use crate::queue::ConvertOptions;
use crate::settings::Settings;

/// Image → image conversion via ffmpeg.
///
/// HEIC and AVIF support depend on the linked ffmpeg providing libheif /
/// libaom (both are present in the official Windows builds from gyan.dev).
/// If the linked ffmpeg lacks support, the underlying [`run_ffmpeg`] call
/// surfaces the original ffmpeg error verbatim — we deliberately don't
/// pre-check codec availability since that would force a separate probe.
pub async fn convert(
    input: &Path,
    output: &Path,
    _source: Format,
    target: Format,
    _options: &ConvertOptions,
    settings: &Settings,
    cancel: Arc<AtomicBool>,
    on_progress: impl Fn(Option<f32>) + Send + Sync + Clone + 'static,
) -> Result<()> {
    let ffmpeg = settings.ffmpeg_executable();

    let input_str = input.to_string_lossy();
    let output_str = output.to_string_lossy();

    let mut args: Vec<&str> = vec!["-i", &input_str];

    match target {
        Format::Png => args.extend(["-pix_fmt", "rgba"]),
        Format::Jpg => args.extend(["-q:v", "2", "-pix_fmt", "yuvj420p"]),
        Format::Webp => args.extend(["-quality", "85"]),
        Format::Avif => args.extend(["-c:v", "libaom-av1", "-still-picture", "1", "-crf", "30"]),
        Format::Heic => args.extend(["-c:v", "libx265", "-crf", "28"]),
        Format::Ico => args.extend(["-vf", "scale=256:256:force_original_aspect_ratio=decrease"]),
        _ => unreachable!("image::convert called with non-image target {:?}", target),
    }

    args.push(&output_str);

    // Image conversions are effectively instantaneous; we do not have a
    // meaningful duration to probe so progress reports as `None` until the
    // process exits with success.
    run_ffmpeg(&ffmpeg, &args, None, cancel, on_progress).await
}
