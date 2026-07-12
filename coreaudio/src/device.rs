use crate::cf::{CFRelease, CFStringRef, cfstring_to_string};
use crate::error::{CoreAudioError, Result};
use crate::ffi::*;
use std::ffi::c_void;

pub struct AudioDevice {
    pub id: AudioObjectID,
}

impl Clone for AudioDevice {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for AudioDevice {}

impl PartialEq for AudioDevice {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for AudioDevice {}

impl std::fmt::Debug for AudioDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AudioDevice {{ id: {} }}", self.id)
    }
}

impl AudioDevice {
    pub fn new(id: AudioObjectID) -> Self {
        Self { id }
    }

    pub fn system_devices() -> Result<Vec<AudioDevice>> {
        let address = AudioObjectPropertyAddress {
            m_selector: K_AUDIO_HARDWARE_PROPERTY_DEVICES,
            m_scope: K_AUDIO_OBJECT_PROPERTY_SCOPE_GLOBAL,
            m_element: K_AUDIO_OBJECT_PROPERTY_ELEMENT_MAIN,
        };

        let mut data_size: u32 = 0;
        let status = unsafe {
            AudioObjectGetPropertyDataSize(
                K_AUDIO_OBJECT_SYSTEM_OBJECT,
                &address,
                0,
                std::ptr::null(),
                &mut data_size,
            )
        };
        CoreAudioError::from_os_status(status)?;

        let device_count = (data_size as usize) / std::mem::size_of::<AudioObjectID>();
        let mut device_ids = vec![0 as AudioObjectID; device_count];

        let status = unsafe {
            AudioObjectGetPropertyData(
                K_AUDIO_OBJECT_SYSTEM_OBJECT,
                &address,
                0,
                std::ptr::null(),
                &mut data_size,
                device_ids.as_mut_ptr() as *mut c_void,
            )
        };
        CoreAudioError::from_os_status(status)?;

        Ok(device_ids.into_iter().map(AudioDevice::new).collect())
    }

    pub fn default_input() -> Result<AudioDevice> {
        Self::get_default_device(K_AUDIO_HARDWARE_PROPERTY_DEFAULT_INPUT_DEVICE)
    }

    pub fn default_output() -> Result<AudioDevice> {
        Self::get_default_device(K_AUDIO_HARDWARE_PROPERTY_DEFAULT_OUTPUT_DEVICE)
    }

    fn get_default_device(selector: u32) -> Result<AudioDevice> {
        let address = AudioObjectPropertyAddress {
            m_selector: selector,
            m_scope: K_AUDIO_OBJECT_PROPERTY_SCOPE_GLOBAL,
            m_element: K_AUDIO_OBJECT_PROPERTY_ELEMENT_MAIN,
        };
        let mut device_id: AudioObjectID = 0;
        let mut data_size = std::mem::size_of::<AudioObjectID>() as u32;

        let status = unsafe {
            AudioObjectGetPropertyData(
                K_AUDIO_OBJECT_SYSTEM_OBJECT,
                &address,
                0,
                std::ptr::null(),
                &mut data_size,
                &mut device_id as *mut AudioObjectID as *mut c_void,
            )
        };
        CoreAudioError::from_os_status(status)?;
        Ok(AudioDevice::new(device_id))
    }

    pub fn sample_rate(&self) -> Result<f64> {
        let address = AudioObjectPropertyAddress {
            m_selector: K_AUDIO_DEVICE_PROPERTY_NOMINAL_SAMPLE_RATE,
            m_scope: K_AUDIO_OBJECT_PROPERTY_SCOPE_GLOBAL,
            m_element: K_AUDIO_OBJECT_PROPERTY_ELEMENT_MAIN,
        };
        let mut sample_rate: f64 = 0.0;
        let mut data_size = std::mem::size_of::<f64>() as u32;
        let status = unsafe {
            AudioObjectGetPropertyData(
                self.id,
                &address,
                0,
                std::ptr::null(),
                &mut data_size,
                &mut sample_rate as *mut f64 as *mut c_void,
            )
        };
        CoreAudioError::from_os_status(status)?;
        Ok(sample_rate)
    }

    pub fn name(&self) -> Result<String> {
        let address = AudioObjectPropertyAddress {
            m_selector: K_AUDIO_DEVICE_PROPERTY_DEVICE_NAME_CFSTRING,
            m_scope: K_AUDIO_OBJECT_PROPERTY_SCOPE_GLOBAL,
            m_element: K_AUDIO_OBJECT_PROPERTY_ELEMENT_MAIN,
        };

        let mut cf_str: CFStringRef = std::ptr::null();
        let mut data_size = std::mem::size_of::<CFStringRef>() as u32;

        let status = unsafe {
            AudioObjectGetPropertyData(
                self.id,
                &address,
                0,
                std::ptr::null(),
                &mut data_size,
                &mut cf_str as *mut CFStringRef as *mut c_void,
            )
        };
        CoreAudioError::from_os_status(status)?;

        let name = cfstring_to_string(cf_str).unwrap_or_else(|| "Unknown Device".to_string());

        if !cf_str.is_null() {
            unsafe { CFRelease(cf_str) };
        }

        Ok(name)
    }

    pub fn input_channel_count(&self) -> Result<u32> {
        self.channel_count(K_AUDIO_OBJECT_PROPERTY_SCOPE_INPUT)
    }

    pub fn output_channel_count(&self) -> Result<u32> {
        self.channel_count(K_AUDIO_OBJECT_PROPERTY_SCOPE_OUTPUT)
    }

    fn channel_count(&self, scope: u32) -> Result<u32> {
        let address = AudioObjectPropertyAddress {
            m_selector: K_AUDIO_DEVICE_PROPERTY_STREAM_CONFIGURATION,
            m_scope: scope,
            m_element: K_AUDIO_OBJECT_PROPERTY_ELEMENT_MAIN,
        };

        let mut data_size: u32 = 0;
        let status = unsafe {
            AudioObjectGetPropertyDataSize(self.id, &address, 0, std::ptr::null(), &mut data_size)
        };

        if status != 0 {
            return Ok(0);
        }
        if data_size == 0 {
            return Ok(0);
        }

        let mut buffer_list_data = vec![0u8; data_size as usize];
        let status = unsafe {
            AudioObjectGetPropertyData(
                self.id,
                &address,
                0,
                std::ptr::null(),
                &mut data_size,
                buffer_list_data.as_mut_ptr() as *mut c_void,
            )
        };

        if status != 0 {
            return Ok(0);
        }

        let buffer_list = unsafe { &*(buffer_list_data.as_ptr() as *const AudioBufferList) };

        let mut total_channels = 0;

        let buffers_ptr = buffer_list.m_buffers.as_ptr();
        for i in 0..buffer_list.m_number_buffers {
            let buffer = unsafe { &*buffers_ptr.add(i as usize) };
            total_channels += buffer.m_number_channels;
        }

        Ok(total_channels)
    }
}
