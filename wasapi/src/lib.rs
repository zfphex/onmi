#![cfg(windows)]
#![allow(non_snake_case, dead_code, non_camel_case_types)]

#[link(name = "ole32")]
#[link(name = "user32")]
extern "system" {}

pub mod audio_client;
pub mod com;
pub mod mmdevice;
pub mod other;
pub mod propvariant;
pub mod unknown;
pub mod waveformat;

pub use audio_client::*;
pub use com::*;
pub use mmdevice::*;
pub use other::*;
pub use propvariant::*;
pub use unknown::*;
pub use waveformat::*;

pub use core::ffi::c_void;
pub use core::mem::{transmute, transmute_copy};

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

pub trait Interface {
    fn id() -> GUID;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct GUID {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

impl GUID {
    pub const fn from_u128(uuid: u128) -> Self {
        Self {
            data1: (uuid >> 96) as u32,
            data2: (uuid >> 80 & 0xffff) as u16,
            data3: (uuid >> 64 & 0xffff) as u16,
            data4: (uuid as u64).to_be_bytes(),
        }
    }
}

//TODO: Change the error type from i32 to something more useful.
pub trait WindowsResult {
    fn as_result<T, P>(self, pointer: *mut P) -> Result<T, i32>;
    fn as_result_owned<T>(self, owned: T) -> Result<T, i32>;
}

impl WindowsResult for i32 {
    #[track_caller]
    fn as_result<T, P>(self, pointer: *mut P) -> Result<T, i32> {
        if self >= 0 {
            unsafe { Ok(transmute_copy(&(pointer as *mut T))) }
        } else {
            Err(self)
        }
    }

    fn as_result_owned<T>(self, owned: T) -> Result<T, i32> {
        if self >= 0 {
            Ok(owned)
        } else {
            Err(self)
        }
    }
}

pub fn wide_string(wide: *mut u16) -> String {
    unsafe {
        assert!(!wide.is_null());

        let slice = std::slice::from_raw_parts(wide, {
            let mut len = 0;
            while *wide.offset(len) != 0 {
                len += 1;
            }
            len as usize
        });

        OsString::from_wide(slice).to_string_lossy().to_string()
    }
}
