//! DLL entrypoints exported to the OS / Explorer.
//!
//! The four `Dll*` functions below are the COM in-process server contract:
//!
//! * [`DllMain`] — captures our HMODULE on `DLL_PROCESS_ATTACH` so that
//!   self-registration can resolve its own filesystem path.
//! * [`DllGetClassObject`] — Explorer asks for an [`IClassFactory`] for a
//!   given CLSID; we return one for our root command.
//! * [`DllCanUnloadNow`] — returns S_OK once the live-object count drops
//!   to zero so Explorer can free us.
//! * [`DllRegisterServer`] / [`DllUnregisterServer`] — what `regsvr32`
//!   calls. Routes to [`crate::registration`].
//!
//! All exports are `extern "system"` and `#[no_mangle]`; the linker also
//! needs the matching `xolariq_shellext.def` so they're exported by the
//! exact uppercase names Windows looks up.

use std::sync::atomic::{AtomicI32, AtomicIsize, Ordering};

use windows::core::*;
use windows::Win32::Foundation::{
    BOOL, CLASS_E_CLASSNOTAVAILABLE, E_POINTER, HMODULE, S_FALSE, S_OK,
};
use windows::Win32::System::Com::IClassFactory;
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;

use crate::factory::ClassFactory;
use crate::registration;
use crate::CLSID_XOLARIQ_ROOT_COMMAND;

/// Number of live COM objects served from this DLL. `DllCanUnloadNow`
/// reports `S_OK` only when this hits zero.
pub(crate) static LIVE_OBJECTS: AtomicI32 = AtomicI32::new(0);

/// HMODULE of *our* DLL, captured during `DllMain(DLL_PROCESS_ATTACH)`.
/// Stored as `isize` so it can live in an atomic without unsafe `Sync`
/// shenanigans.
static DLL_MODULE: AtomicIsize = AtomicIsize::new(0);

/// Standard COM server `DllMain`. Stores the HMODULE we receive so
/// later self-registration can call `GetModuleFileName(hmodule, ...)`.
#[no_mangle]
pub extern "system" fn DllMain(hmodule: HMODULE, reason: u32, _reserved: *mut ()) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        DLL_MODULE.store(hmodule.0 as isize, Ordering::Release);
    }
    BOOL::from(true)
}

pub(crate) fn current_module() -> HMODULE {
    HMODULE(DLL_MODULE.load(Ordering::Acquire) as *mut _)
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut std::ffi::c_void,
) -> HRESULT {
    if rclsid.is_null() || riid.is_null() || ppv.is_null() {
        return E_POINTER;
    }
    // SAFETY: caller contract — pointers must be readable for one GUID.
    let clsid = unsafe { *rclsid };
    let iid = unsafe { *riid };
    if clsid != CLSID_XOLARIQ_ROOT_COMMAND {
        return CLASS_E_CLASSNOTAVAILABLE;
    }
    let factory: IClassFactory = ClassFactory::default().into();
    // SAFETY: `factory` holds the only outstanding ref, `query` writes a
    // new ref into `*ppv` and we transfer ownership to the caller.
    unsafe { factory.query(&iid, ppv) }
}

#[no_mangle]
pub extern "system" fn DllCanUnloadNow() -> HRESULT {
    if LIVE_OBJECTS.load(Ordering::Acquire) == 0 {
        S_OK
    } else {
        S_FALSE
    }
}

#[no_mangle]
pub extern "system" fn DllRegisterServer() -> HRESULT {
    match registration::register(current_module()) {
        Ok(()) => S_OK,
        Err(err) => err.code(),
    }
}

#[no_mangle]
pub extern "system" fn DllUnregisterServer() -> HRESULT {
    match registration::unregister() {
        Ok(()) => S_OK,
        Err(err) => err.code(),
    }
}
