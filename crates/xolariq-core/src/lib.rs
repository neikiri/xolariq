//! Xolariq conversion engine.
//!
//! This crate is the platform-agnostic core of Xolariq. It provides:
//!
//! * File-type detection and format mapping ([`detect`], [`format`], [`kind`]).
//! * Persisted user settings ([`settings`]).
//! * Output-path resolution honoring overwrite/rename modes ([`output`]).
//! * A sequential, cancellable conversion queue ([`queue`]).
//! * Converter dispatch by file kind ([`convert`]).
//! * A thin async wrapper around `ffmpeg` ([`ffmpeg`]).
//!
//! All UI/shell concerns (Tauri, Windows registry, notifications) live in the
//! `app` and `xolariq-shell` crates so this crate stays portable and testable.

pub mod convert;
pub mod detect;
pub mod error;
pub mod ffmpeg;
pub mod format;
pub mod kind;
pub mod output;
pub mod progress;
pub mod queue;
pub mod settings;
pub mod tools;

pub use error::{Error, Result};
pub use format::{Format, FormatList};
pub use kind::FileKind;
pub use progress::{ProgressEvent, ProgressSink};
pub use queue::{ConvertOptions, Job, JobId, QueueHandle};
pub use settings::{OutputMode, Settings};
