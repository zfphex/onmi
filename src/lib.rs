#![allow(unused, static_mut_refs)]
pub mod decoder;
pub mod output;
pub mod thread_cell;

pub use decoder::*;
pub use output::*;
pub use thread_cell::*;

pub use wasapi::IMMDevice;

use std::marker::PhantomData;
use std::path::Path;
use std::thread::JoinHandle;
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
static mut FINSIHED: ThreadCell<bool> = ThreadCell::new(false);

//TODO: Seeking is causing a lot of issues and should be reworked.
//Can cause race conditions while the decoder thread readsl packets and the user tries to seek.
// static mut ELAPSED: ThreadCell<Duration> = ThreadCell::new(Duration::new(0, 0));
static mut ELAPSED: Duration = Duration::new(0, 0);

#[derive(PartialEq, Clone, Debug)]
pub enum State {
    Playing,
    Paused,
    Stopped,
}

#[derive(Debug, Clone)]
pub struct Device {
    pub imm: IMMDevice,
    pub name: String,
}

unsafe impl Send for Device {}
unsafe impl Sync for Device {}

pub struct Player {
    //Force !Send + !Sync
    //User's should not access the player from multiple threads.
    //TODO: Actually I want to defer creation of the player onto a new thread since it's really slow...
    //I feel like the language doesn't allow me to prevent misuse here 🤔.
    // _phantom: PhantomData<*const ()>,
}

impl Player {
    pub fn new(device: Device) -> Self {
        std::thread::spawn(move || {
            unsafe {
                // OUTPUT = Some(Output::new(device, None));
                // OUTPUT.as_mut().unwrap().run();
                Output::new(device, None).run();
            }
        });

        Self {}
    }

    pub fn state(&self) -> State {
        unsafe { (*STATE).clone() }
    }

    pub fn elapsed(&self) -> Duration {
        unsafe { ELAPSED }
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
            let position = (unsafe { ELAPSED }.as_secs_f32() + 10.0).clamp(0.0, f32::MAX);
            // self.elapsed = Duration::from_secs_f32(position);
            decoder.seek(position);
        }
    }

    //TODO: Removeme
    pub fn seek_backward(&self) {
        if let Some(decoder) = unsafe { DECODER.as_mut() } {
            let position = (unsafe { ELAPSED }.as_secs_f32() - 10.0).clamp(0.0, f32::MAX);
            // self.elapsed = Duration::from_secs_f32(position);
            decoder.seek(position);
        }
    }

    pub fn is_finished(&self) -> bool {
        unsafe { *FINSIHED }
    }

    pub fn set_output_device(&self, device: &str) {
        todo!()
    }
}
