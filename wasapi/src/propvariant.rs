use crate::*;
use std::mem::ManuallyDrop;

//Unimplemented vtable.
#[repr(transparent)]
pub struct IStorage(IUnknown);

//Unimplemented vtable.
#[repr(transparent)]
pub struct IStream(IUnknown);

//Unimplemented vtable.
#[repr(transparent)]
pub struct IDispatch(IUnknown);

#[derive(PartialEq, Eq)]
#[repr(transparent)]
pub struct VARENUM(pub u16);

#[repr(C)]
pub struct PROPERTYKEY {
    pub fmtid: GUID,
    pub pid: u32,
}

pub const PKEY_DEVICE_FRIENDLY_NAME: PROPERTYKEY = PROPERTYKEY {
    fmtid: GUID::from_u128(0xa45c254e_df1c_4efd_8020_67d146a850e0),
    pid: 14,
};

#[repr(C)]
pub struct PROPVARIANT {
    pub Anonymous: PROPVARIANT0,
}

#[repr(C)]
pub union PROPVARIANT0 {
    pub Anonymous: std::mem::ManuallyDrop<PROPVARIANT0_0>,
    pub decVal: DECIMAL,
}

#[repr(C)]
pub struct PROPVARIANT0_0 {
    pub vt: VARENUM,
    pub wReserved1: u16,
    pub wReserved2: u16,
    pub wReserved3: u16,
    pub Anonymous: PROPVARIANT_0_0_0,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DECIMAL {
    pub wReserved: u16,
    pub Anonymous1: DECIMAL_0,
    pub Hi32: u32,
    pub Anonymous2: DECIMAL_1,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union DECIMAL_1 {
    pub Anonymous: DECIMAL_1_0,
    pub Lo64: u64,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DECIMAL_1_0 {
    pub Lo32: u32,
    pub Mid32: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union DECIMAL_0 {
    pub Anonymous: DECIMAL_0_0,
    pub signscale: u16,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DECIMAL_0_0 {
    pub scale: u8,
    pub sign: u8,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CY_0 {
    pub Lo: u32,
    pub Hi: i32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union CY {
    pub Anonymous: CY_0,
    pub int64: i64,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FILETIME {
    pub dwLowDateTime: u32,
    pub dwHighDateTime: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CLIPDATA {
    pub cbSize: u32,
    pub ulClipFmt: i32,
    pub pClipData: *mut u8,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct BSTRBLOB {
    pub cbSize: u32,
    pub pData: *mut u8,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct BLOB {
    pub cbSize: u32,
    pub pBlobData: *mut u8,
}

#[repr(C)]
pub struct VERSIONEDSTREAM {
    pub guidVersion: GUID,
    pub pStream: ManuallyDrop<Option<IStream>>,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SAFEARRAY {
    pub cDims: u16,
    pub fFeatures: u16,
    pub cbElements: u32,
    pub cLocks: u32,
    pub pvData: *mut ::core::ffi::c_void,
    pub rgsabound: [SAFEARRAYBOUND; 1],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SAFEARRAYBOUND {
    pub cElements: u32,
    pub lLbound: i32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CAC {
    pub cElems: u32,
    pub pElems: *mut u8,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CAUB {
    pub cElems: u32,
    pub pElems: *mut u8,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CAI {
    pub cElems: u32,
    pub pElems: *mut i16,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CAUI {
    pub cElems: u32,
    pub pElems: *mut u16,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CAL {
    pub cElems: u32,
    pub pElems: *mut i32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CAUL {
    pub cElems: u32,
    pub pElems: *mut u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CAH {
    pub cElems: u32,
    pub pElems: *mut i64,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CAUH {
    pub cElems: u32,
    pub pElems: *mut u64,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CAFLT {
    pub cElems: u32,
    pub pElems: *mut f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CADBL {
    pub cElems: u32,
    pub pElems: *mut f64,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CABOOL {
    pub cElems: u32,
    pub pElems: *mut i16,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CASCODE {
    pub cElems: u32,
    pub pElems: *mut i32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CACY {
    pub cElems: u32,
    pub pElems: *mut CY,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CADATE {
    pub cElems: u32,
    pub pElems: *mut f64,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CAFILETIME {
    pub cElems: u32,
    pub pElems: *mut FILETIME,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CACLSID {
    pub cElems: u32,
    pub pElems: *mut GUID,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CACLIPDATA {
    pub cElems: u32,
    pub pElems: *mut CLIPDATA,
}
//

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CABSTR {
    pub cElems: u32,
    pub pElems: *mut *const u16,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CABSTRBLOB {
    pub cElems: u32,
    pub pElems: *mut BSTRBLOB,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CALPSTR {
    pub cElems: u32,
    pub pElems: *mut *mut u8,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CALPWSTR {
    pub cElems: u32,
    pub pElems: *mut *mut u16,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CAPROPVARIANT {
    pub cElems: u32,
    pub pElems: *mut PROPVARIANT,
}

#[repr(C)]
pub union PROPVARIANT_0_0_0 {
    pub cVal: u8,
    pub bVal: u8,
    pub iVal: i16,
    pub uiVal: u16,
    pub lVal: i32,
    pub ulVal: u32,
    pub intVal: i32,
    pub uintVal: u32,
    pub hVal: i64,
    pub uhVal: u64,
    pub fltVal: f32,
    pub dblVal: f64,
    pub boolVal: i16,
    pub __OBSOLETE__VARIANT_BOOL: i16,
    pub scode: i32,
    pub cyVal: CY,
    pub date: f64,
    pub filetime: FILETIME,
    pub puuid: *mut GUID,
    pub pclipdata: *mut CLIPDATA,
    pub bstrVal: ManuallyDrop<*const u16>,
    pub bstrblobVal: BSTRBLOB,
    pub blob: BLOB,
    pub pszVal: *mut u8,
    pub pwszVal: *mut u16,
    pub punkVal: ManuallyDrop<Option<IUnknown>>,
    pub pdispVal: ManuallyDrop<Option<IDispatch>>,
    pub pStream: ManuallyDrop<Option<IStream>>,
    pub pStorage: ManuallyDrop<Option<IStorage>>,
    pub pVersionedStream: *mut VERSIONEDSTREAM,
    pub parray: *mut SAFEARRAY,
    pub cac: CAC,
    pub caub: CAUB,
    pub cai: CAI,
    pub caui: CAUI,
    pub cal: CAL,
    pub caul: CAUL,
    pub cah: CAH,
    pub cauh: CAUH,
    pub caflt: CAFLT,
    pub cadbl: CADBL,
    pub cabool: CABOOL,
    pub cascode: CASCODE,
    pub cacy: CACY,
    pub cadate: CADATE,
    pub cafiletime: CAFILETIME,
    pub cauuid: CACLSID,
    pub caclipdata: CACLIPDATA,
    pub cabstr: CABSTR,
    pub cabstrblob: CABSTRBLOB,
    pub calpstr: CALPSTR,
    pub calpwstr: CALPWSTR,
    pub capropvar: CAPROPVARIANT,
    pub pcVal: *mut u8,
    pub pbVal: *mut u8,
    pub piVal: *mut i16,
    pub puiVal: *mut u16,
    pub plVal: *mut i32,
    pub pulVal: *mut u32,
    pub pintVal: *mut i32,
    pub puintVal: *mut u32,
    pub pfltVal: *mut f32,
    pub pdblVal: *mut f64,
    pub pboolVal: *mut i16,
    pub pdecVal: *mut DECIMAL,
    pub pscode: *mut i32,
    pub pcyVal: *mut CY,
    pub pdate: *mut f64,
    pub pbstrVal: *mut *const u16,
    pub ppunkVal: *mut Option<IUnknown>,
    pub ppdispVal: *mut Option<IDispatch>,
    pub pparray: *mut *mut SAFEARRAY,
    pub pvarVal: *mut PROPVARIANT,
}

#[repr(C)]
pub struct IPropertyStoreVtbl {
    pub parent: IUnknownVtbl,
    pub GetCount: unsafe extern "system" fn(this: *mut IPropertyStore, cProps: *mut u32) -> i32,
    pub GetAt: unsafe extern "system" fn(
        this: *mut IPropertyStore,
        iProp: u32,
        propkey: *mut PROPERTYKEY,
    ) -> i32,
    pub GetValue: unsafe extern "system" fn(
        this: *mut IPropertyStore,
        key: *const PROPERTYKEY,
        propvar: *mut PROPVARIANT,
    ) -> i32,
    pub SetValue: unsafe extern "system" fn(
        this: *mut IPropertyStore,
        key: *const PROPERTYKEY,
        propvar: *const PROPVARIANT,
    ) -> i32,
    pub Commit: unsafe extern "system" fn(This: *mut IPropertyStore) -> i32,
}

#[repr(transparent)]
#[derive(Debug)]
pub struct IPropertyStore(*mut c_void);

impl IPropertyStore {
    #[inline]
    pub unsafe fn vtable(&self) -> (*mut Self, &IPropertyStoreVtbl) {
        let this: *mut Self = transmute_copy(self);
        (this, (&**(this as *mut *mut IPropertyStoreVtbl)))
    }

    #[inline]
    pub unsafe fn GetCount(&self, cProps: *mut u32) -> Result<(), i32> {
        let (this, vtable) = self.vtable();
        (vtable.GetCount)(this, cProps).as_result_owned(())
    }

    #[inline]
    pub unsafe fn GetAt(&self, iProp: u32) -> Result<PROPERTYKEY, i32> {
        let (this, vtable) = self.vtable();
        let propkey = core::mem::zeroed();
        (vtable.GetAt)(this, iProp, propkey).as_result(propkey)
    }

    #[inline]
    pub unsafe fn GetValue(&self, key: *const PROPERTYKEY) -> Result<PROPVARIANT, i32> {
        let (this, vtable) = self.vtable();
        let mut propvar = core::mem::zeroed();
        (vtable.GetValue)(this, key, &mut propvar).as_result_owned(propvar)
    }

    #[inline]
    pub unsafe fn SetValue(
        &self,
        key: *const PROPERTYKEY,
        propvar: *const PROPVARIANT,
    ) -> Result<(), i32> {
        let (this, vtable) = self.vtable();
        (vtable.SetValue)(this, key, propvar).as_result_owned(())
    }

    #[inline]
    pub unsafe fn Commit(&self) -> i32 {
        let (this, vtable) = self.vtable();
        (vtable.Commit)(this)
    }
}
