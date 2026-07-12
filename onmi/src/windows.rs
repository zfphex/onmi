pub use wasapi::IMMDevice;

#[derive(Debug, Clone)]
pub struct Device {
    pub imm: IMMDevice,
    pub name: String,
}

unsafe impl Send for Device {}
unsafe impl Sync for Device {}

use crate::*;
use std::sync::atomic::Ordering::{AcqRel, Relaxed};
use std::sync::{Arc, Once};
use std::time::Duration;
use wasapi::*;

static ONCE: Once = Once::new();

pub struct OutputDevices {
    pub enumerator: IMMDeviceEnumerator,
}

impl OutputDevices {
    pub fn new() -> Self {
        unsafe { ONCE.call_once(|| CoInitializeEx(ConcurrencyModel::MultiThreaded).unwrap()) };

        Self {
            enumerator: IMMDeviceEnumerator::new().unwrap(),
        }
    }

    pub fn default_device(&self) -> Device {
        unsafe {
            let imm = self
                .enumerator
                .GetDefaultAudioEndpoint(DataFlow::Render, Role::Console)
                .unwrap();

            Device {
                name: imm.name(),
                imm,
            }
        }
    }

    pub fn devices(&self) -> Vec<Device> {
        unsafe {
            let collection = self
                .enumerator
                .EnumAudioEndpoints(DataFlow::Render, DeviceState::Active)
                .unwrap();

            (0..collection.GetCount().unwrap())
                .map(|i| {
                    let imm = collection.Item(i).unwrap();
                    Device {
                        name: imm.name(),
                        imm,
                    }
                })
                .collect()
        }
    }

    pub fn find(&self, device: &str) -> Option<Device> {
        self.devices()
            .iter()
            .find(|d| d.name.as_str() == device)
            .cloned()
    }
}

pub struct Output {
    pub client: IAudioClient,
    pub render: IAudioRenderClient,
    pub format: WAVEFORMATEXTENSIBLE,
    pub event: *mut c_void,
    pub device: Device,
}

unsafe impl Send for Output {}

pub fn new_output(device: Device, sample_rate: Option<u32>) -> Output {
    unsafe {
        ONCE.call_once(|| CoInitializeEx(ConcurrencyModel::MultiThreaded).unwrap());

        let client: IAudioClient = device.imm.Activate(ExecutionContext::All).unwrap();
        let mut format =
            (client.GetMixFormat().unwrap() as *const _ as *const WAVEFORMATEXTENSIBLE).read();

        if format.Format.nChannels < 2 {
            todo!("Support mono devices.");
        }

        if let Some(sample_rate) = sample_rate {
            assert!(COMMON_SAMPLE_RATES.contains(&sample_rate));
            format.Format.nSamplesPerSec = sample_rate;
            format.Format.nAvgBytesPerSec = sample_rate * format.Format.nBlockAlign as u32;
        }

        let (default, _) = client.GetDevicePeriod().unwrap();

        client
            .Initialize(
                ShareMode::Shared,
                AUDCLNT_STREAMFLAGS_EVENTCALLBACK
                    | AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM
                    | AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY,
                default,
                0,
                &format as *const _ as *const WAVEFORMATEX,
                None,
            )
            .unwrap();

        let event = CreateEventA(core::ptr::null_mut(), 0, 0, core::ptr::null_mut());
        assert!(!event.is_null());
        client.SetEventHandle(event as isize).unwrap();

        let render: IAudioRenderClient = client.GetService().unwrap();
        client.Start().unwrap();

        Output {
            client,
            render,
            format,
            event,
            device,
        }
    }
}

pub fn fill_buffer(
    state: &PlayerState,
    output: &Output,
    decoder: &mut Option<Symphonia>,
) -> u32 {
    unsafe {
        let padding = output.client.GetCurrentPadding().unwrap();
        let buffer_size = output.client.GetBufferSize().unwrap();
        let block_align = output.format.Format.nBlockAlign;
        let frames = buffer_size - padding;

        if frames == 0 {
            return frames;
        }

        let size = (frames * block_align as u32) as usize;
        let ptr = output.render.GetBuffer(frames).unwrap();
        let buffer = std::slice::from_raw_parts_mut(ptr, size);
        let channels = output.format.Format.nChannels as usize;

        fill_f32_le(state, decoder, buffer, channels);

        output.render.ReleaseBuffer(frames, 0).unwrap();
        frames
    }
}

pub fn run_output(state: Arc<PlayerState>, output: Output) {
    unsafe {
        set_pro_audio_thread();

        let mut output = Some(output);
        let mut decoder: Option<Symphonia> = None;

        loop {
            if let Some(new_output) = state.pending_output.take() {
                output = Some(new_output);
            }

            if let Some(new_decoder) = state.pending_decoder.take() {
                state.finished.store(false, Relaxed);
                if let Some(out) = output.as_ref() {
                    let _ = out.client.Stop();
                    let _ = out.client.Reset();
                    let _ = out.client.Start();
                }
                decoder = Some(new_decoder);
                state.decoder_pending.store(false, Relaxed);
            }

            let output = output.as_mut().unwrap();

            let seek = state.seek.swap(u64::MAX, AcqRel);
            if seek != u64::MAX {
                if let Some(decoder) = decoder.as_mut() {
                    decoder.seek(Duration::from_nanos(seek), &state);
                }
            }

            WaitForSingleObject(output.event, u32::MAX);

            let mut frames = u32::MAX;

            while frames != 0 {
                if state.state.load(Relaxed) != State::Playing as u8
                    || state.finished.load(Relaxed)
                    || state.decoder_pending.load(Relaxed)
                {
                    break;
                }

                frames = fill_buffer(&state, output, &mut decoder);
            }
        }
    }
}
