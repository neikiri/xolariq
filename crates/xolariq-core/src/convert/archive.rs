use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::process::Command;

use crate::error::{Error, Result};
use crate::format::Format;
use crate::queue::ConvertOptions;
use crate::settings::Settings;

/// Archive → archive conversion driven by 7-Zip's CLI.
///
/// We delegate to the system `7z` binary because it speaks every archive
/// format Xolariq supports (zip, 7z, tar, gz, rar read-only) without us
/// having to ship multiple Rust-native codecs. The flow is:
///
/// 1. Extract the source archive into a per-job temp directory.
/// 2. Re-archive the extracted tree into the target format.
/// 3. Best-effort cleanup of the temp directory.
///
/// `rar` is intentionally read-only because creating RAR archives requires
/// the proprietary `Rar.exe` (not 7-Zip). Attempting `→ rar` yields a clear
/// [`Error::UnsupportedConversion`].
pub async fn convert(
    input: &Path,
    output: &Path,
    source: Format,
    target: Format,
    _options: &ConvertOptions,
    settings: &Settings,
    cancel: Arc<AtomicBool>,
    on_progress: impl Fn(Option<f32>) + Send + Sync + Clone + 'static,
) -> Result<()> {
    if matches!(target, Format::Rar) {
        return Err(Error::unsupported_conversion(
            source.label(),
            target.label(),
        ));
    }

    let seven_zip = settings.seven_zip_executable();
    on_progress(Some(0.0));

    let temp_root = tempdir_for_job()?;

    extract_with_7z(&seven_zip, input, &temp_root, cancel.clone()).await?;
    on_progress(Some(0.5));

    repack_with_7z(&seven_zip, &temp_root, output, target, cancel).await?;
    on_progress(Some(1.0));

    let _ = std::fs::remove_dir_all(&temp_root);

    Ok(())
}

fn tempdir_for_job() -> Result<PathBuf> {
    let base = std::env::temp_dir().join(format!("xolariq-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&base)?;
    Ok(base)
}

async fn extract_with_7z(
    seven_zip: &Path,
    input: &Path,
    dest: &Path,
    cancel: Arc<AtomicBool>,
) -> Result<()> {
    let output_arg = format!("-o{}", dest.display());
    let mut child = Command::new(seven_zip)
        .arg("x")
        .arg("-y")
        .arg(&output_arg)
        .arg(input)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .stdin(Stdio::null())
        .spawn()
        .map_err(map_7z_spawn_err)?;

    wait_with_cancel(&mut child, &cancel).await?;
    let output = child.wait_with_output().await?;
    if !output.status.success() {
        return Err(Error::ToolFailed {
            tool: "7z".into(),
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }
    Ok(())
}

async fn repack_with_7z(
    seven_zip: &Path,
    src_dir: &Path,
    output: &Path,
    target: Format,
    cancel: Arc<AtomicBool>,
) -> Result<()> {
    // Best-effort overwrite — 7z's `a` verb overwrites by default but does
    // not replace an existing archive in-place; remove first.
    if output.exists() {
        let _ = std::fs::remove_file(output);
    }

    let pattern = format!("{}{}*", src_dir.display(), std::path::MAIN_SEPARATOR);
    let archive_type = format_to_7z_type(target)?;

    let mut child = Command::new(seven_zip)
        .arg("a")
        .arg(format!("-t{archive_type}"))
        .arg(output)
        .arg(&pattern)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .stdin(Stdio::null())
        .spawn()
        .map_err(map_7z_spawn_err)?;

    wait_with_cancel(&mut child, &cancel).await?;
    let output = child.wait_with_output().await?;
    if !output.status.success() {
        return Err(Error::ToolFailed {
            tool: "7z".into(),
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }
    Ok(())
}

fn format_to_7z_type(format: Format) -> Result<&'static str> {
    Ok(match format {
        Format::Zip => "zip",
        Format::SevenZ => "7z",
        Format::Tar => "tar",
        Format::Gz => "gzip",
        Format::Rar => return Err(Error::unsupported_conversion("archive", "rar")),
        _ => return Err(Error::unsupported_conversion("archive", format.label())),
    })
}

fn map_7z_spawn_err(e: std::io::Error) -> Error {
    match e.kind() {
        std::io::ErrorKind::NotFound => Error::tool_not_found(
            "7z",
            "Install 7-Zip from https://www.7-zip.org/ and ensure 7z (Windows) or p7zip is on PATH.",
        ),
        _ => Error::Io(e),
    }
}

async fn wait_with_cancel(
    child: &mut tokio::process::Child,
    cancel: &Arc<AtomicBool>,
) -> Result<()> {
    loop {
        if cancel.load(Ordering::Relaxed) {
            let _ = child.start_kill();
            let _ = child.wait().await;
            return Err(Error::Cancelled);
        }
        match child.try_wait() {
            Ok(Some(_)) => return Ok(()),
            Ok(None) => tokio::time::sleep(std::time::Duration::from_millis(100)).await,
            Err(e) => return Err(Error::Io(e)),
        }
    }
}
