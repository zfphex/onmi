#![allow(unused, static_mut_refs)]
pub mod audio;
pub mod decoder;

pub use audio::*;
pub use decoder::*;

use std::path::Path;

//Probably not good to re-export this.
pub use std::time::Duration;

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
    pub decoder: Option<Symphonia>,
    pub output: Option<WasapiOutput>,
    pub volume: f32,
    pub gain: f32,
}

impl PlaybackThread {
    pub const fn new() -> Self {
        Self {
            decoder: None,
            output: None,
            volume: (15.0 / VOLUME_REDUCTION) * 0.5,
            gain: 0.5,
        }
    }
    pub fn update_decoder(&mut self, symphonia: Symphonia) {
        //TODO: Update gain.
        // self.gain = symphonia
        self.decoder = Some(symphonia);
    }
}

static mut PLAYBACK: PlaybackThread = PlaybackThread::new();

pub struct Player {
    elapsed: Duration,
    paused: bool,
    stopped: bool,
}

impl Player {
    pub fn new() -> Self {
        let f = std::thread::spawn(|| {
            unsafe { PLAYBACK.output = Some(WasapiOutput::new()) };
            let channels = unsafe { PLAYBACK.output.as_mut().unwrap().channels() } as usize;

            move |buffer: &mut [u8]| {
                for bytes in buffer.chunks_mut(std::mem::size_of::<f32>() * channels) {
                    //Only allow for stereo outputs.
                    for c in 0..channels.max(2) {
                        unsafe {
                            if let Some(ref mut decoder) = PLAYBACK.decoder {
                                let sample = decoder.next_sample();
                                let value =
                                    (sample * PLAYBACK.volume * PLAYBACK.gain).to_le_bytes();
                                bytes[c * 4..c * 4 + 4].copy_from_slice(&value);
                            }
                        }
                    }
                }
            }
        })
        .join()
        .unwrap();

        //Wait for the thread to finish so that immediate pauses will work.
        std::thread::spawn(move || {
            unsafe { PLAYBACK.output.as_mut().unwrap().run(f) };
        });

        Self {
            elapsed: Duration::new(0, 0),
            paused: true,
            stopped: false,
        }
    }

    /// TODO: Fixme
    /// This allows for the player to be slightly out of sync with the playback thread.
    /// If the user askes to seek and then reads the elapsed time immediately after.
    /// It will not be up to date, because seeking can take time to process.
    pub fn elapsed(&mut self) -> Duration {
        let elapsed = self.elapsed;
        if let Some(decoder) = unsafe { &mut PLAYBACK.decoder } {
            self.elapsed = decoder.elapsed();
        }
        return elapsed;
    }

    pub fn duration(&self) -> Option<Duration> {
        if let Some(decoder) = unsafe { &mut PLAYBACK.decoder } {
            return Some(decoder.duration());
        }

        None
    }

    pub fn toggle_playback(&mut self) {
        self.paused = !self.paused;
    }

    pub fn stop(&mut self) {
        self.stopped = true;
        unsafe { PLAYBACK.decoder = None };
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

        unsafe { PLAYBACK.update_decoder(decoder) };

        Ok(())
    }

    pub fn play(&mut self) {
        self.paused = false;

        if let Some(output) = unsafe { &mut PLAYBACK.output } {
            output.play();
        }
    }

    pub fn pause(&mut self) {
        self.paused = true;

        if let Some(output) = unsafe { &mut PLAYBACK.output } {
            output.pause();
        }
    }

    pub fn set_volume(&mut self, volume: u8) {
        unsafe { PLAYBACK.volume = volume as f32 / VOLUME_REDUCTION }
    }

    pub fn volume(&mut self) -> u8 {
        (unsafe { PLAYBACK.volume } * VOLUME_REDUCTION) as u8
    }

    pub fn seek(&mut self, position: Duration) {
        if let Some(decoder) = unsafe { &mut PLAYBACK.decoder } {
            self.elapsed = position;
            decoder.seek(position.as_secs_f32());
        }
    }

    pub fn seek_forward(&mut self) {
        if let Some(decoder) = unsafe { &mut PLAYBACK.decoder } {
            let position = (decoder.elapsed().as_secs_f32() + 10.0).clamp(0.0, f32::MAX);
            self.elapsed = Duration::from_secs_f32(position);
            decoder.seek(position);
        }
    }

    pub fn seek_backward(&mut self) {
        if let Some(decoder) = unsafe { &mut PLAYBACK.decoder } {
            let position = (decoder.elapsed().as_secs_f32() - 10.0).clamp(0.0, f32::MAX);
            self.elapsed = Duration::from_secs_f32(position);
            decoder.seek(position);
        }
    }
}
