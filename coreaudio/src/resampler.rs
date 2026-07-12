use crate::ffi::*;
use crate::error::{CoreAudioError, Result};
use std::ffi::c_void;

pub struct AudioConverter {
    converter: AudioConverterRef,
    source_asbd: AudioStreamBasicDescription,
    dest_asbd: AudioStreamBasicDescription,
}

struct InputCallbackUserData {
    pub data_ptr: *const f32,
    pub samples_remaining: u32,
    pub channels: u32,
}

unsafe extern "C" fn input_data_proc(
    _in_audio_converter: AudioConverterRef,
    io_number_data_packets: *mut u32,
    io_data: *mut AudioBufferList,
    _out_data_packet_description: *mut *mut AudioStreamPacketDescription,
    in_user_data: *mut c_void,
) -> OSStatus {
    unsafe {
        let user_data = &mut *(in_user_data as *mut InputCallbackUserData);
        
        let requested_packets = *io_number_data_packets;
        if user_data.samples_remaining == 0 {
            *io_number_data_packets = 0;
            return 0;
        }
        
        let channels = user_data.channels;
        let frames_to_copy = std::cmp::min(requested_packets, user_data.samples_remaining / channels);
        
        if frames_to_copy == 0 {
            *io_number_data_packets = 0;
            return 0;
        }
        
        let buffer_list = &mut *io_data;
        buffer_list.m_number_buffers = 1;
        buffer_list.m_buffers[0].m_number_channels = channels;
        buffer_list.m_buffers[0].m_data_byte_size = frames_to_copy * channels * 4;
        buffer_list.m_buffers[0].m_data = user_data.data_ptr as *mut c_void;
        
        *io_number_data_packets = frames_to_copy;
        
        user_data.data_ptr = user_data.data_ptr.add((frames_to_copy * channels) as usize);
        user_data.samples_remaining -= frames_to_copy * channels;
        
        0
    }
}

impl AudioConverter {
    pub fn new(
        source_format: &AudioStreamBasicDescription,
        dest_format: &AudioStreamBasicDescription,
    ) -> Result<Self> {
        let mut converter: AudioConverterRef = std::ptr::null_mut();
        let status = unsafe {
            AudioConverterNew(source_format, dest_format, &mut converter)
        };
        CoreAudioError::from_os_status(status)?;
        Ok(Self {
            converter,
            source_asbd: *source_format,
            dest_asbd: *dest_format,
        })
    }
    
    pub fn convert(&self, input: &[f32], output: &mut [f32]) -> Result<usize> {
        let source_channels = self.source_asbd.m_channels_per_frame;
        let dest_channels = self.dest_asbd.m_channels_per_frame;
        
        let mut user_data = InputCallbackUserData {
            data_ptr: input.as_ptr(),
            samples_remaining: input.len() as u32,
            channels: source_channels,
        };
        
        let mut output_buffer_list = AudioBufferList {
            m_number_buffers: 1,
            m_buffers: [AudioBuffer {
                m_number_channels: dest_channels,
                m_data_byte_size: (output.len() * 4) as u32,
                m_data: output.as_mut_ptr() as *mut c_void,
            }],
        };
        
        let mut io_output_packets = (output.len() as u32) / dest_channels;
        
        let status = unsafe {
            AudioConverterFillComplexBuffer(
                self.converter,
                input_data_proc,
                &mut user_data as *mut _ as *mut c_void,
                &mut io_output_packets,
                &mut output_buffer_list,
                std::ptr::null_mut(),
            )
        };
        
        if status != 0 && io_output_packets == 0 {
            CoreAudioError::from_os_status(status)?;
        }
        
        Ok((io_output_packets * dest_channels) as usize)
    }
}

impl Drop for AudioConverter {
    fn drop(&mut self) {
        unsafe {
            let _ = AudioConverterDispose(self.converter);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_native_resample() {
        let src_asbd = AudioStreamBasicDescription {
            m_sample_rate: 44100.0,
            m_format_id: K_AUDIO_FORMAT_LINEAR_PCM,
            m_format_flags: K_AUDIO_FORMAT_FLAG_IS_FLOAT | K_AUDIO_FORMAT_FLAG_IS_PACKED,
            m_bytes_per_packet: 4,
            m_frames_per_packet: 1,
            m_bytes_per_frame: 4,
            m_channels_per_frame: 1,
            m_bits_per_channel: 32,
            m_reserved: 0,
        };

        let dest_asbd = AudioStreamBasicDescription {
            m_sample_rate: 88200.0,
            m_format_id: K_AUDIO_FORMAT_LINEAR_PCM,
            m_format_flags: K_AUDIO_FORMAT_FLAG_IS_FLOAT | K_AUDIO_FORMAT_FLAG_IS_PACKED,
            m_bytes_per_packet: 4,
            m_frames_per_packet: 1,
            m_bytes_per_frame: 4,
            m_channels_per_frame: 1,
            m_bits_per_channel: 32,
            m_reserved: 0,
        };

        let converter = AudioConverter::new(&src_asbd, &dest_asbd).expect("Failed to create AudioConverter");
        
        let input = vec![0.0f32, 1.0, 2.0, 3.0];
        let mut output = vec![0.0f32; 10];
        
        let written = converter.convert(&input, &mut output).expect("Failed to convert");
        
        assert!(written > 0);
        assert!(written <= output.len());
        
        // Basic range check of converted outputs
        for &val in &output[..written] {
            assert!(val >= 0.0 && val <= 3.0);
        }
    }
}
