use crate::*;

///https://learn.microsoft.com/en-us/windows/win32/api/objbase/ne-objbase-coinit
#[repr(u32)]
#[derive(Debug)]
pub enum ConcurrencyModel {
    ///Initializes the thread for apartment-threaded object concurrency (see Remarks).
    ApartmentThreaded = 0x2,
    ///Initializes the thread for multithreaded object concurrency (see Remarks).
    MultiThreaded = 0x0,
    ///Disables DDE for OLE1 support.
    DisableOLE1DDE = 0x4,
    ///Increase memory usage in an attempt to increase performance.
    SpeedOverMemory = 0x8,
}

///https://learn.microsoft.com/en-us/windows/win32/api/wtypesbase/ne-wtypesbase-clsctx
#[repr(u32)]
#[derive(Debug)]
pub enum ExecutionContext {
    InprocServer = 0x1,
    InprocHandler = 0x2,
    LocalServer = 0x4,
    InprocServer16 = 0x8,
    RemoteServer = 0x10,
    ///CLSCTX_ALL
    All = 0x17,
    InprocHandler16 = 0x20,
    Reserved1 = 0x40,
    Reserved2 = 0x80,
    Reserved3 = 0x100,
    Reserved4 = 0x200,
    NoCodeDownload = 0x400,
    Reserved5 = 0x800,
    NoCustomMarshal = 0x1000,
    EnableCodeDownload = 0x2000,
    NoFailureLog = 0x4000,
    DisableAAA = 0x8000,
    EnableAAA = 0x10000,
    FromDefaultContext = 0x20000,
    Activatex86Server = 0x40000,
    Activate32bitServer,
    Activate64bitServer = 0x80000,
    EnableCloaking = 0x100000,
    Appcontainer = 0x400000,
    ActivateAAAasIU = 0x800000,
    Reserved6 = 0x1000000,
    ActivateArm32Server = 0x2000000,
    // ALLOW_LOWER_TRUST_REGISTRATION = ?,
    PsDll = 0x80000000,
}

pub unsafe fn CoInitializeEx(model: ConcurrencyModel) -> Result<(), i32> {
    extern "system" {
        fn CoInitializeEx(pvReserved: *mut std::ffi::c_void, dwCoInit: u32) -> i32;
    }
    CoInitializeEx(core::mem::zeroed(), transmute(model)).as_result_owned(())
}

pub unsafe fn CoCreateInstance<T>(
    class_id: *const GUID,
    context: ExecutionContext,
    interface_id: *const GUID,
) -> Result<T, i32> {
    extern "system" {
        pub fn CoCreateInstance(
            rclsid: *const GUID,
            pUnkOuter: *mut *mut c_void,
            dwClsContext: u32,
            riid: *const GUID,
            ppv: *mut *mut c_void,
        ) -> i32;
    }
    let mut instance = core::mem::zeroed();
    CoCreateInstance(
        class_id,
        core::mem::zeroed(), //Note that this could be null_mut(), I'm not sure which I should use...
        transmute(context),
        interface_id,
        &mut instance,
    )
    .as_result(instance)
}
