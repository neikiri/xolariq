//! Self-registration helpers driven by `DllRegisterServer` /
//! `DllUnregisterServer` exports.
//!
//! Layout written under `HKEY_CURRENT_USER` (per-user install — no admin
//! prompt; same trade-off as the `xolariq-shell` crate):
//!
//! ```text
//! HKCU\Software\Classes\CLSID\{a4f1d8e2-...}
//!     (Default) = "Xolariq Shell Extension"
//!     HKCU\...\InprocServer32
//!         (Default) = full path to xolariq_shellext.dll
//!         ThreadingModel = "Apartment"
//!
//! HKCU\Software\Classes\*\shell\XolariqRoot
//!     ExplorerCommandHandler = "{a4f1d8e2-...}"
//!     MUIVerb = "Convert with Xolariq"
//!     Icon = <path to xolariq.exe>     (optional, looked up later)
//! ```
//!
//! The wildcard (`*`) class is used for the same reason as in
//! `xolariq-shell`: Windows 11 25H2 only honours classic verbs that live
//! under `*\shell`, not under `.<ext>\shell`.
//!
//! Note that this registration alone makes the verb appear in
//! *"Show more options"*. Showing in the modern Windows 11 default
//! context menu additionally requires a sparse MSIX package referencing
//! this CLSID via `<desktop4:FileExplorerContextMenus>`. That MSIX
//! manifest is shipped separately by the Wix bundle.

use std::path::Path;

use windows::core::*;
use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS, HMODULE};
use windows::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteTreeW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
    KEY_WRITE, REG_OPTION_NON_VOLATILE, REG_SZ,
};

use crate::CLSID_XOLARIQ_ROOT_COMMAND;
use crate::SHELL_EXT_FRIENDLY_NAME;

const ROOT_VERB_NAME: &str = "XolariqRoot";

/// Path of the currently loaded DLL — needed so `InprocServer32` points
/// at the actual file on disk, not whatever path the bundler thinks it
/// is. Called from `DllRegisterServer` while we're already inside our
/// own DLL, so `GetModuleFileNameW(None, ...)` is wrong; we always pass
/// our HMODULE captured during `DllMain`.
pub(crate) fn dll_path(module: HMODULE) -> Result<String> {
    let mut buf = [0u16; 1024];
    // SAFETY: GetModuleFileNameW writes up to `buf.len()` UTF-16 code units
    // into our buffer and returns the count it wrote (excluding NUL).
    let len = unsafe { GetModuleFileNameW(module, &mut buf) };
    if len == 0 || len as usize >= buf.len() {
        return Err(Error::from_win32());
    }
    Ok(String::from_utf16_lossy(&buf[..len as usize]))
}

/// Write all keys / values needed to register the COM server and the
/// classic root verb that delegates to it.
pub(crate) fn register(module: HMODULE) -> Result<()> {
    let dll = dll_path(module)?;
    let clsid_str = format!("{{{:?}}}", CLSID_XOLARIQ_ROOT_COMMAND);
    let clsid_str = clsid_str.to_uppercase();

    // 1. CLSID
    {
        let path = format!("Software\\Classes\\CLSID\\{}", clsid_str);
        let key = create_subkey(HKEY_CURRENT_USER, &path)?;
        set_string(key, None, SHELL_EXT_FRIENDLY_NAME)?;
        close(key);
    }
    // 2. CLSID\InprocServer32
    {
        let path = format!("Software\\Classes\\CLSID\\{}\\InprocServer32", clsid_str);
        let key = create_subkey(HKEY_CURRENT_USER, &path)?;
        set_string(key, None, &dll)?;
        set_string(key, Some("ThreadingModel"), "Apartment")?;
        close(key);
    }
    // 3. *\shell\XolariqRoot — the verb pointing at our handler.
    {
        let path = format!("Software\\Classes\\*\\shell\\{}", ROOT_VERB_NAME);
        let key = create_subkey(HKEY_CURRENT_USER, &path)?;
        set_string(key, Some("MUIVerb"), "Convert with Xolariq")?;
        set_string(key, Some("ExplorerCommandHandler"), &clsid_str)?;
        close(key);
    }

    Ok(())
}

/// Inverse of [`register`].
pub(crate) fn unregister() -> Result<()> {
    let clsid_str = format!("{{{:?}}}", CLSID_XOLARIQ_ROOT_COMMAND).to_uppercase();
    let _ = delete_tree(
        HKEY_CURRENT_USER,
        &format!("Software\\Classes\\CLSID\\{}", clsid_str),
    );
    let _ = delete_tree(
        HKEY_CURRENT_USER,
        &format!("Software\\Classes\\*\\shell\\{}", ROOT_VERB_NAME),
    );
    Ok(())
}

fn create_subkey(parent: HKEY, path: &str) -> Result<HKEY> {
    let mut handle = HKEY::default();
    let wide: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
    // SAFETY: path is NUL-terminated; we receive ownership of the handle
    // and close it via `close` once writes are done.
    let status = unsafe {
        RegCreateKeyExW(
            parent,
            PCWSTR(wide.as_ptr()),
            0,
            PCWSTR::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            None,
            &mut handle,
            None,
        )
    };
    if status != ERROR_SUCCESS {
        return Err(Error::from_hresult(HRESULT::from_win32(status.0)));
    }
    Ok(handle)
}

fn set_string(key: HKEY, name: Option<&str>, value: &str) -> Result<()> {
    let name_wide: Option<Vec<u16>> = name.map(|n| n.encode_utf16().chain(Some(0)).collect());
    let value_wide: Vec<u16> = value.encode_utf16().chain(Some(0)).collect();
    let bytes = unsafe {
        std::slice::from_raw_parts(
            value_wide.as_ptr() as *const u8,
            value_wide.len() * std::mem::size_of::<u16>(),
        )
    };
    let name_pcwstr = match name_wide.as_deref() {
        Some(n) => PCWSTR(n.as_ptr()),
        None => PCWSTR::null(),
    };
    // SAFETY: `bytes` lives until the end of this function call; the
    // registry API copies the data internally.
    let status = unsafe { RegSetValueExW(key, name_pcwstr, 0, REG_SZ, Some(bytes)) };
    if status != ERROR_SUCCESS {
        return Err(Error::from_hresult(HRESULT::from_win32(status.0)));
    }
    Ok(())
}

fn delete_tree(parent: HKEY, path: &str) -> Result<()> {
    let wide: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
    // SAFETY: `wide` is a NUL-terminated UTF-16 string.
    let status = unsafe { RegDeleteTreeW(parent, PCWSTR(wide.as_ptr())) };
    if status == ERROR_SUCCESS || status == ERROR_FILE_NOT_FOUND {
        Ok(())
    } else {
        Err(Error::from_hresult(HRESULT::from_win32(status.0)))
    }
}

fn close(key: HKEY) {
    // SAFETY: Always called on a key produced by `create_subkey`. We
    // ignore the return value because we have no recovery path for a
    // close failure.
    let _ = unsafe { RegCloseKey(key) };
}

/// Returns the path used by the parent verb's icon. The icon resolves
/// to the installed `xolariq.exe`, which is colocated with the DLL.
#[allow(dead_code)]
pub(crate) fn xolariq_exe_next_to(dll_path: &str) -> Option<String> {
    let dll = Path::new(dll_path);
    let dir = dll.parent()?;
    let candidate = dir.join("xolariq.exe");
    if candidate.is_file() {
        candidate.to_str().map(str::to_owned)
    } else {
        None
    }
}
