pub mod decoder;
pub mod engine;
pub mod metadata;
pub mod state;

pub use decoder::*;
pub use engine::*;
pub use metadata::*;
pub use state::*;

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;

use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;
use std::time::Duration;

pub const VOLUME_REDUCTION: f32 = 75.0;
pub const UNKNOWN_TITLE: &str = "UnknownTitle";
pub const UNKNOWN_ALBUM: &str = "Unknown Album";
pub const UNKNOWN_ARTIST: &str = "Unknown Artist";
pub const COMMON_SAMPLE_RATES: [u32; 13] = [
    5512, 8000, 11025, 16000, 22050, 32000, 44100, 48000, 64000, 88200, 96000, 176400, 192000,
];

#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum State {
    Playing = 0,
    Paused = 1,
    Stopped = 2,
}

pub struct Player {
    pub state: Arc<PlayerState>,
    pub device: Device,
    pub current_song_sample_rate: Option<u32>,
}

impl Player {
    pub fn new(device: Device) -> Self {
        let state = PlayerState::global();
        let d = device.clone();
        let thread_state = Arc::clone(&state);
        std::thread::spawn(move || {
            run_output(thread_state, new_output(d, None));
        });

        Self {
            state,
            device,
            current_song_sample_rate: None,
        }
    }

    pub fn toggle_playback(&self) {
        let state = self.state.state.load(Relaxed);
        if state == State::Paused as u8 {
            self.state.state.store(State::Playing as u8, Relaxed);
        } else if state == State::Playing as u8 {
            self.state.state.store(State::Paused as u8, Relaxed);
        }
    }

    pub fn stop(&self) {
        self.state.state.store(State::Stopped as u8, Relaxed);
    }

    pub fn play_song(
        &mut self,
        path: impl AsRef<std::path::Path>,
        replay_gain: Option<f32>,
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

        if self.current_song_sample_rate.unwrap_or_default() != decoder.sample_rate {
            self.state
                .pending_output
                .publish(new_output(self.device.clone(), Some(decoder.sample_rate)));
        }

        self.current_song_sample_rate = Some(decoder.sample_rate);

        self.state.state.store(State::Stopped as u8, Relaxed);
        self.state.elapsed.store(0, Relaxed);
        self.state
            .duration
            .store(decoder.duration.as_nanos() as u64, Relaxed);
        self.state
            .gain
            .store(replay_gain.unwrap_or(0.5).to_bits(), Relaxed);
        self.state.decoder_pending.store(true, Relaxed);
        self.state.pending_decoder.publish(decoder);

        if start_playback {
            self.state.state.store(State::Playing as u8, Relaxed);
        } else {
            self.state.state.store(State::Paused as u8, Relaxed);
        }

        Ok(())
    }

    pub fn play(&self) {
        self.state.state.store(State::Playing as u8, Relaxed);
    }

    pub fn pause(&self) {
        self.state.state.store(State::Paused as u8, Relaxed);
    }

    pub fn set_volume(&self, volume: u8) {
        self.state
            .volume
            .store((volume.clamp(0, 100) as f32 / VOLUME_REDUCTION).to_bits(), Relaxed);
    }

    pub fn volume_up(&self) {
        let volume = (f32::from_bits(self.state.volume.load(Relaxed)) * VOLUME_REDUCTION) as u8;
        self.set_volume((volume + 5).clamp(0, 100));
    }

    pub fn volume_down(&self) {
        let volume = (f32::from_bits(self.state.volume.load(Relaxed)) * VOLUME_REDUCTION) as u8;
        self.set_volume(volume.saturating_sub(5).clamp(0, 100));
    }

    pub fn seek_to(&self, position: Duration) {
        self.state
            .seek
            .store(position.as_nanos() as u64, Relaxed);
    }

    pub fn seek_forward(&self, secs: f32) {
        let elapsed = Duration::from_nanos(self.state.elapsed.load(Relaxed));
        self.state
            .seek
            .store((elapsed + Duration::from_secs_f32(secs)).as_nanos() as u64, Relaxed);
    }

    pub fn seek_backward(&self, secs: f32) {
        let elapsed = Duration::from_nanos(self.state.elapsed.load(Relaxed));
        self.state.seek.store(
            elapsed
                .saturating_sub(Duration::from_secs_f32(secs))
                .as_nanos() as u64,
            Relaxed,
        );
    }

    pub fn set_output_device(&mut self, device: Device) {
        self.device = device.clone();
        self.state
            .pending_output
            .publish(new_output(device, self.current_song_sample_rate));
    }
}
