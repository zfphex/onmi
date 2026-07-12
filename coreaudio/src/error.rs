use core::fmt;
use crate::ffi::OSStatus;

pub enum CoreAudioError {
    NotRunning,
    Unspecified,
    UnknownProperty,
    BadPropertySize,
    IllegalOperation,
    BadObject,
    BadDevice,
    BadStream,
    UnsupportedOperation,
    UnsupportedFormat,
    Permissions,
    Unknown(OSStatus),
}

impl CoreAudioError {
    pub fn from_os_status(status: OSStatus) -> std::result::Result<(), CoreAudioError> {
        if status == 0 {
            return Ok(());
        }

        // Convert four-character codes to i32
        // 'stop' = 1937010544
        // 'what' = 2003329396
        // 'who?' = 2003332927
        // '!siz' = 561211770
        // 'nope' = 1852797029
        // '!obj' = 560947818
        // '!dev' = 560227702
        // '!str' = 561214578
        // 'unop' = 1970171760
        // '!dat' = 560226420
        // '!prm' = 561017453

        Err(match status {
            1937010544 => CoreAudioError::NotRunning,
            2003329396 => CoreAudioError::Unspecified,
            2003332927 => CoreAudioError::UnknownProperty,
            561211770 => CoreAudioError::BadPropertySize,
            1852797029 => CoreAudioError::IllegalOperation,
            560947818 => CoreAudioError::BadObject,
            560227702 => CoreAudioError::BadDevice,
            561214578 => CoreAudioError::BadStream,
            1970171760 => CoreAudioError::UnsupportedOperation,
            560226420 => CoreAudioError::UnsupportedFormat,
            561017453 => CoreAudioError::Permissions,
            _ => CoreAudioError::Unknown(status),
        })
    }
}

impl fmt::Debug for CoreAudioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreAudioError::NotRunning => {
                write!(f, "CoreAudioError::NotRunning (Hardware not running)")
            }
            CoreAudioError::Unspecified => write!(
                f,
                "CoreAudioError::Unspecified (Unspecified hardware error)"
            ),
            CoreAudioError::UnknownProperty => write!(
                f,
                "CoreAudioError::UnknownProperty (Hardware unknown property)"
            ),
            CoreAudioError::BadPropertySize => {
                write!(f, "CoreAudioError::BadPropertySize (Bad property size)")
            }
            CoreAudioError::IllegalOperation => {
                write!(f, "CoreAudioError::IllegalOperation (Illegal operation)")
            }
            CoreAudioError::BadObject => write!(f, "CoreAudioError::BadObject (Bad audio object)"),
            CoreAudioError::BadDevice => write!(f, "CoreAudioError::BadDevice (Bad audio device)"),
            CoreAudioError::BadStream => write!(f, "CoreAudioError::BadStream (Bad audio stream)"),
            CoreAudioError::UnsupportedOperation => write!(
                f,
                "CoreAudioError::UnsupportedOperation (Unsupported operation)"
            ),
            CoreAudioError::UnsupportedFormat => {
                write!(f, "CoreAudioError::UnsupportedFormat (Unsupported format)")
            }
            CoreAudioError::Permissions => {
                write!(f, "CoreAudioError::Permissions (Permissions error)")
            }
            CoreAudioError::Unknown(code) => {
                write!(f, "CoreAudioError::Unknown(OSStatus: {})", code)
            }
        }
    }
}

impl fmt::Display for CoreAudioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl std::error::Error for CoreAudioError {}

pub type Result<T> = std::result::Result<T, CoreAudioError>;
