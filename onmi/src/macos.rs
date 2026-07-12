use crate::*;
use coreaudio::{AudioDevice, AudioStream};
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

pub fn new_output(device: Device, sample_rate: Option<u32>) -> Output {
    let mut channels = device.audio.output_channel_count().unwrap_or(2);
    if channels < 2 {
        channels = 2;
    }

    let sample_rate = match sample_rate {
        Some(rate) => {
            assert!(COMMON_SAMPLE_RATES.contains(&rate));
            rate
        }
        None => device.audio.sample_rate().unwrap_or(44100.0) as u32,
    };

    Output {
        device,
        sample_rate,
        channels,
    }
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

fn start_stream(output: &Output, ctx: *mut c_void) -> AudioStream {
    unsafe {
        AudioStream::start_output(
            output.device.audio,
            output.sample_rate as f64,
            output.channels,
            render_callback,
            ctx,
        )
        .unwrap()
    }
}

pub fn run_output(state: Arc<PlayerState>, mut output: Output) {
    let mut ctx = Box::new(AudioCtx {
        state: Arc::clone(&state),
        decoder: None,
        channels: output.channels,
    });
    let ctx_ptr = ctx.as_mut() as *mut AudioCtx as *mut c_void;

    let mut stream = start_stream(&output, ctx_ptr);

    loop {
        if let Some(new_output) = state.pending_output.take() {
            drop(stream);
            output = new_output;
            ctx.channels = output.channels;
            stream = start_stream(&output, ctx_ptr);
        }

        std::thread::park_timeout(Duration::from_millis(50));
    }
}
