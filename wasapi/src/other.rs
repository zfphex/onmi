use std::ffi::c_void;

#[repr(C)]
pub struct SecurityAttributes {
    pub nLength: u32,
    pub lpSecurityDescriptor: *mut c_void,
    pub bInheritHandle: i32,
}

#[link(name = "Avrt")]
unsafe extern "system" {
    pub fn CreateEventA(
        lpEventAttributes: *mut SecurityAttributes,
        bManualReset: i32,
        bInitialState: i32,
        lpName: *const i8,
    ) -> *mut c_void;
    pub fn WaitForSingleObject(hHandle: *mut c_void, dwMilliseconds: u32) -> u32;
    pub fn AvSetMmThreadCharacteristicsA(TaskName: *const i8, TaskIndex: *mut u32) -> *mut c_void;
    pub fn GetLastError() -> u32;
}

pub const ERROR_INVALID_TASK_NAME: u32 = 1550;
pub const ERROR_INVALID_TASK_INDEX: u32 = 1551;
pub const ERROR_PRIVILEGE_NOT_HELD: u32 = 1314;

pub fn set_pro_audio_thread() -> u32 {
    unsafe {
        let mut task_index = 0;
        if AvSetMmThreadCharacteristicsA("Pro Audio\0".as_ptr() as *const i8, &mut task_index)
            .is_null()
        {
            match GetLastError() {
                ERROR_INVALID_TASK_NAME => panic!("Invalid task name"),
                ERROR_INVALID_TASK_INDEX => panic!("Invalid task index"),
                ERROR_PRIVILEGE_NOT_HELD => panic!("Required privilege not held"),
                _ => unreachable!(),
            }
        }
        task_index
    }
}

pub const WAIT_OBJECT_0: u32 = 0;
pub const WAIT_TIMEOUT: u32 = 258;
