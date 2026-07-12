use crate::{PlayerState, State};
use std::path::PathBuf;
use std::sync::atomic::Ordering::Relaxed;
use std::time::Duration;
use std::{fs::File, path::Path};
use symphonia::core::formats::{FormatReader, Track, TrackType};
use symphonia::core::units::TimeBase;
use symphonia::{
    core::{
        codecs::audio::{AudioDecoder, AudioDecoderOptions},
        formats::{FormatOptions, SeekMode, SeekTo, probe::Hint},
        io::MediaSourceStream,
        meta::MetadataOptions,
        units::Time,
    },
    default::get_probe,
};

pub struct Symphonia {
    pub format_reader: Box<dyn FormatReader>,
    pub decoder: Box<dyn AudioDecoder>,
    pub track: Track,
    pub error_count: u8,
    pub finished: bool,
    pub buffer: Vec<f32>,
    pub buffer_len: usize,
    pub pos: usize,
    pub sample_rate: u32,
    pub channels: u32,
    pub time_base: TimeBase,
    pub path: PathBuf,
    pub duration: Duration,
}

impl Symphonia {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(path.as_ref())?;
        let path = path.as_ref().to_path_buf();
        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        let format_reader = get_probe().probe(
            &Hint::new(),
            mss,
            FormatOptions::default()
                .prebuild_seek_index(false)
                .seek_index_fill_period_ms(20),
            MetadataOptions::default(),
        )?;
        let track = format_reader
            .default_track(TrackType::Audio)
            .unwrap()
            .to_owned();
        let time_base = track.time_base.unwrap();
        let duration = track
            .duration
            .or(track.num_frames.map(symphonia::core::units::Duration::new))
            .and_then(|duration| duration.timestamp_from(symphonia::core::units::Timestamp::ZERO))
            .map(|duration_ts| time_base.calc_time_saturating(duration_ts))
            .map(|time| Duration::from_nanos(time.as_nanos() as u64))
            .unwrap_or_default();
        let codec_params = track
            .codec_params
            .as_ref()
            .and_then(|params| params.audio())
            .unwrap();
        let decoder = symphonia::default::get_codecs()
            .make_audio_decoder(codec_params, &AudioDecoderOptions::default())?;

        Ok(Self {
            sample_rate: codec_params.sample_rate.unwrap(),
            channels: codec_params
                .channels
                .as_ref()
                .map(|c| c.count() as u32)
                .unwrap_or(2),
            format_reader,
            decoder,
            track,
            error_count: 0,
            finished: false,
            buffer: Vec::new(),
            buffer_len: 0,
            pos: 0,
            time_base,
            path,
            duration,
        })
    }

    pub fn seek(&mut self, pos: Duration, state: &PlayerState) {
        if pos >= self.duration {
            self.finished = true;
            state.mark_finished();
            state
                .elapsed
                .store(self.duration.as_nanos() as u64, Relaxed);
            return;
        }

        if self
            .format_reader
            .seek(
                SeekMode::Coarse,
                SeekTo::Time {
                    time: Time::from_nanos_u64(pos.as_nanos() as u64),
                    track_id: None,
                },
            )
            .is_ok()
        {
            self.decoder.reset();
            self.buffer_len = 0;
            self.pos = 0;
            self.finished = false;
            state.finished.store(false, Relaxed);
        }

        state.elapsed.store(pos.as_nanos() as u64, Relaxed);
    }

    pub fn next_sample(&mut self, state: &PlayerState) -> Option<f32> {
        if self.pos >= self.buffer_len {
            if !self.fill_packet(state) {
                return None;
            }
        }

        let sample = self.buffer[self.pos];
        self.pos += 1;
        Some(sample)
    }

    fn fill_packet(&mut self, state: &PlayerState) -> bool {
        if self.error_count > 2 || self.finished {
            return false;
        }

        let next_packet = match self.format_reader.next_packet() {
            Ok(Some(next_packet)) => {
                self.error_count = 0;
                next_packet
            }
            Ok(None) => {
                self.finished = true;
                return false;
            }
            Err(_) => {
                self.error_count += 1;
                return self.fill_packet(state);
            }
        };

        if next_packet.track_id != self.track.id {
            return self.fill_packet(state);
        }

        if let Some(time) = self.time_base.calc_time(next_packet.pts) {
            let time = time.as_secs_f64().max(0.0);
            let elapsed = Duration::from_secs_f64(time);
            if elapsed > self.duration {
                self.finished = true;
                return false;
            }

            if state.state.load(Relaxed) != State::Stopped as u8 {
                state.elapsed.store(elapsed.as_nanos() as u64, Relaxed);
            }
        } else {
            unreachable!("Packet is timeless, one cannot be timeless...? Only me 🗿")
        }

        match self.decoder.decode(&next_packet) {
            Ok(decoded) => {
                let n = decoded.samples_interleaved();
                if self.buffer.len() < n {
                    self.buffer.resize(n, 0.0);
                }
                decoded.copy_to_slice_interleaved(&mut self.buffer[..n]);
                self.buffer_len = n;
                self.pos = 0;
                true
            }
            Err(_) => {
                self.error_count += 1;
                self.fill_packet(state)
            }
        }
    }
}
