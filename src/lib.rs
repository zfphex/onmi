#![allow(unused, static_mut_refs)]
pub mod decoder;
pub mod output;
pub mod thread_cell;

pub use decoder::*;
pub use output::*;
pub use thread_cell::*;

pub use wasapi::IMMDevice;

use std::path::Path;
use std::time::Duration;

//Scale the volume (0 - 100) down to something more reasonable to listen to.
//TODO: This should be configurable.
pub const VOLUME_REDUCTION: f32 = 75.0;
pub const UNKNOWN_TITLE: &str = "Unknown Title";
pub const UNKNOWN_ALBUM: &str = "Unknown Album";
pub const UNKNOWN_ARTIST: &str = "Unknown Artist";
pub const COMMON_SAMPLE_RATES: [u32; 13] = [
    5512, 8000, 11025, 16000, 22050, 32000, 44100, 48000, 64000, 88200, 96000, 176400, 192000,
];

//TODO: Seeking can cause race conditions. I think it's fine...?
static mut DECODER: Option<Symphonia> = None;

//Safety: The output device needs to be initalised before creating the output thread.
//OUTPUT is only read once on creation.
static mut OUTPUT: Option<Output> = None;

static mut VOLUME: ThreadCell<f32> = ThreadCell::new((15.0 / VOLUME_REDUCTION) * 0.5);
//TODO: Gain is never updated.
static mut GAIN: ThreadCell<f32> = ThreadCell::new(0.5);
static mut DURATION: ThreadCell<Duration> = ThreadCell::new(Duration::new(0, 0));
static mut STATE: ThreadCell<State> = ThreadCell::new(State::Stopped);
static mut ELAPSED: ThreadCell<Duration> = ThreadCell::new(Duration::new(0, 0));
static mut FINSIHED: ThreadCell<bool> = ThreadCell::new(false);

#[derive(PartialEq, Clone, Debug)]
pub enum State {
    Playing,
    Paused,
    Stopped,
}

pub struct Player {}

impl Player {
    pub fn new() -> Self {
        unsafe { OUTPUT = Some(Output::new(None)) };
        std::thread::spawn(move || {
            // eprintln!("PLAYBACK THREAD: {:?}", std::thread::current().id());
            unsafe { OUTPUT.as_mut().unwrap().run() };
        });

        Self {}
    }

    pub fn state(&self) -> State {
        unsafe { (*STATE).clone() }
    }

    pub fn elapsed(&self) -> Duration {
        unsafe { *ELAPSED }
    }

    pub fn duration(&self) -> Duration {
        unsafe { *DURATION }
    }

    pub fn toggle_playback(&self) {
        unsafe {
            if *STATE == State::Paused {
                *STATE = State::Playing;
            } else if *STATE == State::Playing {
                *STATE = State::Paused;
            }
        }
    }

    pub fn stop(&self) {
        unsafe { *STATE = State::Stopped };

        //This could cause UB.
        // unsafe { DECODER = None };
    }

    pub fn play_song(&self, path: impl AsRef<Path>, start: bool) -> Result<(), String> {
        let decoder = match Symphonia::new(&path) {
            Ok(s) => s,
            Err(e) => {
                return Err(format!(
                    "Failed to play: {}, Error: {e}",
                    path.as_ref().to_string_lossy()
                ));
            }
        };

        if start {
            unsafe { *STATE = State::Playing };
        } else {
            unsafe { *STATE = State::Paused };
        }

        unsafe { DECODER = Some(decoder) };

        Ok(())
    }

    pub fn play(&self) {
        unsafe { *STATE = State::Playing };
    }

    pub fn pause(&self) {
        unsafe { *STATE = State::Paused };
    }

    pub fn set_volume(&self, volume: u8) {
        unsafe { *VOLUME = volume as f32 / VOLUME_REDUCTION }
        // unsafe { PLAYBACK.volume = volume as f32 / VOLUME_REDUCTION }
    }

    pub fn volume_up(&self) {
        self.set_volume((self.volume() + 5).clamp(0, 100))
    }

    pub fn volume_down(&self) {
        dbg!(self.volume());
        self.set_volume((self.volume() - 5).clamp(0, 100))
    }

    pub fn volume(&self) -> u8 {
        (unsafe { *VOLUME } * VOLUME_REDUCTION) as u8
    }

    pub fn seek(&self, position: Duration) {
        if let Some(decoder) = unsafe { DECODER.as_mut() } {
            // self.elapsed = position;
            decoder.seek(position.as_secs_f32());
        }
    }

    //TODO: Removeme
    pub fn seek_forward(&self) {
        if let Some(decoder) = unsafe { DECODER.as_mut() } {
            let position = (decoder.elapsed().as_secs_f32() + 10.0).clamp(0.0, f32::MAX);
            // self.elapsed = Duration::from_secs_f32(position);
            decoder.seek(position);
        }
    }

    //TODO: Removeme
    pub fn seek_backward(&self) {
        if let Some(decoder) = unsafe { DECODER.as_mut() } {
            let position = (decoder.elapsed().as_secs_f32() - 10.0).clamp(0.0, f32::MAX);
            // self.elapsed = Duration::from_secs_f32(position);
            decoder.seek(position);
        }
    }

    pub fn devices(&self) -> Vec<(String, IMMDevice)> {
        if let Some(output) = unsafe { OUTPUT.as_ref() } {
            output.devices()
        } else {
            Vec::new()
        }
    }
    pub fn default_device(&self) -> Option<(String, IMMDevice)> {
        if let Some(output) = unsafe { OUTPUT.as_ref() } {
            Some(output.default_device())
        } else {
            None
        }
    }

    pub fn is_finished(&self) -> bool {
        unsafe { *FINSIHED }
    }

    pub fn set_output_device(&self, device: &str) {
        todo!()
    }
}
