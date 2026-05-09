//! Xolariq COM IExplorerCommand shell extension.
//!
//! # What this is
//!
//! A self-registering Windows COM in-process server (`xolariq_shellext.dll`)
//! that exposes a single root [`IExplorerCommand`] — *"Convert with
//! Xolariq"* — that Explorer renders in the file context menu. The root
//! command exposes a flat list of sub-commands (one per supported target
//! format) via [`IExplorerCommandProvider`] / [`IEnumExplorerCommand`],
//! which Explorer shows as a cascading submenu.
//!
//! # Why a COM extension
//!
//! Plain registry verbs (the `xolariq-shell` crate) are picked up by
//! Windows 10 cleanly but Windows 11 25H2 routes them under
//! *"Show more options"*. To appear in the default Win11 context menu we
//! need an `IExplorerCommand` handler — and, eventually, a sparse MSIX
//! package that points at this DLL via
//! `<desktop4:FileExplorerContextMenus>` in the AppxManifest. The
//! scaffolding here is the foundation; see `ARCHITECTURE.md` for the
//! current state of the MSIX wrapper.
//!
//! # Layout
//!
//! * [`exports`] — DLL entrypoints (`DllMain`, `DllGetClassObject`,
//!   `DllRegisterServer`, `DllUnregisterServer`, `DllCanUnloadNow`).
//! * [`factory`] — minimal [`IClassFactory`] for the root command.
//! * [`root`] — the root command's [`IExplorerCommand`] implementation.
//! * [`registration`] — registry helpers used by the self-registration
//!   exports.
//! * [`formats`] — the static format/kind table the extension renders
//!   from (kept duplicated from `xolariq-core` so the DLL stays small
//!   and free of tokio / directories / serde dependencies).
//!
//! * [`subcommands`] — `IEnumExplorerCommand` and leaf `IExplorerCommand`
//!   items (one per target format, plus a separator and a "Settings"
//!   entry that opens the Xolariq Settings window).

#![cfg(windows)]
#![allow(non_snake_case)]
#![deny(unsafe_op_in_unsafe_fn)]

mod exports;
mod factory;
mod formats;
mod registration;
mod root;
mod subcommands;

pub use exports::*;

use windows::core::GUID;

/// CLSID under which the root command is registered.
///
/// Generated once with `uuidgen`; **do not change** — that would break
/// every existing install. Stable identifiers are also why we don't pull
/// this from a build script or environment variable.
pub(crate) const CLSID_XOLARIQ_ROOT_COMMAND: GUID =
    GUID::from_u128(0xa4f1d8e2_7c5b_4a9c_b1e3_2f8d9c4a5b7e);

/// Display name shown in `regsvr32` errors and `regedit`. Not user-facing
/// during normal operation.
pub(crate) const SHELL_EXT_FRIENDLY_NAME: &str = "Xolariq Shell Extension";
