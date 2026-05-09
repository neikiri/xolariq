use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::error::Result;
use crate::ffmpeg::{probe_duration, run_ffmpeg};
use crate::format::Format;
use crate::queue::ConvertOptions;
use crate::settings::Settings;

/// Audio → audio conversion via ffmpeg.
///
/// The codec/quality flags below favour broad compatibility and reasonable
/// defaults rather than exposing every knob ffmpeg supports — the free tier
/// of Xolariq deliberately ships without an "advanced encoding UI".
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
        Format::Mp3 => vec!["-c:a", "libmp3lame", "-q:a", "2"],
        Format::Wav => vec!["-c:a", "pcm_s16le"],
        Format::Flac => vec!["-c:a", "flac", "-compression_level", "5"],
        Format::Aac => vec!["-c:a", "aac", "-b:a", "192k"],
        Format::Ogg => vec!["-c:a", "libvorbis", "-q:a", "5"],
        Format::Opus => vec!["-c:a", "libopus", "-b:a", "128k"],
        _ => unreachable!("audio::convert called with non-audio target {:?}", target),
    };

    let mut args: Vec<&str> = vec!["-i", &input_str, "-vn", "-map_metadata", metadata_arg];
    args.extend(codec_args);
    args.push(&output_str);

    run_ffmpeg(&ffmpeg, &args, duration, cancel, on_progress).await
}
