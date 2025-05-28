use crate::*;
use std::time::{Duration, Instant};
use wasapi::*;

pub struct WasapiOutput {
    pub client: IAudioClient,
    pub render: IAudioRenderClient,
    pub format: WAVEFORMATEXTENSIBLE,
    pub enumerator: IMMDeviceEnumerator,
    pub event: *mut c_void,
    pub playing: bool,
}

impl WasapiOutput {
    pub fn new() -> Self {
        //Use the default sample rate.
        Self::new_with_sample_rate(None)
    }

    pub fn new_with_sample_rate(sample_rate: Option<u32>) -> Self {
        unsafe {
            CoInitializeEx(ConcurrencyModel::MultiThreaded).unwrap();
            // set_pro_audio_thread();

            let enumerator = IMMDeviceEnumerator::new().unwrap();
            let device = enumerator
                .GetDefaultAudioEndpoint(DataFlow::Render, Role::Console)
                .unwrap();

            let client: IAudioClient = device.Activate(ExecutionContext::All).unwrap();
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

            Self {
                enumerator,
                client,
                render,
                format,
                event,
                playing: true,
            }
        }
    }

    pub fn devices(&self) -> Vec<(String, IMMDevice)> {
        unsafe {
            let collection = self
                .enumerator
                .EnumAudioEndpoints(DataFlow::Render, DeviceState::Active)
                .unwrap();

            (0..collection.GetCount().unwrap())
                .map(|i| {
                    let device = collection.Item(i).unwrap();
                    (device.name(), device)
                })
                .collect()
        }
    }

    pub fn update_sample_rate(&mut self, sample_rate: u32) {
        if self.format.Format.nSamplesPerSec != sample_rate {
            unsafe { self.client.Stop().unwrap() };
            *self = Self::new_with_sample_rate(Some(sample_rate));
        }
    }

    #[inline]
    pub const fn sample_rate(&self) -> u32 {
        self.format.Format.nSamplesPerSec
    }

    #[inline]
    pub const fn block_align(&self) -> u16 {
        self.format.Format.nBlockAlign
    }

    #[inline]
    pub const fn channels(&self) -> u16 {
        self.format.Format.nChannels
    }

    pub const fn play(&mut self) {
        self.playing = true;
    }

    pub const fn pause(&mut self) {
        self.playing = false;
    }

    pub fn fill<F: FnMut(&mut [u8])>(&self, mut f: F) -> u32 {
        unsafe {
            let padding = self.client.GetCurrentPadding().unwrap();
            let buffer_size = self.client.GetBufferSize().unwrap();
            let block_align = self.block_align();
            let frames = buffer_size - padding;

            if frames == 0 {
                return frames;
            }

            let size = (frames * block_align as u32) as usize;
            let ptr = self.render.GetBuffer(frames).unwrap();
            let buffer = std::slice::from_raw_parts_mut(ptr, size);

            f(buffer);

            self.render.ReleaseBuffer(frames, 0).unwrap();
            return frames;
        }
    }

    pub fn run<F: FnMut(&mut [u8])>(&self, mut f: F) {
        unsafe {
            //Allow for playback options to be changed.
            PLAYBACK.reset_thread();

            let (period, _) = self.client.GetDevicePeriod().unwrap();
            let period = Duration::from_nanos(period as u64 * 100);
            let mut last_event = Instant::now();
            set_pro_audio_thread();

            loop {
                WaitForSingleObject(self.event, u32::MAX);

                let now = Instant::now();
                let elapsed = now - last_event;
                last_event = now;

                if elapsed > period + Duration::from_millis(2) {
                    println!("WARNING: Waited {:?} (expected {:?})", elapsed, period);
                }

                let mut i = 0;
                let mut frames = u32::MAX;

                while frames != 0 {
                    if self.playing {
                        frames = self.fill(&mut f);
                    } else {
                        frames = 0;
                    }

                    if i > 1 {
                        println!(
                            "WARNING: Missed event(s) or underflow, buffer had space for {} frames! iteration: {}",
                            frames, i
                        );
                    }
                    i += 1;
                }
            }
        }
    }
}
