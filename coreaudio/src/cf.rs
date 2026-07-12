use std::ffi::c_void;
use std::os::raw::c_char;

pub type CFStringRef = *const c_void;

pub const K_CF_STRING_ENCODING_UTF8: u32 = 0x08000100;

unsafe extern "C" {
    pub fn CFStringGetCStringPtr(theString: CFStringRef, encoding: u32) -> *const c_char;
    pub fn CFStringGetCString(
        theString: CFStringRef,
        buffer: *mut c_char,
        bufferSize: isize,
        encoding: u32,
    ) -> u8;
    pub fn CFStringGetLength(theString: CFStringRef) -> isize;
    pub fn CFStringGetMaximumSizeForEncoding(length: isize, encoding: u32) -> isize;
    pub fn CFRelease(cf: *const c_void);
}

pub fn cfstring_to_string(cf_str: CFStringRef) -> Option<String> {
    if cf_str.is_null() {
        return None;
    }

    unsafe {
        // Try fast path first
        let ptr = CFStringGetCStringPtr(cf_str, K_CF_STRING_ENCODING_UTF8);
        if !ptr.is_null() {
            let c_str = std::ffi::CStr::from_ptr(ptr);
            return Some(c_str.to_string_lossy().into_owned());
        }

        // Slow path: allocate buffer
        let length = CFStringGetLength(cf_str);
        if length == 0 {
            return Some(String::new());
        }

        let max_size = CFStringGetMaximumSizeForEncoding(length, K_CF_STRING_ENCODING_UTF8);
        if max_size <= 0 {
            return None;
        }

        let mut buffer: Vec<u8> = vec![0; (max_size + 1) as usize];
        let success = CFStringGetCString(
            cf_str,
            buffer.as_mut_ptr() as *mut c_char,
            buffer.len() as isize,
            K_CF_STRING_ENCODING_UTF8,
        );

        if success != 0 {
            let c_str = std::ffi::CStr::from_ptr(buffer.as_ptr() as *const c_char);
            Some(c_str.to_string_lossy().into_owned())
        } else {
            None
        }
    }
}
