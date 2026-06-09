#![allow(static_mut_refs)]
pub mod decoder;
pub mod metadata;
pub mod output;
pub mod thread_cell;

pub use decoder::*;
pub use metadata::*;
pub use output::*;
pub use thread_cell::*;

pub use wasapi::IMMDevice;

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};
use std::time::Duration;

//Scale the volume (0 - 100) down to something more reasonable to listen to.
//TODO: This should be configurable.
pub const VOLUME_REDUCTION: f32 = 75.0;
pub const UNKNOWN_TITLE: &str = "UnknownTitle";
pub const UNKNOWN_ALBUM: &str = "Unknown Album";
pub const UNKNOWN_ARTIST: &str = "Unknown Artist";
pub const COMMON_SAMPLE_RATES: [u32; 13] = [
    5512, 8000, 11025, 16000, 22050, 32000, 44100, 48000, 64000, 88200, 96000, 176400, 192000,
];

//If this is `Some` the playback thread will take it and move it into the local thread.
//This can write from both threads but _should_ never double write.
static mut NEXT_DECODER: Option<Symphonia> = None;
static mut NEXT_OUTPUT: Option<Output> = None;

//Should be fine
static mut VOLUME: ThreadCell<f32> = ThreadCell::new((15.0 / VOLUME_REDUCTION) * 0.5);
static mut GAIN: ThreadCell<f32> = ThreadCell::new(0.5);
static mut DURATION: ThreadCell<Duration> = ThreadCell::new(Duration::new(0, 0));
static mut STATE: ThreadCell<State> = ThreadCell::new(State::Stopped);

//There is some weird behaviour after the playback thread stops.
//Ideally something else would start the thread up
//and this would only be written on the playback thread.
static FINISHED: AtomicBool = AtomicBool::new(false);

//Seeking can change this value from any thread.
static ELAPSED: AtomicU64 = AtomicU64::new(0);

//Just use the max value as None/not seeking.
static SEEK: AtomicU64 = AtomicU64::new(u64::MAX);

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
    // _phantom: std::marker::PhantomData<*const ()>,

    //We only use this to update the sample rate
    //when the Output isn't some yet.
    pub device: Device,
    pub current_song_sample_rate: Option<u32>,
}

impl Player {
    pub fn new(device: Device) -> Self {
        //TODO: Currently the library cannot handle changing output devices.
        //Windows requires that output devices be destroyed (sometimes) to change sample rates.
        //Not sure how to handle swapping output devices.
        let d = device.clone();
        std::thread::spawn(move || {
            output::run(new_output(d, None));
        });

        Self {
            device,
            current_song_sample_rate: None,
            // _phantom: std::marker::PhantomData,
        }
    }

    pub fn state(&self) -> State {
        unsafe { (*STATE).clone() }
    }

    pub fn elapsed(&self) -> Duration {
        Duration::from_nanos(ELAPSED.load(Relaxed))
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

    pub fn play_song(
        &mut self,
        path: impl AsRef<std::path::Path>,
        //Set how the volume should be scaled.
        replay_gain: Option<f32>,
        //Can start the player paused.
        start_playback: bool,
    ) -> Result<(), String> {
        let decoder = match Symphonia::new(&path) {
            Ok(s) => s,
            Err(e) => {
                return Err(format!(
                    "Failed to play: {}, Error: {e}",
                    path.as_ref().to_string_lossy()
                ));
            }
        };

        //Update the sample rate if it's different, or it hasn't been set yet.
        if self.current_song_sample_rate.unwrap_or_default() != decoder.sample_rate {
            unsafe {
                NEXT_OUTPUT = Some(new_output(self.device.clone(), Some(decoder.sample_rate)))
            };
        }

        self.current_song_sample_rate = Some(decoder.sample_rate);

        if start_playback {
            unsafe { *STATE = State::Playing };
        } else {
            unsafe { *STATE = State::Paused };
        }

        //Default to half volume, not sure if this is a good deafult.
        //Some songs don't have replay gain values and this reduces the volume jump between songs.
        unsafe { *GAIN = replay_gain.unwrap_or(0.5) }
        unsafe { NEXT_DECODER = Some(decoder) };

        //Since the output thread will stop and set this to true.
        //Tell the output thread that a new song has started.
        //Do not remove this.
        FINISHED.store(false, Relaxed);

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
    }

    pub fn volume_up(&self) {
        self.set_volume((self.volume() + 5).clamp(0, 100))
    }

    pub fn volume_down(&self) {
        self.set_volume((self.volume().saturating_sub(5)).clamp(0, 100))
    }

    pub fn volume(&self) -> u8 {
        (unsafe { *VOLUME } * VOLUME_REDUCTION) as u8
    }

    pub fn seek_to(&self, position: Duration) {
        SEEK.store(position.as_nanos() as u64, Relaxed);
    }

    pub fn seek_forward(&self, secs: f32) {
        let duration = self.elapsed() + Duration::from_secs_f32(secs);
        SEEK.store(duration.as_nanos() as u64, Relaxed);
    }

    pub fn seek_backward(&self, secs: f32) {
        let duration = self.elapsed().saturating_sub(Duration::from_secs_f32(secs));
        SEEK.store(duration.as_nanos() as u64, Relaxed);
    }

    pub fn is_finished(&self) -> bool {
        FINISHED.load(Relaxed)
    }

    pub fn set_output_device(&mut self, device: Device) {
        self.device = device.clone();
        unsafe { NEXT_OUTPUT = Some(new_output(device, self.current_song_sample_rate)) };
    }
}
