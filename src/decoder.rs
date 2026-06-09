use std::path::PathBuf;
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

use crate::*;

pub struct Symphonia {
    pub format_reader: Box<dyn FormatReader>,
    pub decoder: Box<dyn AudioDecoder>,
    pub track: Track,
    pub error_count: u8,
    pub finished: bool,
    pub buffer: Option<Vec<f32>>,
    pub pos: usize,
    pub sample_rate: u32,
    pub time_base: TimeBase,
    pub path: PathBuf,
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

        //Update the elapsed so that it's never out of sync with the duration.
        //I had a one frame visual bug where the duration was updated before the elapsed was reset.
        //The decoder will never write when the state is stopped.
        unsafe { *STATE = State::Stopped }
        ELAPSED.store(0, Relaxed);
        unsafe { *DURATION = duration };

        Ok(Self {
            sample_rate: codec_params.sample_rate.unwrap(),
            format_reader,
            decoder,
            track,
            error_count: 0,
            finished: false,
            buffer: None,
            pos: 0,
            time_base,
            path,
        })
    }

    pub fn seek(&mut self, pos: Duration) {
        //TODO: This is pretty scuffed and might break under certain conditions.
        if unsafe { pos > *DURATION } {
            self.finished = true;
            return;
        }

        //Ignore errors.
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
            self.buffer = None;
            self.pos = 0;
            self.finished = false;
        }

        ELAPSED.store(pos.as_nanos() as u64, Relaxed);
    }

    pub fn next_sample(&mut self) -> Option<f32> {
        if self.buffer.is_none() {
            let Some(buffer) = self.next_packet() else {
                // The player must have finished.
                return None;
            };

            self.buffer = Some(buffer);
        }

        if let Some(buffer) = &self.buffer {
            if self.pos < buffer.len() {
                let sample = buffer[self.pos];
                self.pos += 1;
                return Some(sample);
            } else {
                self.pos = 0;
                self.buffer = None;
                return self.next_sample();
            }
        }

        unreachable!()
    }

    pub fn next_packet(&mut self) -> Option<Vec<f32>> {
        if self.error_count > 2 || self.finished {
            return None;
        }

        let next_packet = match self.format_reader.next_packet() {
            Ok(Some(next_packet)) => {
                self.error_count = 0;
                next_packet
            }
            Ok(None) => {
                return None;
            }
            Err(_) => {
                self.error_count += 1;
                return self.next_packet();
            }
        };

        if next_packet.track_id != self.track.id {
            return self.next_packet();
        }

        if let Some(time) = self.time_base.calc_time(next_packet.pts) {
            //IDK sometimes we get negative timestamps I guess 🙄.
            let time = time.as_secs_f64().max(0.0);
            let duration = Duration::from_secs_f64(time);
            if unsafe { duration > *DURATION } {
                self.finished = true;
                return None;
            }

            //The elapsed time is reset to zero when playing a new song.
            //Never overwrite the value when stopped.
            if unsafe { *STATE != State::Stopped } {
                ELAPSED.store(duration.as_nanos() as u64, Relaxed);
            }
        } else {
            unreachable!("Packet is timeless, one cannot be timeless...? Only me 🗿")
        }

        match self.decoder.decode(&next_packet) {
            Ok(decoded) => {
                //TODO: Don't allocate here.
                let mut buffer = vec![0.0; decoded.samples_interleaved()];
                decoded.copy_to_slice_interleaved(&mut buffer);
                Some(buffer)
            }
            Err(_) => {
                self.error_count += 1;
                self.next_packet()
            }
        }
    }
}
