//! Sub-command items rendered in the flyout when the user clicks
//! "Convert with Xolariq" in the Explorer context menu.
//!
//! Each format target gets its own [`SubCommand`] (e.g. "To MP3"),
//! followed by a [`SeparatorCommand`] and a [`SettingsCommand`] that
//! opens the Xolariq Settings window.

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::sync::atomic::Ordering;

use windows::core::*;
use windows::Win32::Foundation::BOOL;
use windows::Win32::UI::Shell::{
    IEnumExplorerCommand, IEnumExplorerCommand_Impl, IExplorerCommand, IExplorerCommand_Impl,
    IShellItemArray, ECF_ISSEPARATOR, ECS_ENABLED,
};

use crate::exports::LIVE_OBJECTS;
use crate::formats::Format;
use crate::root::copy_to_pwstr;

// ---------------------------------------------------------------------------
// SubCommand — one per target format (e.g. "To MP3")
// ---------------------------------------------------------------------------

#[implement(IExplorerCommand)]
pub(crate) struct SubCommand {
    format: Format,
}

impl SubCommand {
    pub fn new(format: Format) -> Self {
        LIVE_OBJECTS.fetch_add(1, Ordering::AcqRel);
        Self { format }
    }
}

impl Drop for SubCommand {
    fn drop(&mut self) {
        LIVE_OBJECTS.fetch_sub(1, Ordering::AcqRel);
    }
}

impl IExplorerCommand_Impl for SubCommand_Impl {
    fn GetTitle(&self, _items: Option<&IShellItemArray>) -> Result<PWSTR> {
        let title: Vec<u16> = format!("To {}", self.format.label)
            .encode_utf16()
            .chain(Some(0))
            .collect();
        alloc_pwstr(&title)
    }

    fn GetIcon(&self, _items: Option<&IShellItemArray>) -> Result<PWSTR> {
        Err(Error::from_hresult(windows::Win32::Foundation::E_NOTIMPL))
    }

    fn GetToolTip(&self, _items: Option<&IShellItemArray>) -> Result<PWSTR> {
        Err(Error::from_hresult(windows::Win32::Foundation::E_NOTIMPL))
    }

    fn GetCanonicalName(&self) -> Result<GUID> {
        Err(Error::from_hresult(windows::Win32::Foundation::E_NOTIMPL))
    }

    fn GetState(&self, _items: Option<&IShellItemArray>, _ok_to_be_slow: BOOL) -> Result<u32> {
        Ok(ECS_ENABLED.0 as u32)
    }

    fn Invoke(
        &self,
        items: Option<&IShellItemArray>,
        _bind_ctx: Option<&windows::Win32::System::Com::IBindCtx>,
    ) -> Result<()> {
        let paths = crate::root::collect_paths(items).unwrap_or_default();
        if paths.is_empty() {
            return Ok(());
        }
        spawn_xolariq_with_target(&paths, self.format.ext);
        Ok(())
    }

    fn GetFlags(&self) -> Result<u32> {
        Ok(0)
    }

    fn EnumSubCommands(&self) -> Result<IEnumExplorerCommand> {
        Err(Error::from_hresult(windows::Win32::Foundation::E_NOTIMPL))
    }
}

// ---------------------------------------------------------------------------
// SeparatorCommand — visual divider line before "Settings"
// ---------------------------------------------------------------------------

#[implement(IExplorerCommand)]
pub(crate) struct SeparatorCommand;

impl SeparatorCommand {
    pub fn new() -> Self {
        LIVE_OBJECTS.fetch_add(1, Ordering::AcqRel);
        Self
    }
}

impl Drop for SeparatorCommand {
    fn drop(&mut self) {
        LIVE_OBJECTS.fetch_sub(1, Ordering::AcqRel);
    }
}

impl IExplorerCommand_Impl for SeparatorCommand_Impl {
    fn GetTitle(&self, _items: Option<&IShellItemArray>) -> Result<PWSTR> {
        Err(Error::from_hresult(windows::Win32::Foundation::E_NOTIMPL))
    }

    fn GetIcon(&self, _items: Option<&IShellItemArray>) -> Result<PWSTR> {
        Err(Error::from_hresult(windows::Win32::Foundation::E_NOTIMPL))
    }

    fn GetToolTip(&self, _items: Option<&IShellItemArray>) -> Result<PWSTR> {
        Err(Error::from_hresult(windows::Win32::Foundation::E_NOTIMPL))
    }

    fn GetCanonicalName(&self) -> Result<GUID> {
        Err(Error::from_hresult(windows::Win32::Foundation::E_NOTIMPL))
    }

    fn GetState(&self, _items: Option<&IShellItemArray>, _ok_to_be_slow: BOOL) -> Result<u32> {
        Ok(ECS_ENABLED.0 as u32)
    }

    fn Invoke(
        &self,
        _items: Option<&IShellItemArray>,
        _bind_ctx: Option<&windows::Win32::System::Com::IBindCtx>,
    ) -> Result<()> {
        Ok(())
    }

    fn GetFlags(&self) -> Result<u32> {
        Ok(ECF_ISSEPARATOR.0 as u32)
    }

    fn EnumSubCommands(&self) -> Result<IEnumExplorerCommand> {
        Err(Error::from_hresult(windows::Win32::Foundation::E_NOTIMPL))
    }
}

// ---------------------------------------------------------------------------
// SettingsCommand — opens the Xolariq Settings window
// ---------------------------------------------------------------------------

/// Title shown for the settings item.
const SETTINGS_TITLE: PCWSTR = w!("Settings");

#[implement(IExplorerCommand)]
pub(crate) struct SettingsCommand;

impl SettingsCommand {
    pub fn new() -> Self {
        LIVE_OBJECTS.fetch_add(1, Ordering::AcqRel);
        Self
    }
}

impl Drop for SettingsCommand {
    fn drop(&mut self) {
        LIVE_OBJECTS.fetch_sub(1, Ordering::AcqRel);
    }
}

impl IExplorerCommand_Impl for SettingsCommand_Impl {
    fn GetTitle(&self, _items: Option<&IShellItemArray>) -> Result<PWSTR> {
        copy_to_pwstr(SETTINGS_TITLE)
    }

    fn GetIcon(&self, _items: Option<&IShellItemArray>) -> Result<PWSTR> {
        // Look for settings.ico next to the DLL.
        if let Some(path) = settings_ico_path() {
            let wide: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
            return alloc_pwstr(&wide);
        }
        Err(Error::from_hresult(windows::Win32::Foundation::E_NOTIMPL))
    }

    fn GetToolTip(&self, _items: Option<&IShellItemArray>) -> Result<PWSTR> {
        copy_to_pwstr(w!("Open Xolariq settings"))
    }

    fn GetCanonicalName(&self) -> Result<GUID> {
        Err(Error::from_hresult(windows::Win32::Foundation::E_NOTIMPL))
    }

    fn GetState(&self, _items: Option<&IShellItemArray>, _ok_to_be_slow: BOOL) -> Result<u32> {
        Ok(ECS_ENABLED.0 as u32)
    }

    fn Invoke(
        &self,
        _items: Option<&IShellItemArray>,
        _bind_ctx: Option<&windows::Win32::System::Com::IBindCtx>,
    ) -> Result<()> {
        spawn_xolariq_settings();
        Ok(())
    }

    fn GetFlags(&self) -> Result<u32> {
        Ok(0)
    }

    fn EnumSubCommands(&self) -> Result<IEnumExplorerCommand> {
        Err(Error::from_hresult(windows::Win32::Foundation::E_NOTIMPL))
    }
}

// ---------------------------------------------------------------------------
// SubCommandEnum — IEnumExplorerCommand over a Vec<IExplorerCommand>
// ---------------------------------------------------------------------------

#[implement(IEnumExplorerCommand)]
pub(crate) struct SubCommandEnum {
    items: Vec<IExplorerCommand>,
    index: std::sync::atomic::AtomicUsize,
}

impl SubCommandEnum {
    /// Build the flyout items for the given source file extension.
    /// Returns all matching target formats + separator + Settings.
    pub fn for_extension(ext: &str) -> Self {
        let mut items: Vec<IExplorerCommand> = Vec::new();

        if let Some(source) = crate::formats::from_extension(ext) {
            for target in crate::formats::targets_for(source) {
                let cmd: IExplorerCommand = SubCommand::new(target).into();
                items.push(cmd);
            }
        }

        // If there were target items, add a separator before Settings.
        if !items.is_empty() {
            let sep: IExplorerCommand = SeparatorCommand::new().into();
            items.push(sep);
        }

        let settings: IExplorerCommand = SettingsCommand::new().into();
        items.push(settings);

        LIVE_OBJECTS.fetch_add(1, Ordering::AcqRel);
        Self {
            items,
            index: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Build the flyout showing all formats (when extension is unknown or
    /// we want to show everything).
    pub fn all_formats() -> Self {
        let mut items: Vec<IExplorerCommand> = Vec::new();

        for &fmt in crate::formats::ALL {
            let cmd: IExplorerCommand = SubCommand::new(fmt).into();
            items.push(cmd);
        }

        if !items.is_empty() {
            let sep: IExplorerCommand = SeparatorCommand::new().into();
            items.push(sep);
        }

        let settings: IExplorerCommand = SettingsCommand::new().into();
        items.push(settings);

        LIVE_OBJECTS.fetch_add(1, Ordering::AcqRel);
        Self {
            items,
            index: std::sync::atomic::AtomicUsize::new(0),
        }
    }
}

impl Drop for SubCommandEnum {
    fn drop(&mut self) {
        LIVE_OBJECTS.fetch_sub(1, Ordering::AcqRel);
    }
}

impl IEnumExplorerCommand_Impl for SubCommandEnum_Impl {
    fn Next(
        &self,
        celt: u32,
        pUICommand: *mut Option<IExplorerCommand>,
        pceltFetched: *mut u32,
    ) -> HRESULT {
        let idx = self.index.load(Ordering::Acquire);
        let remaining = self.items.len().saturating_sub(idx);
        let to_fetch = std::cmp::min(celt as usize, remaining);

        for i in 0..to_fetch {
            // SAFETY: pUICommand points to caller-owned array of at least
            // `celt` Option<IExplorerCommand> slots.
            unsafe {
                *pUICommand.add(i) = Some(self.items[idx + i].clone());
            }
        }

        self.index.store(idx + to_fetch, Ordering::Release);

        if !pceltFetched.is_null() {
            // SAFETY: caller guarantees pceltFetched is valid when non-null.
            unsafe { *pceltFetched = to_fetch as u32 };
        }

        if to_fetch == celt as usize {
            windows::Win32::Foundation::S_OK
        } else {
            windows::Win32::Foundation::S_FALSE
        }
    }

    fn Clone(&self) -> Result<IEnumExplorerCommand> {
        Err(Error::from_hresult(windows::Win32::Foundation::E_NOTIMPL))
    }

    fn Reset(&self) -> Result<()> {
        self.index.store(0, Ordering::Release);
        Ok(())
    }

    fn Skip(&self, celt: u32) -> Result<()> {
        let _ = self.index.fetch_add(celt as usize, Ordering::AcqRel);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Heap-allocate a COM-owned PWSTR from a NUL-terminated UTF-16 slice.
fn alloc_pwstr(wide: &[u16]) -> Result<PWSTR> {
    use windows::Win32::Foundation::E_OUTOFMEMORY;
    use windows::Win32::System::Com::CoTaskMemAlloc;

    let bytes = wide.len() * std::mem::size_of::<u16>();
    let raw = unsafe { CoTaskMemAlloc(bytes) } as *mut u16;
    if raw.is_null() {
        return Err(Error::from_hresult(E_OUTOFMEMORY));
    }
    unsafe {
        std::ptr::copy_nonoverlapping(wide.as_ptr(), raw, wide.len());
    }
    Ok(PWSTR(raw))
}

/// Locate `xolariq.exe` next to this DLL and spawn it with `--target`
/// and `--input` arguments.
fn spawn_xolariq_with_target(paths: &[OsString], target_ext: &str) {
    use std::process::Command;

    let Some(exe) = crate::root::exe_next_to_dll() else {
        return;
    };
    let mut cmd = Command::new(exe);
    cmd.arg("--target").arg(target_ext);
    for p in paths {
        cmd.arg("--input").arg(p);
    }
    let _ = cmd.spawn();
}

/// Spawn `xolariq.exe` with no input files — the app opens the Settings
/// window when launched without conversion arguments.
fn spawn_xolariq_settings() {
    use std::process::Command;

    let Some(exe) = crate::root::exe_next_to_dll() else {
        return;
    };
    let _ = Command::new(exe).spawn();
}

/// Return the absolute path to `settings.ico` next to this DLL, if it exists.
fn settings_ico_path() -> Option<String> {
    let module = crate::exports::current_module();
    let mut buf = [0u16; 1024];
    let len =
        unsafe { windows::Win32::System::LibraryLoader::GetModuleFileNameW(module, &mut buf) };
    if len == 0 {
        return None;
    }
    let dll_path: std::path::PathBuf = OsString::from_wide(&buf[..len as usize]).into();
    let dir = dll_path.parent()?;
    let candidate = dir.join("settings.ico");
    if candidate.is_file() {
        candidate.to_str().map(str::to_owned)
    } else {
        None
    }
}
