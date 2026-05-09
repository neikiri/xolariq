//! [`IClassFactory`] implementation that constructs root commands.
//!
//! Explorer asks `DllGetClassObject` for an `IClassFactory` for our
//! CLSID; this is what it gets back. The single
//! [`IClassFactory::CreateInstance`] entrypoint builds a fresh
//! [`crate::root::RootCommand`] and returns the requested interface
//! pointer to it.

use windows::core::*;
use windows::Win32::Foundation::{BOOL, CLASS_E_NOAGGREGATION, E_NOINTERFACE, E_POINTER, S_OK};
use windows::Win32::System::Com::{IClassFactory, IClassFactory_Impl};

use crate::root::RootCommand;

#[implement(IClassFactory)]
#[derive(Default)]
pub(crate) struct ClassFactory;

impl IClassFactory_Impl for ClassFactory_Impl {
    fn CreateInstance(
        &self,
        punkouter: Option<&IUnknown>,
        riid: *const GUID,
        ppvobject: *mut *mut core::ffi::c_void,
    ) -> Result<()> {
        if ppvobject.is_null() || riid.is_null() {
            return Err(Error::from_hresult(E_POINTER));
        }
        // SAFETY: caller contract — see IClassFactory docs.
        unsafe { *ppvobject = std::ptr::null_mut() };

        if punkouter.is_some() {
            return Err(Error::from_hresult(CLASS_E_NOAGGREGATION));
        }

        let command = RootCommand::default();
        // RootCommand implements IExplorerCommand; QueryInterface picks
        // the right vtable for whichever IID Explorer asked for.
        let iunknown: IUnknown = command.into();
        // SAFETY: `riid` is a valid GUID pointer, `ppvobject` is a valid
        // out pointer; `query` writes a new ref-counted pointer.
        let hr = unsafe { iunknown.query(&*riid, ppvobject) };
        if hr == S_OK {
            Ok(())
        } else {
            Err(Error::from_hresult(E_NOINTERFACE))
        }
    }

    fn LockServer(&self, _flock: BOOL) -> Result<()> {
        // We track lifetime through the live-object counter in
        // `crate::exports::LIVE_OBJECTS`; LockServer is a no-op here.
        Ok(())
    }
}
