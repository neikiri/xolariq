use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

/// Top-level error type for the Xolariq engine.
#[derive(Debug, Error)]
pub enum Error {
    #[error("input file does not exist: {0}")]
    InputNotFound(PathBuf),

    #[error("unsupported source format for file: {0}")]
    UnsupportedSource(PathBuf),

    #[error("conversion from {from} to {to} is not supported")]
    UnsupportedConversion { from: String, to: String },

    #[error("required external tool not found: {tool}. {hint}")]
    ToolNotFound { tool: String, hint: String },

    #[error("external tool '{tool}' failed with exit code {code:?}: {stderr}")]
    ToolFailed {
        tool: String,
        code: Option<i32>,
        stderr: String,
    },

    #[error("conversion was cancelled")]
    Cancelled,

    #[error("output path already exists and overwrite is disabled: {0}")]
    OutputExists(PathBuf),

    #[error("settings error: {0}")]
    Settings(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("invalid configuration: {0}")]
    Config(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl Error {
    pub fn tool_not_found(tool: impl Into<String>, hint: impl Into<String>) -> Self {
        Self::ToolNotFound {
            tool: tool.into(),
            hint: hint.into(),
        }
    }

    pub fn unsupported_conversion(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self::UnsupportedConversion {
            from: from.into(),
            to: to.into(),
        }
    }
}
