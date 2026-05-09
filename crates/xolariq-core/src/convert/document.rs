use std::path::Path;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::process::Command;

use crate::error::{Error, Result};
use crate::format::Format;
use crate::queue::ConvertOptions;
use crate::settings::Settings;

/// Document conversion via [pandoc](https://pandoc.org/).
///
/// Pandoc handles the full grid of `txt`, `markdown`, `html`, `epub`, `docx`
/// pairs natively. PDF *output* requires a working LaTeX or `wkhtmltopdf`
/// install on the user's machine; PDF *input* is not supported by pandoc and
/// returns [`Error::UnsupportedConversion`]. Future iterations may layer a
/// poppler-based PDF reader in front of this dispatcher.
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
    if matches!(source, Format::Pdf) {
        return Err(Error::unsupported_conversion(
            source.label(),
            target.label(),
        ));
    }

    let pandoc = settings.pandoc_executable();

    on_progress(None);

    let from = pandoc_format(source);
    let to = pandoc_format(target);

    let mut command = Command::new(&pandoc);
    command
        .arg("--from")
        .arg(from)
        .arg("--to")
        .arg(to)
        .arg("--standalone")
        .arg("-o")
        .arg(output)
        .arg(input)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());

    let mut child = command.spawn().map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => Error::tool_not_found(
            "pandoc",
            "Install pandoc from https://pandoc.org/installing.html. PDF output additionally requires a LaTeX distribution (e.g. MiKTeX).",
        ),
        _ => Error::Io(e),
    })?;

    // Poll for cancellation while pandoc runs.
    loop {
        if cancel.load(Ordering::Relaxed) {
            let _ = child.start_kill();
            let _ = child.wait().await;
            return Err(Error::Cancelled);
        }
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => tokio::time::sleep(std::time::Duration::from_millis(100)).await,
            Err(e) => return Err(Error::Io(e)),
        }
    }

    let output_status = child.wait_with_output().await?;
    if output_status.status.success() {
        on_progress(Some(1.0));
        Ok(())
    } else {
        Err(Error::ToolFailed {
            tool: "pandoc".into(),
            code: output_status.status.code(),
            stderr: String::from_utf8_lossy(&output_status.stderr).into_owned(),
        })
    }
}

/// Map our [`Format`] enum to the strings pandoc expects.
fn pandoc_format(format: Format) -> &'static str {
    match format {
        Format::Markdown => "markdown",
        Format::Html => "html",
        Format::Txt => "plain",
        Format::Docx => "docx",
        Format::Epub => "epub",
        Format::Pdf => "pdf",
        _ => "plain",
    }
}
