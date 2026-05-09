//! Root [`IExplorerCommand`] — the *"Convert with Xolariq"* parent verb
//! that Explorer renders in the file context menu.
//!
//! The root command returns [`ECF_HASSUBCOMMANDS`] and delegates to
//! [`crate::subcommands::SubCommandEnum`] which enumerates one leaf
//! per supported target format, a separator, and a "Settings" entry.

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::sync::atomic::Ordering;

use windows::core::*;
use windows::Win32::Foundation::BOOL;
use windows::Win32::UI::Shell::{
    IEnumExplorerCommand, IExplorerCommand, IExplorerCommand_Impl, IShellItem, IShellItemArray,
    ECF_HASSUBCOMMANDS, ECS_ENABLED, SIGDN_FILESYSPATH,
};

use crate::exports::LIVE_OBJECTS;

/// Title shown to the user.
const ROOT_TITLE: PCWSTR = w!("Convert with Xolariq");

#[implement(IExplorerCommand)]
pub(crate) struct RootCommand;

impl Default for RootCommand {
    fn default() -> Self {
        // The in-process server tracks live objects so `DllCanUnloadNow`
        // can return `S_OK` once Explorer has dropped every reference.
        LIVE_OBJECTS.fetch_add(1, Ordering::AcqRel);
        Self
    }
}

impl Drop for RootCommand {
    fn drop(&mut self) {
        LIVE_OBJECTS.fetch_sub(1, Ordering::AcqRel);
    }
}

impl IExplorerCommand_Impl for RootCommand_Impl {
    fn GetTitle(&self, _items: Option<&IShellItemArray>) -> Result<PWSTR> {
        copy_to_pwstr(ROOT_TITLE)
    }

    fn GetIcon(&self, _items: Option<&IShellItemArray>) -> Result<PWSTR> {
        // Use the icon of `xolariq.exe` living next to the DLL.
        copy_to_pwstr(w!("xolariq.exe,0"))
    }

    fn GetToolTip(&self, _items: Option<&IShellItemArray>) -> Result<PWSTR> {
        copy_to_pwstr(w!("Convert this file to another format"))
    }

    fn GetCanonicalName(&self) -> Result<GUID> {
        Ok(crate::CLSID_XOLARIQ_ROOT_COMMAND)
    }

    fn GetState(&self, _items: Option<&IShellItemArray>, _ok_to_be_slow: BOOL) -> Result<u32> {
        Ok(ECS_ENABLED.0 as u32)
    }

    fn Invoke(
        &self,
        items: Option<&IShellItemArray>,
        _bind_ctx: Option<&windows::Win32::System::Com::IBindCtx>,
    ) -> Result<()> {
        let paths = collect_paths(items).unwrap_or_default();
        if paths.is_empty() {
            return Ok(());
        }
        spawn_xolariq(&paths);
        Ok(())
    }

    fn GetFlags(&self) -> Result<u32> {
        Ok(ECF_HASSUBCOMMANDS.0 as u32)
    }

    fn EnumSubCommands(&self) -> Result<IEnumExplorerCommand> {
        let enumerator = crate::subcommands::SubCommandEnum::all_formats();
        Ok(enumerator.into())
    }
}

/// Pull the filesystem path out of every `IShellItem` in the array.
/// Returns `None` if the array can't be enumerated; an empty `Vec` if
/// the array is empty.
pub(crate) fn collect_paths(items: Option<&IShellItemArray>) -> Option<Vec<OsString>> {
    let array = items?;
    let count = unsafe { array.GetCount() }.ok()?;
    let mut out = Vec::with_capacity(count as usize);
    for i in 0..count {
        let Ok(item): Result<IShellItem> = (unsafe { array.GetItemAt(i) }) else {
            continue;
        };
        if let Ok(pwstr) = unsafe { item.GetDisplayName(SIGDN_FILESYSPATH) } {
            // SAFETY: GetDisplayName returns an OS-allocated PWSTR; we
            // copy its contents and free it via CoTaskMemFree.
            let s = unsafe { read_pwstr(pwstr) };
            out.push(s);
        }
    }
    Some(out)
}

/// SAFETY: `pwstr` must be a valid NUL-terminated UTF-16 string allocated
/// by COM (we free it via CoTaskMemFree before returning).
unsafe fn read_pwstr(pwstr: PWSTR) -> OsString {
    use windows::Win32::System::Com::CoTaskMemFree;
    if pwstr.is_null() {
        return OsString::new();
    }
    // Walk to the NUL terminator. We deliberately avoid PWSTR::to_string
    // because we need OsString to round-trip non-Unicode paths cleanly.
    let mut len = 0usize;
    while unsafe { *pwstr.0.add(len) } != 0 {
        len += 1;
    }
    let slice = unsafe { std::slice::from_raw_parts(pwstr.0, len) };
    let owned = OsString::from_wide(slice);
    unsafe { CoTaskMemFree(Some(pwstr.0 as *const _)) };
    owned
}

/// Locate `xolariq.exe` next to this DLL and spawn it with the selected
/// paths as arguments. Errors are silently swallowed — there is no
/// useful place to surface them from inside Explorer's invoke pipeline,
/// and the most common failure (xolariq.exe missing) is already covered
/// by the installer.
fn spawn_xolariq(paths: &[OsString]) {
    use std::process::Command;

    let Some(exe) = exe_next_to_dll() else {
        return;
    };
    let mut cmd = Command::new(exe);
    for p in paths {
        cmd.arg(p);
    }
    let _ = cmd.spawn();
}

pub(crate) fn exe_next_to_dll() -> Option<std::path::PathBuf> {
    let mut buf = [0u16; 1024];
    let module = crate::exports::current_module();
    // SAFETY: `module` was captured during DllMain; buf is sized.
    let len =
        unsafe { windows::Win32::System::LibraryLoader::GetModuleFileNameW(module, &mut buf) };
    if len == 0 {
        return None;
    }
    let dll_path: std::path::PathBuf = OsString::from_wide(&buf[..len as usize]).into();
    let dir = dll_path.parent()?;
    let exe = dir.join("xolariq.exe");
    if exe.is_file() {
        Some(exe)
    } else {
        None
    }
}

/// Heap-copy a static UTF-16 string into a `PWSTR` that the caller takes
/// ownership of (via `CoTaskMemFree`). Used by every IExplorerCommand
/// title / icon / tooltip getter.
pub(crate) fn copy_to_pwstr(src: PCWSTR) -> Result<PWSTR> {
    use windows::Win32::Foundation::E_OUTOFMEMORY;
    use windows::Win32::System::Com::CoTaskMemAlloc;

    // SAFETY: `src` is a NUL-terminated UTF-16 string by the PCWSTR
    // contract; we count its length and allocate a matching buffer.
    let len = unsafe { src.len() };
    let bytes = (len + 1) * std::mem::size_of::<u16>();
    // SAFETY: `CoTaskMemAlloc` does not require a prior `CoInitializeEx`.
    let raw = unsafe { CoTaskMemAlloc(bytes) } as *mut u16;
    if raw.is_null() {
        return Err(Error::from_hresult(E_OUTOFMEMORY));
    }
    // SAFETY: `raw` was just allocated with capacity for `len + 1` u16s;
    // `src.0` points to at least `len + 1` u16s by PCWSTR contract.
    unsafe {
        std::ptr::copy_nonoverlapping(src.0, raw, len + 1);
    }
    Ok(PWSTR(raw))
}
