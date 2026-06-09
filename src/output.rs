use crate::*;
use std::{
    sync::Once,
    time::{Duration, Instant},
};
use wasapi::*;

//Initialise the COM library.
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

pub fn new_output(device: Device, sample_rate: Option<u32>) -> Output {
    unsafe {
        ONCE.call_once(|| CoInitializeEx(ConcurrencyModel::MultiThreaded).unwrap());

        let client: IAudioClient = device.imm.Activate(ExecutionContext::All).unwrap();
        let mut format =
            (client.GetMixFormat().unwrap() as *const _ as *const WAVEFORMATEXTENSIBLE).read();

        if format.Format.nChannels < 2 {
            todo!("Support mono devices.");
        }

        //Update format to desired sample rate.
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

pub fn update_sample_rate(output: &mut Output, sample_rate: u32) {
    if output.format.Format.nSamplesPerSec != sample_rate {
        unsafe { output.client.Stop().unwrap() };
        *output = new_output(output.device.clone(), Some(sample_rate));
    }
}

pub fn fill_buffer(output: &Output, decoder: &mut Option<Symphonia>) -> u32 {
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

        //Don't abruptly change the volume/gain when filling packets.
        //I don't know how much overhead a threadcell creates. Maybe it's fine?
        let volume = *VOLUME;
        let gain = *GAIN;

        for bytes in buffer.chunks_mut(std::mem::size_of::<f32>() * channels) {
            //Only allow for stereo outputs.
            for c in 0..channels.min(2) {
                if let Some(decoder) = decoder {
                    let sample = decoder.next_sample();

                    if sample.is_none() {
                        FINISHED.store(true, Relaxed);
                    }

                    let sample = sample.unwrap_or_default();
                    let value = (sample * volume * gain).to_le_bytes();
                    bytes[c * 4..c * 4 + 4].copy_from_slice(&value);
                } else {
                    //If there is no decoder, just fill with zeroes.
                    bytes[c * 4..c * 4 + 4].fill(0);
                }
            }
        }

        output.render.ReleaseBuffer(frames, 0).unwrap();
        return frames;
    }
}

//TODO: How can I log the errors from here?
//Maybe some type of callback?
pub fn run(output: Output) {
    unsafe {
        let (period, _) = output.client.GetDevicePeriod().unwrap();
        let mut period = Duration::from_nanos(period as u64 * 100);
        let mut last_event = Instant::now();

        set_pro_audio_thread();

        let mut output = Some(output);
        let mut decoder: Option<Symphonia> = None;

        loop {
            if let Some(new_output) = NEXT_OUTPUT.take() {
                let (device_period, _) = new_output.client.GetDevicePeriod().unwrap();
                period = Duration::from_nanos(device_period as u64 * 100);
                output = Some(new_output);
            }

            if let Some(new_decoder) = NEXT_DECODER.take() {
                decoder = Some(new_decoder);
            }

            //There should always be a valid output device.
            //Otherwise there would be no event handle and we couldn't wait.
            let output = output.as_mut().unwrap();

            //Seek if the user wants too :)
            let seek = SEEK.load(Relaxed);
            if seek != u64::MAX {
                if let Some(decoder) = decoder.as_mut() {
                    decoder.seek(Duration::from_nanos(seek));
                }
                SEEK.store(u64::MAX, Relaxed);
            }

            WaitForSingleObject(output.event, u32::MAX);

            let now = Instant::now();
            let elapsed = now - last_event;
            last_event = now;

            if elapsed > period + Duration::from_millis(2) {
                // println!("WARNING: Waited {:?} (expected {:?})", elapsed, period);
            }

            let mut i = 0;
            let mut frames = u32::MAX;

            while frames != 0 {
                if *STATE != State::Playing || FINISHED.load(Relaxed) {
                    break;
                }

                frames = fill_buffer(&output, &mut decoder);

                if i > 1 {
                    // println!(
                    //     "WARNING: Missed event(s) or underflow, buffer had space for {} frames! iteration: {}",
                    //     frames, i
                    // );
                }
                i += 1;
            }
        }
    }
}
