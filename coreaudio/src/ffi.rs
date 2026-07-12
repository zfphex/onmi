use core::ffi::c_void;

pub type OSStatus = i32;
pub type AudioObjectID = u32;
pub type Float32 = f32;

pub const fn four_cc(s: &[u8; 4]) -> u32 {
    ((s[0] as u32) << 24) | ((s[1] as u32) << 16) | ((s[2] as u32) << 8) | (s[3] as u32)
}

// CoreAudio Format IDs
pub const K_AUDIO_FORMAT_LINEAR_PCM: u32 = four_cc(b"lpcm");

// Linear PCM Flags
pub const K_AUDIO_FORMAT_FLAG_IS_FLOAT: u32 = 1 << 0;
pub const K_AUDIO_FORMAT_FLAG_IS_BIG_ENDIAN: u32 = 1 << 1;
pub const K_AUDIO_FORMAT_FLAG_IS_PACKED: u32 = 1 << 3;
pub const K_AUDIO_FORMAT_FLAG_IS_NON_INTERLEAVED: u32 = 1 << 5;

// Scopes and Elements
pub const K_AUDIO_OBJECT_PROPERTY_SCOPE_GLOBAL: u32 = four_cc(b"glob");
pub const K_AUDIO_OBJECT_PROPERTY_SCOPE_INPUT: u32 = four_cc(b"inpt");
pub const K_AUDIO_OBJECT_PROPERTY_SCOPE_OUTPUT: u32 = four_cc(b"outp");
pub const K_AUDIO_OBJECT_PROPERTY_ELEMENT_MAIN: u32 = 0; // historically called 'master'

// System Object Properties
pub const K_AUDIO_OBJECT_SYSTEM_OBJECT: AudioObjectID = 1;
pub const K_AUDIO_HARDWARE_PROPERTY_DEVICES: u32 = four_cc(b"dev#");
pub const K_AUDIO_HARDWARE_PROPERTY_DEFAULT_INPUT_DEVICE: u32 = four_cc(b"dIn ");
pub const K_AUDIO_HARDWARE_PROPERTY_DEFAULT_OUTPUT_DEVICE: u32 = four_cc(b"dOut");

// Device Properties
pub const K_AUDIO_DEVICE_PROPERTY_DEVICE_NAME_CFSTRING: u32 = four_cc(b"lnam");
pub const K_AUDIO_DEVICE_PROPERTY_STREAM_CONFIGURATION: u32 = four_cc(b"slay");
pub const K_AUDIO_DEVICE_PROPERTY_NOMINAL_SAMPLE_RATE: u32 = four_cc(b"nsrt");

// Component Constants
pub const K_AUDIO_UNIT_TYPE_OUTPUT: u32 = four_cc(b"auou");
pub const K_AUDIO_UNIT_SUB_TYPE_HAL_OUTPUT: u32 = four_cc(b"ahal");
pub const K_AUDIO_UNIT_MANUFACTURER_APPLE: u32 = four_cc(b"appl");

// AudioUnit Scopes
pub const K_AUDIO_UNIT_SCOPE_GLOBAL: u32 = 0;
pub const K_AUDIO_UNIT_SCOPE_INPUT: u32 = 1;
pub const K_AUDIO_UNIT_SCOPE_OUTPUT: u32 = 2;

// AudioUnit Properties
pub const K_AUDIO_OUTPUT_UNIT_PROPERTY_CURRENT_DEVICE: u32 = 2000;
pub const K_AUDIO_OUTPUT_UNIT_PROPERTY_ENABLE_IO: u32 = 2003;
pub const K_AUDIO_OUTPUT_UNIT_PROPERTY_SET_INPUT_CALLBACK: u32 = 2005;
pub const K_AUDIO_UNIT_PROPERTY_STREAM_FORMAT: u32 = 8;
pub const K_AUDIO_UNIT_PROPERTY_SET_RENDER_CALLBACK: u32 = 23;

#[repr(C)]
pub struct AudioObjectPropertyAddress {
    pub m_selector: u32,
    pub m_scope: u32,
    pub m_element: u32,
}

#[repr(C)]
pub struct AudioStreamBasicDescription {
    pub m_sample_rate: f64,
    pub m_format_id: u32,
    pub m_format_flags: u32,
    pub m_bytes_per_packet: u32,
    pub m_frames_per_packet: u32,
    pub m_bytes_per_frame: u32,
    pub m_channels_per_frame: u32,
    pub m_bits_per_channel: u32,
    pub m_reserved: u32,
}

impl Clone for AudioStreamBasicDescription {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for AudioStreamBasicDescription {}

#[repr(C)]
pub struct AudioBuffer {
    pub m_number_channels: u32,
    pub m_data_byte_size: u32,
    pub m_data: *mut c_void,
}

#[repr(C)]
pub struct AudioBufferList {
    pub m_number_buffers: u32,
    pub m_buffers: [AudioBuffer; 1],
}

#[repr(C)]
pub struct AudioTimeStamp {
    pub m_sample_time: f64,
    pub m_host_time: u64,
    pub m_rate_scalar: f64,
    pub m_word_clock_time: u64,
    pub m_smpte_time: SMPTETime,
    pub m_flags: u32,
    pub m_reserved: u32,
}

#[repr(C)]
pub struct SMPTETime {
    pub m_subframes: i16,
    pub m_subframe_divisor: i16,
    pub m_counter: u32,
    pub m_type: u32,
    pub m_flags: u32,
    pub m_hours: i16,
    pub m_minutes: i16,
    pub m_seconds: i16,
    pub m_frames: i16,
}

#[repr(C)]
pub struct AURenderCallbackStruct {
    pub input_proc: unsafe extern "C" fn(
        *mut c_void,
        *mut u32,
        *const AudioTimeStamp,
        u32,
        u32,
        *mut AudioBufferList,
    ) -> OSStatus,
    pub input_proc_ref_con: *mut c_void,
}

#[repr(C)]
pub struct AudioComponentDescription {
    pub component_type: u32,
    pub component_sub_type: u32,
    pub component_manufacturer: u32,
    pub component_flags: u32,
    pub component_flags_mask: u32,
}

pub type AudioComponent = *mut c_void;
pub type AudioComponentInstance = *mut c_void;
pub type AudioUnit = AudioComponentInstance;

pub type AudioConverterRef = *mut c_void;

#[repr(C)]
pub struct AudioStreamPacketDescription {
    pub m_start_offset: i64,
    pub m_variable_frames_in_packet: u32,
    pub m_data_byte_size: u32,
}

pub type AudioConverterComplexInputDataProc = unsafe extern "C" fn(
    in_audio_converter: AudioConverterRef,
    io_number_data_packets: *mut u32,
    io_data: *mut AudioBufferList,
    out_data_packet_description: *mut *mut AudioStreamPacketDescription,
    in_user_data: *mut c_void,
) -> OSStatus;

#[link(name = "CoreAudio", kind = "framework")]
#[link(name = "AudioUnit", kind = "framework")]
#[link(name = "CoreFoundation", kind = "framework")]
#[link(name = "AudioToolbox", kind = "framework")]
unsafe extern "C" {
    // CoreAudio FFI
    pub fn AudioObjectGetPropertyDataSize(
        in_object_id: AudioObjectID,
        in_address: *const AudioObjectPropertyAddress,
        in_qualifier_data_size: u32,
        in_qualifier_data: *const c_void,
        out_data_size: *mut u32,
    ) -> OSStatus;

    pub fn AudioObjectGetPropertyData(
        in_object_id: AudioObjectID,
        in_address: *const AudioObjectPropertyAddress,
        in_qualifier_data_size: u32,
        in_qualifier_data: *const c_void,
        io_data_size: *mut u32,
        out_data: *mut c_void,
    ) -> OSStatus;

    pub fn AudioObjectSetPropertyData(
        in_object_id: AudioObjectID,
        in_address: *const AudioObjectPropertyAddress,
        in_qualifier_data_size: u32,
        in_qualifier_data: *const c_void,
        in_data_size: u32,
        in_data: *const c_void,
    ) -> OSStatus;

    // AudioUnit / AudioComponent FFI
    pub fn AudioComponentFindNext(
        in_component: AudioComponent,
        in_desc: *const AudioComponentDescription,
    ) -> AudioComponent;

    pub fn AudioComponentInstanceNew(
        in_component: AudioComponent,
        out_instance: *mut AudioComponentInstance,
    ) -> OSStatus;

    pub fn AudioComponentInstanceDispose(in_instance: AudioComponentInstance) -> OSStatus;

    pub fn AudioUnitInitialize(in_unit: AudioUnit) -> OSStatus;

    pub fn AudioUnitSetProperty(
        in_unit: AudioUnit,
        in_id: u32,
        in_scope: u32,
        in_element: u32,
        in_data: *const c_void,
        in_data_size: u32,
    ) -> OSStatus;

    pub fn AudioUnitRender(
        in_unit: AudioUnit,
        io_action_flags: *mut u32,
        in_time_stamp: *const AudioTimeStamp,
        in_output_bus_number: u32,
        in_number_frames: u32,
        io_data: *mut AudioBufferList,
    ) -> OSStatus;

    pub fn AudioOutputUnitStart(in_unit: AudioUnit) -> OSStatus;
    
    pub fn AudioOutputUnitStop(in_unit: AudioUnit) -> OSStatus;

    // AudioConverter FFI
    pub fn AudioConverterNew(
        in_source_format: *const AudioStreamBasicDescription,
        in_destination_format: *const AudioStreamBasicDescription,
        out_audio_converter: *mut AudioConverterRef,
    ) -> OSStatus;

    pub fn AudioConverterDispose(
        in_audio_converter: AudioConverterRef,
    ) -> OSStatus;

    pub fn AudioConverterFillComplexBuffer(
        in_audio_converter: AudioConverterRef,
        in_input_data_proc: AudioConverterComplexInputDataProc,
        in_input_data_proc_user_data: *mut c_void,
        io_output_data_packet_size: *mut u32,
        out_output_data: *mut AudioBufferList,
        out_packet_description: *mut AudioStreamPacketDescription,
    ) -> OSStatus;
}
