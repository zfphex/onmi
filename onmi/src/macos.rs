use crate::*;
use coreaudio::{
    AudioDevice, AudioObjectGetPropertyData, AudioObjectID, AudioObjectPropertyAddress,
    AudioStream, K_AUDIO_HARDWARE_PROPERTY_DEFAULT_OUTPUT_DEVICE,
    K_AUDIO_OBJECT_PROPERTY_ELEMENT_MAIN, K_AUDIO_OBJECT_PROPERTY_SCOPE_GLOBAL,
    K_AUDIO_OBJECT_SYSTEM_OBJECT,
};
use std::ffi::c_void;
use std::sync::atomic::Ordering::{AcqRel, Relaxed};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Device {
    pub audio: AudioDevice,
    pub name: String,
}

pub struct OutputDevices;

impl OutputDevices {
    pub fn new() -> Self {
        Self
    }

    pub fn default_device(&self) -> Device {
        let audio = AudioDevice::default_output().unwrap();
        Device {
            name: audio.name().unwrap_or_else(|_| "Unknown".to_string()),
            audio,
        }
    }

    pub fn devices(&self) -> Vec<Device> {
        AudioDevice::system_devices()
            .unwrap_or_default()
            .into_iter()
            .filter(|d| d.output_channel_count().unwrap_or(0) > 0)
            .map(|audio| Device {
                name: audio.name().unwrap_or_else(|_| "Unknown".to_string()),
                audio,
            })
            .collect()
    }

    pub fn find(&self, device: &str) -> Option<Device> {
        self.devices()
            .into_iter()
            .find(|d| d.name.as_str() == device)
    }
}

pub struct Output {
    pub device: Device,
    pub sample_rate: u32,
    pub channels: u32,
}

unsafe impl Send for Output {}

pub fn try_new_output(device: Device, sample_rate: Option<u32>) -> Option<Output> {
    let channels = match device.audio.output_channel_count() {
        Ok(0) | Err(_) => 2,
        Ok(c) => c,
    };

    let sample_rate = match sample_rate {
        Some(rate) => {
            if !COMMON_SAMPLE_RATES.contains(&rate) {
                return None;
            }
            rate
        }
        None => device.audio.sample_rate().unwrap_or(44100.0) as u32,
    };

    Some(Output {
        device,
        sample_rate,
        channels,
    })
}

pub fn new_output(device: Device, sample_rate: Option<u32>) -> Output {
    try_new_output(device, sample_rate).expect("failed to open output")
}

struct AudioCtx {
    state: Arc<PlayerState>,
    decoder: Option<Symphonia>,
    channels: u32,
}

extern "C" fn render_callback(context: *mut c_void, buffer_ptr: *mut f32, total_samples: usize) {
    unsafe {
        let ctx = &mut *(context as *mut AudioCtx);
        let state = &*ctx.state;

        if state.shutdown.load(Relaxed) {
            let buffer =
                std::slice::from_raw_parts_mut(buffer_ptr, total_samples);
            buffer.fill(0.0);
            return;
        }

        if let Some(new_decoder) = state.pending_decoder.take() {
            state.finished.store(false, Relaxed);
            ctx.decoder = Some(new_decoder);
            state.decoder_pending.store(false, Relaxed);
        }

        let seek = state.seek.swap(u64::MAX, AcqRel);
        if seek != u64::MAX {
            if let Some(decoder) = ctx.decoder.as_mut() {
                decoder.seek(Duration::from_nanos(seek), state);
            }
        }

        let channels = ctx.channels as usize;
        let buffer =
            std::slice::from_raw_parts_mut(buffer_ptr as *mut u8, total_samples * size_of::<f32>());
        fill_f32_le(state, &mut ctx.decoder, buffer, channels);
    }
}

fn start_stream(output: &Output, ctx: *mut c_void) -> Option<AudioStream> {
    unsafe {
        AudioStream::start_output(
            output.device.audio,
            output.sample_rate as f64,
            output.channels,
            render_callback,
            ctx,
        )
        .ok()
    }
}

fn default_output_id() -> Option<AudioObjectID> {
    let address = AudioObjectPropertyAddress {
        m_selector: K_AUDIO_HARDWARE_PROPERTY_DEFAULT_OUTPUT_DEVICE,
        m_scope: K_AUDIO_OBJECT_PROPERTY_SCOPE_GLOBAL,
        m_element: K_AUDIO_OBJECT_PROPERTY_ELEMENT_MAIN,
    };
    let mut device_id: AudioObjectID = 0;
    let mut data_size = size_of::<AudioObjectID>() as u32;
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
    if status == 0 && device_id != 0 {
        Some(device_id)
    } else {
        None
    }
}

pub fn run_output(state: Arc<PlayerState>, mut output: Output) {
    let mut ctx = Box::new(AudioCtx {
        state: Arc::clone(&state),
        decoder: None,
        channels: output.channels,
    });
    let ctx_ptr = ctx.as_mut() as *mut AudioCtx as *mut c_void;

    let mut stream = match start_stream(&output, ctx_ptr) {
        Some(s) => Some(s),
        None => {
            state.set_error(RuntimeError::StreamStart);
            None
        }
    };

    let mut last_default = default_output_id();

    loop {
        if state.shutdown.load(Relaxed) {
            break;
        }

        if let Some(new_output) = state.pending_output.take() {
            drop(stream.take());
            output = new_output;
            ctx.channels = output.channels;
            stream = match start_stream(&output, ctx_ptr) {
                Some(s) => Some(s),
                None => {
                    state.set_error(RuntimeError::StreamStart);
                    None
                }
            };
        }

        if state.follow_default.load(Relaxed) {
            let current_default = default_output_id();
            if current_default.is_some() && current_default != last_default {
                last_default = current_default;
                if let Some(id) = current_default {
                    if id != output.device.audio.id {
                        let device = Device {
                            name: AudioDevice::new(id)
                                .name()
                                .unwrap_or_else(|_| "Unknown".to_string()),
                            audio: AudioDevice::new(id),
                        };
                        if let Some(new_output) =
                            try_new_output(device, Some(output.sample_rate))
                        {
                            drop(stream.take());
                            output = new_output;
                            ctx.channels = output.channels;
                            stream = match start_stream(&output, ctx_ptr) {
                                Some(s) => Some(s),
                                None => {
                                    state.set_error(RuntimeError::StreamStart);
                                    None
                                }
                            };
                        } else {
                            state.set_error(RuntimeError::OutputOpen);
                        }
                    }
                }
            }
        }

        std::thread::park_timeout(Duration::from_millis(10));
    }

    drop(stream.take());
}
