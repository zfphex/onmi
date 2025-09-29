use std::io::ErrorKind;
use std::time::Duration;
use std::{fs::File, path::Path};
use symphonia::core::errors::Error;
use symphonia::core::formats::{FormatReader, Track};
use symphonia::core::units::TimeBase;
use symphonia::{
    core::{
        audio::SampleBuffer,
        codecs,
        formats::{FormatOptions, SeekMode, SeekTo},
        io::MediaSourceStream,
        meta::MetadataOptions,
        probe::Hint,
        units::Time,
    },
    default::get_probe,
};

use crate::*;

pub struct Symphonia {
    pub format_reader: Box<dyn FormatReader>,
    pub decoder: Box<dyn codecs::Decoder>,
    pub track: Track,
    pub error_count: u8,
    pub finished: bool,
    pub buffer: Option<SampleBuffer<f32>>,
    pub pos: usize,
    duration: u64,
    time_base: TimeBase,
}

impl Symphonia {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        let probed = get_probe().format(
            &Hint::default(),
            mss,
            &FormatOptions {
                prebuild_seek_index: false,
                seek_index_fill_rate: 20,
                enable_gapless: false,
            },
            &MetadataOptions::default(),
        )?;

        let track = probed.format.default_track().unwrap().to_owned();
        let n_frames = track.codec_params.n_frames.unwrap_or_default();
        let duration = track.codec_params.start_ts + n_frames;
        let time_base = track.codec_params.time_base.unwrap();
        let decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &codecs::DecoderOptions::default())?;

        unsafe {
            let time = time_base.calc_time(duration);
            *DURATION = Duration::from_secs(time.seconds) + Duration::from_secs_f64(time.frac);
        }

        Ok(Self {
            format_reader: probed.format,
            decoder,
            track,
            duration,
            error_count: 0,
            finished: false,
            buffer: None,
            pos: 0,
            time_base,
        })
    }

    pub fn sample_rate(&self) -> u32 {
        self.track.codec_params.sample_rate.unwrap()
    }

    pub fn seek(&mut self, pos: f32) {
        let pos = Duration::from_secs_f32(pos);

        //TODO: This is pretty scuffed and might break under certain conditions.
        if unsafe { pos > *DURATION } {
            self.finished = true;
            return;
        }

        //Ignore errors.
        let _ = self.format_reader.seek(
            SeekMode::Coarse,
            SeekTo::Time {
                time: Time::new(pos.as_secs(), pos.subsec_nanos() as f64 / 1_000_000_000.0),
                track_id: None,
            },
        );

        unsafe { ELAPSED = pos };
    }

    pub fn next_sample(&mut self) -> Option<f32> {
        if self.buffer.is_none() {
            if let Some(packet) = self.next_packet() {
                self.buffer = Some(packet);
            }
        }

        if let Some(buffer) = &self.buffer {
            if self.pos < buffer.samples().len() {
                let sample = buffer.samples()[self.pos];
                self.pos += 1;
                return Some(sample);
            } else {
                self.pos = 0;
                self.buffer = None;
                return self.next_sample();
            }
        }

        return None;
    }

    pub fn next_packet(&mut self) -> Option<SampleBuffer<f32>> {
        if self.error_count > 2 || self.finished {
            return None;
        }

        let next_packet = match self.format_reader.next_packet() {
            Ok(next_packet) => {
                self.error_count = 0;
                next_packet
            }
            Err(err) => match err {
                Error::IoError(e) if e.kind() == ErrorKind::UnexpectedEof => {
                    //Just in case my 250ms addition is not enough.
                    if unsafe { ELAPSED } + Duration::from_secs(1) > unsafe { *DURATION } {
                        self.finished = true;
                        return None;
                    } else {
                        self.error_count += 1;
                        return self.next_packet();
                    }
                }
                _ => {
                    // gonk_core::log!("{}", err);
                    self.error_count += 1;
                    return self.next_packet();
                }
            },
        };

        let elapsed = next_packet.ts();
        let time = self.time_base.calc_time(elapsed);
        unsafe { ELAPSED = Duration::from_secs(time.seconds) + Duration::from_secs_f64(time.frac) };

        if unsafe { ELAPSED > *DURATION } {
            self.finished = true;
            return None;
        }

        match self.decoder.decode(&next_packet) {
            Ok(decoded) => {
                let mut buffer =
                    SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());
                buffer.copy_interleaved_ref(decoded);
                Some(buffer)
            }
            Err(_) => {
                self.error_count += 1;
                self.next_packet()
            }
        }
    }
}
