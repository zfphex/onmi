use crate::*;

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct WAVEFORMATEX {
    pub wFormatTag: u16,
    pub nChannels: u16,
    pub nSamplesPerSec: u32,
    pub nAvgBytesPerSec: u32,
    pub nBlockAlign: u16,
    pub wBitsPerSample: u16,
    pub cbSize: u16,
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct WAVEFORMATEXTENSIBLE {
    pub Format: WAVEFORMATEX,
    pub Samples: u16,
    pub dwChannelMask: u32,
    pub SubFormat: GUID,
}
