#![allow(unused, static_mut_refs)]
pub mod audio;
pub mod decoder;
pub mod thread_cell;

pub use audio::*;
pub use decoder::*;
pub use thread_cell::*;

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

pub struct PlaybackThread {
    pub output: Option<WasapiOutput>,
    pub volume: f32,
    pub gain: f32,
}

impl PlaybackThread {
    pub const fn new() -> Self {
        Self {
            output: None,
            volume: (15.0 / VOLUME_REDUCTION) * 0.5,
            gain: 0.5,
        }
    }
}

static mut PLAYBACK: PlaybackThread = PlaybackThread::new();
static mut DECODER: Option<Symphonia> = None;
static mut STATE: State = State::Stopped;
static mut ELAPSED: Duration = Duration::new(0, 0);
static mut DURATION: Option<Duration> = None;

#[derive(PartialEq)]
pub enum State {
    Playing,
    Paused,
    Stopped,
}

pub struct Player {}

impl Player {
    pub fn new() -> Self {
        unsafe { PLAYBACK.output = Some(WasapiOutput::new(None)) };
        std::thread::spawn(move || {
            eprintln!("PLAYBACK THREAD: {:?}", std::thread::current().id());
            unsafe { PLAYBACK.output.as_mut().unwrap().run() };
        });

        Self {}
    }

    pub fn elapsed(&mut self) -> Duration {
        unsafe { ELAPSED }
    }

    pub fn duration(&self) -> Option<Duration> {
        unsafe { DURATION }
    }

    pub fn toggle_playback(&mut self) {
        unsafe {
            if STATE == State::Paused {
                STATE = State::Playing;
            } else if STATE == State::Playing {
                STATE = State::Paused;
            }
        }
    }

    pub fn stop(&mut self) {
        unsafe { STATE = State::Stopped };
        unsafe { DECODER = None };
    }

    pub fn play_song(&mut self, path: impl AsRef<Path>) -> Result<(), String> {
        let decoder = match Symphonia::new(&path) {
            Ok(s) => s,
            Err(e) => {
                return Err(format!(
                    "Failed to play: {}, Error: {e}",
                    path.as_ref().to_string_lossy()
                ));
            }
        };

        unsafe { STATE = State::Playing };
        unsafe { DECODER = Some(decoder) };

        Ok(())
    }

    pub fn play(&mut self) {
        unsafe { STATE = State::Playing };
    }

    pub fn pause(&mut self) {
        unsafe { STATE = State::Paused };
    }

    pub fn set_volume(&mut self, volume: u8) {
        unsafe { PLAYBACK.volume = volume as f32 / VOLUME_REDUCTION }
    }

    pub fn volume(&mut self) -> u8 {
        (unsafe { PLAYBACK.volume } * VOLUME_REDUCTION) as u8
    }

    pub fn seek(&mut self, position: Duration) {
        if let Some(decoder) = unsafe { DECODER.as_mut() } {
            // self.elapsed = position;
            decoder.seek(position.as_secs_f32());
        }
    }

    pub fn seek_forward(&mut self) {
        if let Some(decoder) = unsafe { DECODER.as_mut() } {
            let position = (decoder.elapsed().as_secs_f32() + 10.0).clamp(0.0, f32::MAX);
            // self.elapsed = Duration::from_secs_f32(position);
            decoder.seek(position);
        }
    }

    pub fn seek_backward(&mut self) {
        if let Some(decoder) = unsafe { DECODER.as_mut() } {
            let position = (decoder.elapsed().as_secs_f32() - 10.0).clamp(0.0, f32::MAX);
            // self.elapsed = Duration::from_secs_f32(position);
            decoder.seek(position);
        }
    }
}
