use std::ops::BitOr;

use crate::*;

#[repr(u32)]
#[derive(Debug)]
pub enum DataFlow {
    Render = 0,
    Capture = 1,
    All = 2,
}

#[repr(u32)]
#[derive(Debug)]
/// Windows has a feature called 'communication devices', this allows you to set a different input and output devices
/// when using communication software.
/// This feature is essentially useless and isn't used in programs like Discord.
/// I have never seen any mention of 'multimedia devices' in windows.
/// It's also important to note that the *new* Windows 10 settings doesn't have any options to set device roles.
/// My guess it was implemented in Windows 7 and immediatly abandoned, and since Microsoft doesn't change it's API ever,
/// we're stuck writing pointless code.
///
/// https://learn.microsoft.com/en-us/windows/win32/api/mmdeviceapi/ne-mmdeviceapi-erole
/// https://learn.microsoft.com/en-us/windows/win32/coreaudio/device-roles-in-windows-vista
pub enum Role {
    /// Games, system notification sounds, and voice commands.
    Console = 0,
    /// Music, movies, narration, and live music recording.
    Multimedia = 1,
    /// Voice communications (talking to another person).
    Communications = 2,
}

#[repr(u32)]
#[derive(Debug)]
pub enum DeviceState {
    Active = 1,
    Disabled = 2,
    NotPresent = 4,
    Unplugged = 8,
    ///Warning: this will cause rust to panic.
    All = 15,
}

//FIXME:
impl BitOr for DeviceState {
    type Output = u32;

    fn bitor(self, rhs: Self) -> Self::Output {
        unsafe {
            let s: u32 = transmute(self);
            let rhs: u32 = transmute(rhs);
            s | rhs
        }
    }
}

#[repr(u32)]
#[derive(Debug)]
pub enum StorageAccessMode {
    Read = 0,
    Write = 1,
    ReadWrite = 2,
}

//Something to do with propvariants idk. Probably the number in the enum/vtable that is LPWSTR.
pub const VT_LPWSTR: u16 = 31;

#[repr(C)]
#[derive(Debug)]
pub struct IMMDeviceEnumeratorVtbl {
    pub base: IUnknownVtbl,
    pub EnumAudioEndpoints: unsafe extern "system" fn(
        this: *mut IMMDeviceEnumerator,
        data_flow: DataFlow,       //u32
        device_state: DeviceState, //u32
        devices: *mut *mut c_void,
    ) -> i32,
    pub GetDefaultAudioEndpoint: unsafe extern "system" fn(
        this: *mut IMMDeviceEnumerator,
        data_flow: DataFlow, //u32
        role: Role,          //u32
        endpoint: *mut *mut c_void,
    ) -> i32,
    pub GetDevice: unsafe extern "system" fn(
        this: *mut IMMDeviceEnumerator,
        str_id: *const u16,
        device: *mut *mut c_void,
    ) -> i32,
    pub RegisterEndpointNotificationCallback:
        unsafe extern "system" fn(this: *mut IMMDeviceEnumerator, client: *mut c_void) -> i32,
    pub UnregisterEndpointNotificationCallback:
        unsafe extern "system" fn(this: *mut IMMDeviceEnumerator, client: *mut c_void) -> i32,
}

#[repr(transparent)]
#[derive(Debug)]
pub struct IMMDeviceEnumerator(*mut c_void);

unsafe impl Send for IMMDeviceEnumerator {}
unsafe impl Sync for IMMDeviceEnumerator {}

impl IMMDeviceEnumerator {
    ///CLSID_MM_DEVICE_ENUMERATOR
    pub const CLASS_ID: GUID = GUID::from_u128(0xbcde0395_e52f_467c_8e3d_c4579291692e);
    ///IID_IMM_DEVICE_ENUMERATOR
    pub const INTERFACE_ID: GUID = GUID::from_u128(0xa95664d2_9614_4f35_a746_de8db63617e6);

    pub fn new() -> Result<Self, i32> {
        unsafe { CoCreateInstance(&Self::CLASS_ID, ExecutionContext::All, &Self::INTERFACE_ID) }
    }

    #[inline]
    pub unsafe fn vtable(&self) -> (*mut Self, &IMMDeviceEnumeratorVtbl) {
        let this: *mut Self = transmute_copy(self);
        (this, (&**(this as *mut *mut IMMDeviceEnumeratorVtbl)))
    }

    #[inline]
    pub unsafe fn EnumAudioEndpoints(
        &self,
        dataFlow: DataFlow,
        //TODO: Allow for bitflags here.
        device_state: DeviceState,
    ) -> Result<IMMDeviceCollection, i32> {
        let mut devices = core::mem::zeroed();
        let (this, vtable) = self.vtable();
        (vtable.EnumAudioEndpoints)(this, dataFlow, device_state, &mut devices).as_result(devices)
    }

    #[inline]
    pub unsafe fn GetDefaultAudioEndpoint(
        &self,
        dataFlow: DataFlow,
        role: Role,
    ) -> Result<IMMDevice, i32> {
        let mut device = core::mem::zeroed();
        let (this, vtable) = self.vtable();
        (vtable.GetDefaultAudioEndpoint)(this, dataFlow, role, &mut device).as_result(device)
    }

    #[inline]
    pub unsafe fn GetDevice(&self, str_id: *const u16) -> Result<IMMDevice, i32> {
        let mut device = core::mem::zeroed();
        let (this, vtable) = self.vtable();
        (vtable.GetDevice)(this, str_id, &mut device).as_result(device)
    }

    // #[inline]
    // pub unsafe fn RegisterEndpointNotificationCallback(
    //     &self,
    //     pClient: *mut IMMNotificationClient,
    // ) -> i32 {
    //     // ((*self.vtable).RegisterEndpointNotificationCallback)(
    //     //     self as *const _ as *mut _,
    //     //     pClient,
    //     // )
    // }

    // #[inline]
    // pub unsafe fn UnregisterEndpointNotificationCallback(
    //     &self,
    //     pClient: *mut IMMNotificationClient,
    // ) -> i32 {
    //     // ((*self.vtable).UnregisterEndpointNotificationCallback)(
    //     //     self as *const _ as *mut _,
    //     //     pClient,
    //     // )
    // }
}

#[repr(C)]
#[derive(Debug)]
pub struct IMMDeviceVtbl {
    pub parent: IUnknownVtbl,
    pub Activate: unsafe extern "system" fn(
        this: *mut IMMDevice,
        iid: *const GUID,
        execution_context: ExecutionContext,
        activation_params: *mut PROPVARIANT,
        interface: *mut *mut c_void,
    ) -> i32,
    pub OpenPropertyStore: unsafe extern "system" fn(
        this: *mut IMMDevice,
        stgm_access: StorageAccessMode,
        properties: *mut *mut IPropertyStore,
    ) -> i32,
    pub GetId: unsafe extern "system" fn(this: *mut IMMDevice, str_id: *mut *mut u16) -> i32,
    pub GetState: unsafe extern "system" fn(this: *mut IMMDevice, state: *mut u32) -> i32,
}

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IMMDevice(*mut c_void);

unsafe impl Send for IMMDevice {}
unsafe impl Sync for IMMDevice {}

impl IMMDevice {
    //TODO: Return a result.
    pub fn name(&self) -> String {
        unsafe {
            let store = self.OpenPropertyStore(StorageAccessMode::Read).unwrap();
            //TODO: This can fail when using DeviceState::All, code -536870389, not sure how to handle it?
            //https://learn.microsoft.com/en-us/windows/win32/api/propsys/nf-propsys-ipropertystore-getvalue
            let prop = store.GetValue(&PKEY_DEVICE_FRIENDLY_NAME).unwrap();
            assert!(prop.Anonymous.Anonymous.vt.0 == VT_LPWSTR); //TODO: This will never fail remove?
            let data = prop.Anonymous.Anonymous.Anonymous.pwszVal;
            // String::from_utf16(v)
            wide_string(data)
        }
    }

    #[inline]
    pub unsafe fn vtable(&self) -> (*mut Self, &IMMDeviceVtbl) {
        let this: *mut Self = transmute_copy(self);
        (this, (&**(this as *mut *mut IMMDeviceVtbl)))
    }

    #[inline]
    pub unsafe fn Activate<T: Interface>(
        &self,
        execution_context: ExecutionContext,
    ) -> Result<T, i32> {
        let (this, vtable) = self.vtable();
        let mut interface = core::mem::zeroed();

        (vtable.Activate)(
            this,
            &T::id(),
            execution_context,
            //DirectSound is not supported since this is set to null.
            core::mem::zeroed(),
            // core::ptr::null_mut(),
            &mut interface,
        )
        .as_result(interface)
    }

    #[inline]
    pub unsafe fn OpenPropertyStore(
        &self,
        access_mode: StorageAccessMode,
    ) -> Result<IPropertyStore, i32> {
        let mut properties = core::mem::zeroed();
        let (this, vtable) = self.vtable();
        (vtable.OpenPropertyStore)(this, access_mode, &mut properties).as_result(properties)
    }

    // #[inline]
    // pub unsafe fn GetId(&self, ppstrId: *mut LPWSTR) -> i32 {
    //     ((*self.lpVtbl).GetId)(self as *const _ as *mut _, ppstrId)
    // }

    // #[inline]
    // pub unsafe fn GetState(&self, pdwState: *mut u32) -> i32 {
    //     ((*self.lpVtbl).GetState)(self as *const _ as *mut _, pdwState)
    // }
}

#[repr(C)]
pub struct IMMDeviceCollectionVtbl {
    pub parent: IUnknownVtbl,
    pub GetCount:
        unsafe extern "system" fn(this: *mut IMMDeviceCollection, device_count: *const u32) -> i32,
    pub Item: unsafe extern "system" fn(
        this: *mut IMMDeviceCollection,
        device_index: u32,
        device: *mut *mut c_void,
    ) -> i32,
}

#[repr(transparent)]
#[derive(Debug)]
pub struct IMMDeviceCollection(*mut c_void);

unsafe impl Send for IMMDeviceCollection {}
unsafe impl Sync for IMMDeviceCollection {}

impl IMMDeviceCollection {
    #[inline]
    pub unsafe fn vtable(&self) -> (*mut Self, &IMMDeviceCollectionVtbl) {
        let this: *mut Self = transmute_copy(self);
        (this, (&**(this as *mut *mut IMMDeviceCollectionVtbl)))
    }

    #[inline]
    pub unsafe fn GetCount(&self) -> Result<u32, i32> {
        let (this, vtable) = self.vtable();
        let mut device_count = core::mem::zeroed();
        (vtable.GetCount)(this, &mut device_count).as_result_owned(device_count)
    }

    #[inline]
    pub unsafe fn Item(&self, device_index: u32) -> Result<IMMDevice, i32> {
        let (this, vtable) = self.vtable();
        let mut device = core::mem::zeroed();
        (vtable.Item)(this, device_index, &mut device).as_result(device)
    }
}
