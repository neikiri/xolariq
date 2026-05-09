use std::path::Path;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::error::{Error, Result};

/// Run `ffmpeg` with the supplied arguments and stream progress to `on_progress`.
///
/// Progress is parsed from the `-progress pipe:1` key/value stream that ffmpeg
/// writes when the option is set. We append the option ourselves if the caller
/// did not supply it, and we always insert `-nostats -hide_banner` to keep the
/// stderr channel free of noise.
///
/// `cancel` is checked between progress updates; setting it to `true` aborts
/// the child process and returns [`Error::Cancelled`].
pub async fn run_ffmpeg(
    ffmpeg_path: &Path,
    args: &[&str],
    duration_secs: Option<f64>,
    cancel: Arc<AtomicBool>,
    on_progress: impl Fn(Option<f32>) + Send + Sync + 'static,
) -> Result<()> {
    let mut command = Command::new(ffmpeg_path);
    command
        .arg("-hide_banner")
        .arg("-nostats")
        .arg("-y")
        .args(args)
        .arg("-progress")
        .arg("pipe:1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());

    let mut child = command.spawn().map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => Error::tool_not_found(
            "ffmpeg",
            "Install FFmpeg from https://ffmpeg.org/download.html and ensure it is on PATH or set the path in Settings.",
        ),
        _ => Error::Io(e),
    })?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| Error::Internal("ffmpeg stdout missing".into()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| Error::Internal("ffmpeg stderr missing".into()))?;

    // Capture stderr so we can attach it to ToolFailed if ffmpeg exits non-zero.
    let stderr_task = tokio::spawn(async move {
        let mut buf = String::new();
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            if buf.len() < 8192 {
                buf.push_str(&line);
                buf.push('\n');
            }
        }
        buf
    });

    let mut reader = BufReader::new(stdout).lines();
    let mut last_percent: Option<f32> = None;
    let mut current_us: u64 = 0;

    loop {
        if cancel.load(Ordering::Relaxed) {
            let _ = child.start_kill();
            let _ = child.wait().await;
            let _ = stderr_task.await;
            return Err(Error::Cancelled);
        }

        let next = tokio::select! {
            line = reader.next_line() => line,
            _ = tokio::time::sleep(std::time::Duration::from_millis(250)) => Ok(None),
        };

        match next {
            Ok(Some(line)) => {
                if let Some(rest) = line.strip_prefix("out_time_us=") {
                    if let Ok(us) = rest.trim().parse::<u64>() {
                        current_us = us;
                    }
                } else if let Some(rest) = line.strip_prefix("out_time_ms=") {
                    // Older ffmpeg emits microseconds under this key. Use it as a fallback.
                    if let Ok(us) = rest.trim().parse::<u64>() {
                        if current_us == 0 {
                            current_us = us;
                        }
                    }
                } else if line.starts_with("progress=") {
                    let percent = duration_secs
                        .filter(|d| *d > 0.0)
                        .map(|d| (current_us as f64 / 1_000_000.0 / d) as f32)
                        .map(|p| p.clamp(0.0, 1.0));
                    if percent != last_percent {
                        last_percent = percent;
                        on_progress(percent);
                    }
                }
            }
            Ok(None) => {
                // EOF: fall through to wait()
                if !line_pipe_open(&child) {
                    break;
                }
            }
            Err(e) => return Err(Error::Io(e)),
        }

        if let Ok(Some(_)) = child.try_wait() {
            break;
        }
    }

    let status = child.wait().await?;
    let stderr_output = stderr_task.await.unwrap_or_default();

    if status.success() {
        on_progress(Some(1.0));
        Ok(())
    } else {
        Err(Error::ToolFailed {
            tool: "ffmpeg".into(),
            code: status.code(),
            stderr: stderr_output,
        })
    }
}

fn line_pipe_open(_child: &tokio::process::Child) -> bool {
    // tokio::process::Child does not expose pipe state; we rely on try_wait()
    // in the caller to detect process exit and let the loop terminate.
    true
}

/// Probe a media file for its duration, in seconds. Used to give image-free
/// audio/video conversions an accurate progress percentage.
pub async fn probe_duration(ffmpeg_path: &Path, input: &Path) -> Result<Option<f64>> {
    // Re-use ffmpeg with `-i` and parse stderr; this avoids requiring ffprobe.
    let output = Command::new(ffmpeg_path)
        .arg("-hide_banner")
        .arg("-i")
        .arg(input)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .stdin(Stdio::null())
        .output()
        .await
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => Error::tool_not_found(
                "ffmpeg",
                "Install FFmpeg from https://ffmpeg.org/download.html and ensure it is on PATH.",
            ),
            _ => Error::Io(e),
        })?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    Ok(parse_duration(&stderr))
}

fn parse_duration(stderr: &str) -> Option<f64> {
    // Look for `Duration: HH:MM:SS.cs`
    let needle = "Duration: ";
    let idx = stderr.find(needle)?;
    let rest = &stderr[idx + needle.len()..];
    let end = rest.find(',').unwrap_or(rest.len());
    let ts = rest[..end].trim();
    let mut parts = ts.split(':');
    let h: f64 = parts.next()?.parse().ok()?;
    let m: f64 = parts.next()?.parse().ok()?;
    let s: f64 = parts.next()?.parse().ok()?;
    Some(h * 3600.0 + m * 60.0 + s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_duration_handles_typical_output() {
        let sample = "Input #0, wav, from 'a.wav':\n  Duration: 00:01:23.45, bitrate: 1411 kb/s\n";
        assert_eq!(parse_duration(sample), Some(83.45));
    }

    #[test]
    fn parse_duration_returns_none_when_missing() {
        assert_eq!(parse_duration("nope"), None);
    }
}
