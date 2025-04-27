pub mod audio;
pub mod decoder;

pub use audio::*;
pub use decoder::*;

use std::{path::Path, time::Duration};

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

pub struct Player {
    volume: f32,
    pub gain: f32,
    pub decoder: Option<Symphonia>,
    pub elapsed: Duration,
    pub duration: Duration,
    pub paused: bool,
    pub stopped: bool,
}

impl Player {
    pub const fn new() -> Self {
        Self {
            volume: 15.0 / VOLUME_REDUCTION,
            gain: 0.5,
            decoder: None,
            elapsed: Duration::new(0, 0),
            duration: Duration::new(0, 0),
            paused: true,
            stopped: false,
        }
    }

    pub fn toggle_playback(&mut self) {
        self.paused = !self.paused;
    }

    pub fn stop(&mut self) {
        self.stopped = true;
        self.decoder = None;
    }

    pub fn play_song(&mut self, path: impl AsRef<Path>) -> Result<(), String> {
        self.decoder = match Symphonia::new(&path) {
            Ok(s) => Some(s),
            Err(e) => {
                return Err(format!(
                    "Failed to play: {}, Error: {e}",
                    path.as_ref().to_string_lossy()
                ));
            }
        };

        Ok(())
    }

    pub fn play(&mut self) {
        self.paused = false;
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn set_volume(&mut self, volume: u8) {
        self.volume = volume as f32 / VOLUME_REDUCTION;
    }

    pub fn volume(&mut self) -> u8 {
        (self.volume * VOLUME_REDUCTION) as u8
    }

    //Position is a percentage between (1 - 100).
    pub fn seek(&mut self, position: f32) {
        if let Some(decoder) = &mut self.decoder {
            self.elapsed = Duration::from_secs_f32(position);
            decoder.seek(position);
        }
    }

    pub fn seek_forward(&mut self) {
        // info!(
        //     "Seeking {} / {}",
        //     sym.elapsed().as_secs_f32() + 10.0,
        //     sym.duration().as_secs_f32()
        // );

        if let Some(decoder) = &mut self.decoder {
            let position = (decoder.elapsed().as_secs_f32() + 10.0).clamp(0.0, f32::MAX);
            self.elapsed = Duration::from_secs_f32(position);
            decoder.seek(position);
        }
    }

    pub fn seek_backwards(&mut self) {
        // info!(
        //     "Seeking {} / {}",
        //     sym.elapsed().as_secs_f32() - 10.0,
        //     sym.duration().as_secs_f32()
        // );

        if let Some(decoder) = &mut self.decoder {
            let position = (decoder.elapsed().as_secs_f32() - 10.0).clamp(0.0, f32::MAX);
            self.elapsed = Duration::from_secs_f32(position);
            decoder.seek(position);
        }
    }
}
