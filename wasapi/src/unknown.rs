use crate::*;

/// All COM interfaces (and thus WinRT classes and interfaces) implement
/// [IUnknown](https://docs.microsoft.com/en-us/windows/win32/api/unknwn/nn-unknwn-iunknown)
/// under the hood to provide reference-counted lifetime management as well as the ability
/// to query for additional interfaces that the object may implement.
#[repr(transparent)]
pub struct IUnknown(std::ptr::NonNull<std::ffi::c_void>);

#[repr(C)]
#[derive(Debug)]
pub struct IUnknownVtbl {
    pub QueryInterface: unsafe extern "system" fn(
        this: *mut IUnknown,
        iid: &GUID,
        interface: *mut *const std::ffi::c_void,
    ) -> i32,
    pub AddRef: unsafe extern "system" fn(this: *mut IUnknown) -> u32,
    pub Release: unsafe extern "system" fn(this: *mut IUnknown) -> u32,
}

impl IUnknown {
    #[inline]
    pub unsafe fn vtable(&self) -> (*mut Self, &IUnknownVtbl) {
        let this: *mut Self = transmute_copy(self);
        (this, (&**(this as *mut *mut IUnknownVtbl)))
    }

    const INTERFACE_ID: GUID = GUID::from_u128(0x00000000_0000_0000_c000_000000000046);
}

impl Clone for IUnknown {
    fn clone(&self) -> Self {
        unsafe {
            let (this, vtable) = self.vtable();
            (vtable.AddRef)(this);
        }

        Self(self.0)
    }
}

impl Drop for IUnknown {
    fn drop(&mut self) {
        unsafe {
            let (this, vtable) = self.vtable();
            (vtable.Release)(this);
        }
    }
}
