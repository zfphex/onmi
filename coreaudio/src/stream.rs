use crate::device::AudioDevice;
use crate::error::{CoreAudioError, Result};
use crate::ffi::*;
use std::ffi::c_void;

pub type RenderCallback =
    extern "C" fn(context: *mut c_void, buffer_ptr: *mut f32, total_samples: usize);

#[repr(C)]
struct InternalContext {
    pub user_callback: RenderCallback,
    pub user_context: *mut c_void,
    pub audio_unit: AudioUnit,
    pub channels: u32,
}

unsafe extern "C" fn core_audio_trampoline(
    in_ref_con: *mut c_void,
    _io_action_flags: *mut u32,
    _in_time_stamp: *const AudioTimeStamp,
    _in_bus_number: u32,
    in_number_frames: u32,
    io_data: *mut AudioBufferList,
) -> OSStatus {
    unsafe {
        let internal_ctx = &*(in_ref_con as *const InternalContext);

        let buffer_list = &mut *io_data;
        if buffer_list.m_number_buffers > 0 {
            let buffer_ptr = buffer_list.m_buffers[0].m_data as *mut f32;
            let channels = buffer_list.m_buffers[0].m_number_channels;
            let total_samples = (in_number_frames * channels) as usize;

            (internal_ctx.user_callback)(internal_ctx.user_context, buffer_ptr, total_samples);
        }
    }

    0
}

unsafe extern "C" fn core_audio_input_trampoline(
    in_ref_con: *mut c_void,
    io_action_flags: *mut u32,
    in_time_stamp: *const AudioTimeStamp,
    in_bus_number: u32,
    in_number_frames: u32,
    _io_data: *mut AudioBufferList,
) -> OSStatus {
    unsafe {
        let internal_ctx = &*(in_ref_con as *const InternalContext);

        let channels = internal_ctx.channels;
        let bytes_per_frame = channels * 4;
        let mut data = vec![0u8; (in_number_frames * bytes_per_frame) as usize];

        let mut buffer_list = AudioBufferList {
            m_number_buffers: 1,
            m_buffers: [AudioBuffer {
                m_number_channels: channels,
                m_data_byte_size: (in_number_frames * bytes_per_frame) as u32,
                m_data: data.as_mut_ptr() as *mut c_void,
            }],
        };

        let status = AudioUnitRender(
            internal_ctx.audio_unit,
            io_action_flags,
            in_time_stamp,
            in_bus_number,
            in_number_frames,
            &mut buffer_list,
        );

        if status == 0 {
            let buffer_ptr = buffer_list.m_buffers[0].m_data as *mut f32;
            let total_samples = (in_number_frames * channels) as usize;

            (internal_ctx.user_callback)(internal_ctx.user_context, buffer_ptr, total_samples);
        }

        status
    }
}

pub struct AudioStream {
    audio_unit: AudioUnit,
    _context_box: *mut InternalContext,
}

impl AudioStream {
    pub unsafe fn start_output(
        device: AudioDevice,
        sample_rate: f64,
        channels: u32,
        callback: RenderCallback,
        context: *mut c_void,
    ) -> Result<Self> {
        unsafe {
            let desc = AudioComponentDescription {
                component_type: K_AUDIO_UNIT_TYPE_OUTPUT,
                component_sub_type: K_AUDIO_UNIT_SUB_TYPE_HAL_OUTPUT,
                component_manufacturer: K_AUDIO_UNIT_MANUFACTURER_APPLE,
                component_flags: 0,
                component_flags_mask: 0,
            };

            let comp = AudioComponentFindNext(std::ptr::null_mut(), &desc);
            if comp.is_null() {
                return Err(CoreAudioError::Unspecified);
            }

            let mut audio_unit: AudioUnit = std::ptr::null_mut();
            CoreAudioError::from_os_status(AudioComponentInstanceNew(comp, &mut audio_unit))?;

            let enable_io: u32 = 1;
            CoreAudioError::from_os_status(AudioUnitSetProperty(
                audio_unit,
                K_AUDIO_OUTPUT_UNIT_PROPERTY_ENABLE_IO,
                K_AUDIO_UNIT_SCOPE_OUTPUT,
                0,
                &enable_io as *const u32 as *const c_void,
                std::mem::size_of::<u32>() as u32,
            ))?;

            let disable_io: u32 = 0;
            let _ = AudioUnitSetProperty(
                audio_unit,
                K_AUDIO_OUTPUT_UNIT_PROPERTY_ENABLE_IO,
                K_AUDIO_UNIT_SCOPE_INPUT,
                1,
                &disable_io as *const u32 as *const c_void,
                std::mem::size_of::<u32>() as u32,
            );

            let device_id = device.id;
            CoreAudioError::from_os_status(AudioUnitSetProperty(
                audio_unit,
                K_AUDIO_OUTPUT_UNIT_PROPERTY_CURRENT_DEVICE,
                K_AUDIO_UNIT_SCOPE_GLOBAL,
                0,
                &device_id as *const AudioObjectID as *const c_void,
                std::mem::size_of::<AudioObjectID>() as u32,
            ))?;

            let bytes_per_frame = channels * 4;
            let asbd = AudioStreamBasicDescription {
                m_sample_rate: sample_rate,
                m_format_id: K_AUDIO_FORMAT_LINEAR_PCM,
                m_format_flags: K_AUDIO_FORMAT_FLAG_IS_FLOAT | K_AUDIO_FORMAT_FLAG_IS_PACKED,
                m_bytes_per_packet: bytes_per_frame,
                m_frames_per_packet: 1,
                m_bytes_per_frame: bytes_per_frame,
                m_channels_per_frame: channels,
                m_bits_per_channel: 32,
                m_reserved: 0,
            };

            CoreAudioError::from_os_status(AudioUnitSetProperty(
                audio_unit,
                K_AUDIO_UNIT_PROPERTY_STREAM_FORMAT,
                K_AUDIO_UNIT_SCOPE_INPUT,
                0,
                &asbd as *const _ as *const c_void,
                std::mem::size_of::<AudioStreamBasicDescription>() as u32,
            ))?;

            let internal_ctx = Box::new(InternalContext {
                user_callback: callback,
                user_context: context,
                audio_unit,
                channels,
            });

            let ctx_ptr = Box::into_raw(internal_ctx);

            let callback_struct = AURenderCallbackStruct {
                input_proc: core_audio_trampoline,
                input_proc_ref_con: ctx_ptr as *mut c_void,
            };

            CoreAudioError::from_os_status(AudioUnitSetProperty(
                audio_unit,
                K_AUDIO_UNIT_PROPERTY_SET_RENDER_CALLBACK,
                K_AUDIO_UNIT_SCOPE_INPUT,
                0,
                &callback_struct as *const _ as *const c_void,
                std::mem::size_of::<AURenderCallbackStruct>() as u32,
            ))?;

            CoreAudioError::from_os_status(AudioUnitInitialize(audio_unit))?;
            CoreAudioError::from_os_status(AudioOutputUnitStart(audio_unit))?;

            Ok(Self {
                audio_unit,
                _context_box: ctx_ptr,
            })
        }
    }

    pub unsafe fn start_input(
        device: AudioDevice,
        sample_rate: f64,
        channels: u32,
        callback: RenderCallback,
        context: *mut c_void,
    ) -> Result<Self> {
        unsafe {
            let desc = AudioComponentDescription {
                component_type: K_AUDIO_UNIT_TYPE_OUTPUT,
                component_sub_type: K_AUDIO_UNIT_SUB_TYPE_HAL_OUTPUT,
                component_manufacturer: K_AUDIO_UNIT_MANUFACTURER_APPLE,
                component_flags: 0,
                component_flags_mask: 0,
            };

            let comp = AudioComponentFindNext(std::ptr::null_mut(), &desc);
            if comp.is_null() {
                return Err(CoreAudioError::Unspecified);
            }

            let mut audio_unit: AudioUnit = std::ptr::null_mut();
            CoreAudioError::from_os_status(AudioComponentInstanceNew(comp, &mut audio_unit))?;

            // Enable IO on input bus (Bus 1)
            let enable_io: u32 = 1;
            CoreAudioError::from_os_status(AudioUnitSetProperty(
                audio_unit,
                K_AUDIO_OUTPUT_UNIT_PROPERTY_ENABLE_IO,
                K_AUDIO_UNIT_SCOPE_INPUT,
                1,
                &enable_io as *const u32 as *const c_void,
                std::mem::size_of::<u32>() as u32,
            ))?;

            // Disable IO on output bus (Bus 0)
            let disable_io: u32 = 0;
            let _ = AudioUnitSetProperty(
                audio_unit,
                K_AUDIO_OUTPUT_UNIT_PROPERTY_ENABLE_IO,
                K_AUDIO_UNIT_SCOPE_OUTPUT,
                0,
                &disable_io as *const u32 as *const c_void,
                std::mem::size_of::<u32>() as u32,
            );

            let device_id = device.id;
            CoreAudioError::from_os_status(AudioUnitSetProperty(
                audio_unit,
                K_AUDIO_OUTPUT_UNIT_PROPERTY_CURRENT_DEVICE,
                K_AUDIO_UNIT_SCOPE_GLOBAL,
                0,
                &device_id as *const AudioObjectID as *const c_void,
                std::mem::size_of::<AudioObjectID>() as u32,
            ))?;

            let bytes_per_frame = channels * 4;
            let asbd = AudioStreamBasicDescription {
                m_sample_rate: sample_rate,
                m_format_id: K_AUDIO_FORMAT_LINEAR_PCM,
                m_format_flags: K_AUDIO_FORMAT_FLAG_IS_FLOAT | K_AUDIO_FORMAT_FLAG_IS_PACKED,
                m_bytes_per_packet: bytes_per_frame,
                m_frames_per_packet: 1,
                m_bytes_per_frame: bytes_per_frame,
                m_channels_per_frame: channels,
                m_bits_per_channel: 32,
                m_reserved: 0,
            };

            CoreAudioError::from_os_status(AudioUnitSetProperty(
                audio_unit,
                K_AUDIO_UNIT_PROPERTY_STREAM_FORMAT,
                K_AUDIO_UNIT_SCOPE_OUTPUT,
                1,
                &asbd as *const _ as *const c_void,
                std::mem::size_of::<AudioStreamBasicDescription>() as u32,
            ))?;

            let internal_ctx = Box::new(InternalContext {
                user_callback: callback,
                user_context: context,
                audio_unit,
                channels,
            });

            let ctx_ptr = Box::into_raw(internal_ctx);

            let callback_struct = AURenderCallbackStruct {
                input_proc: core_audio_input_trampoline,
                input_proc_ref_con: ctx_ptr as *mut c_void,
            };

            CoreAudioError::from_os_status(AudioUnitSetProperty(
                audio_unit,
                K_AUDIO_OUTPUT_UNIT_PROPERTY_SET_INPUT_CALLBACK,
                K_AUDIO_UNIT_SCOPE_GLOBAL,
                0,
                &callback_struct as *const _ as *const c_void,
                std::mem::size_of::<AURenderCallbackStruct>() as u32,
            ))?;

            CoreAudioError::from_os_status(AudioUnitInitialize(audio_unit))?;
            CoreAudioError::from_os_status(AudioOutputUnitStart(audio_unit))?;

            Ok(Self {
                audio_unit,
                _context_box: ctx_ptr,
            })
        }
    }
}

impl Drop for AudioStream {
    fn drop(&mut self) {
        unsafe {
            AudioOutputUnitStop(self.audio_unit);
            AudioComponentInstanceDispose(self.audio_unit);
            let _ = Box::from_raw(self._context_box);
        }
    }
}
