pub mod decoder;
pub mod rb;
use std::time::{Duration, Instant};

pub use decoder::*;
pub use rb::*;

pub const UNKNOWN_TITLE: &str = "Unknown Title";
pub const UNKNOWN_ALBUM: &str = "Unknown Album";
pub const UNKNOWN_ARTIST: &str = "Unknown Artist";
pub const COMMON_SAMPLE_RATES: [u32; 13] = [
    5512, 8000, 11025, 16000, 22050, 32000, 44100, 48000, 64000, 88200, 96000, 176400, 192000,
];

#[derive(Debug, Clone, PartialEq)]
pub struct Song {
    pub title: String,
    pub album: String,
    pub artist: String,
    pub disc_number: u8,
    pub track_number: u8,
    pub path: String,
    pub gain: f32,
}

impl Song {
    pub fn new() -> Self {
        Self {
            title: UNKNOWN_TITLE.to_string(),
            album: UNKNOWN_ALBUM.to_string(),
            artist: UNKNOWN_ARTIST.to_string(),
            disc_number: 1,
            track_number: 1,
            path: String::new(),
            gain: 0.0,
        }
    }
}

use wasapi::*;

pub struct Wasapi {
    pub client: IAudioClient,
    pub render: IAudioRenderClient,
    pub format: WAVEFORMATEXTENSIBLE,
    pub enumerator: IMMDeviceEnumerator,
    pub event: *mut c_void,
}

impl Wasapi {
    pub fn new() -> Self {
        //Use the default sample rate.
        Self::new_with_sample_rate(None)
    }

    pub fn new_with_sample_rate(sample_rate: Option<u32>) -> Self {
        unsafe {
            CoInitializeEx(ConcurrencyModel::MultiThreaded).unwrap();
            set_pro_audio_thread();

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
            }
        }
    }

    pub fn update_sample_rate(&mut self, sample_rate: u32) {
        let current_sample_rate = self.format.Format.nSamplesPerSec;

        if current_sample_rate != sample_rate {
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

    pub fn run<F: FnMut(&mut [u8])>(self, mut f: F) {
        unsafe {
            let (period, _) = self.client.GetDevicePeriod().unwrap();
            let period = Duration::from_nanos(period as u64 * 100);
            let mut last_event = Instant::now();

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
                    frames = self.fill(&mut f);

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
