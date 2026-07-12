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

const WAIT_MS: u32 = 50;

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
            .into_iter()
            .find(|d| d.name.as_str() == device)
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

impl Drop for Output {
    fn drop(&mut self) {
        unsafe {
            let _ = self.client.Stop();
            if !self.event.is_null() {
                CloseHandle(self.event);
                self.event = core::ptr::null_mut();
            }
        }
    }
}

pub fn try_new_output(device: Device, sample_rate: Option<u32>) -> Option<Output> {
    unsafe {
        ONCE.call_once(|| {
            let _ = CoInitializeEx(ConcurrencyModel::MultiThreaded);
        });

        let client: IAudioClient = device.imm.Activate(ExecutionContext::All).ok()?;
        let mut format =
            (client.GetMixFormat().ok()? as *const _ as *const WAVEFORMATEXTENSIBLE).read();

        if format.Format.nChannels == 0 {
            return None;
        }

        if let Some(sample_rate) = sample_rate {
            if !COMMON_SAMPLE_RATES.contains(&sample_rate) {
                return None;
            }
            format.Format.nSamplesPerSec = sample_rate;
            format.Format.nAvgBytesPerSec = sample_rate * format.Format.nBlockAlign as u32;
        }

        let (default, _) = client.GetDevicePeriod().ok()?;

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
            .ok()?;

        let event = CreateEventA(core::ptr::null_mut(), 0, 0, core::ptr::null_mut());
        if event.is_null() {
            return None;
        }
        if client.SetEventHandle(event as isize).is_err() {
            CloseHandle(event);
            return None;
        }

        let render: IAudioRenderClient = client.GetService().ok()?;
        client.Start().ok()?;

        Some(Output {
            client,
            render,
            format,
            event,
            device,
        })
    }
}

pub fn new_output(device: Device, sample_rate: Option<u32>) -> Output {
    try_new_output(device, sample_rate).expect("failed to open output")
}

pub fn fill_buffer(
    state: &PlayerState,
    output: &Output,
    decoder: &mut Option<Symphonia>,
) -> u32 {
    unsafe {
        let padding = match output.client.GetCurrentPadding() {
            Ok(p) => p,
            Err(_) => {
                state.set_error(RuntimeError::StreamStart);
                return 0;
            }
        };
        let buffer_size = match output.client.GetBufferSize() {
            Ok(s) => s,
            Err(_) => {
                state.set_error(RuntimeError::StreamStart);
                return 0;
            }
        };
        let block_align = output.format.Format.nBlockAlign;
        let frames = buffer_size - padding;

        if frames == 0 {
            return frames;
        }

        let size = (frames * block_align as u32) as usize;
        let ptr = match output.render.GetBuffer(frames) {
            Ok(p) => p,
            Err(_) => {
                state.set_error(RuntimeError::StreamStart);
                return 0;
            }
        };
        let buffer = std::slice::from_raw_parts_mut(ptr, size);
        let channels = output.format.Format.nChannels as usize;

        fill_f32_le(state, decoder, buffer, channels);

        let _ = output.render.ReleaseBuffer(frames, 0);
        frames
    }
}

fn same_device(a: &Device, b: &Device) -> bool {
    a.name == b.name
}

pub fn run_output(state: Arc<PlayerState>, output: Output) {
    unsafe {
        set_pro_audio_thread();

        let mut output = Some(output);
        let mut decoder: Option<Symphonia> = None;
        let devices = OutputDevices::new();

        loop {
            if state.shutdown.load(Relaxed) {
                break;
            }

            if let Some(new_output) = state.pending_output.take() {
                if let Some(old) = output.take() {
                    let _ = old.client.Stop();
                    drop(old);
                }
                output = Some(new_output);
            }

            if state.follow_default.load(Relaxed) {
                if let Some(current) = output.as_ref() {
                    let def = devices.default_device();
                    if !same_device(&current.device, &def) {
                        let rate = current.format.Format.nSamplesPerSec;
                        if let Some(new_output) = try_new_output(def, Some(rate)) {
                            if let Some(old) = output.take() {
                                let _ = old.client.Stop();
                                drop(old);
                            }
                            output = Some(new_output);
                        } else {
                            state.set_error(RuntimeError::OutputOpen);
                        }
                    }
                }
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

            let Some(out) = output.as_mut() else {
                std::thread::sleep(Duration::from_millis(WAIT_MS as u64));
                continue;
            };

            let seek = state.seek.swap(u64::MAX, AcqRel);
            if seek != u64::MAX {
                if let Some(decoder) = decoder.as_mut() {
                    decoder.seek(Duration::from_nanos(seek), &state);
                }
            }

            WaitForSingleObject(out.event, WAIT_MS);

            if state.shutdown.load(Relaxed) {
                break;
            }

            let mut frames = u32::MAX;

            while frames != 0 {
                if state.state.load(Relaxed) != State::Playing as u8
                    || state.finished.load(Relaxed)
                    || state.decoder_pending.load(Relaxed)
                    || state.shutdown.load(Relaxed)
                {
                    break;
                }

                frames = fill_buffer(&state, out, &mut decoder);
            }
        }

        if let Some(out) = output.take() {
            let _ = out.client.Stop();
            drop(out);
        }
    }
}
