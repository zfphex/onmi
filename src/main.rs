#![allow(static_mut_refs, unused, unsafe_op_in_unsafe_fn)]
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use crossbeam_queue::ArrayQueue;
use mini::{defer_results, profile};
use onmi::*;
use symphonia::core::audio::SampleBuffer;
use wasapi::*;

const COMMON_SAMPLE_RATES: [u32; 13] = [
    5512, 8000, 11025, 16000, 22050, 32000, 44100, 48000, 64000, 88200, 96000, 176400, 192000,
];

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Device {
    pub inner: IMMDevice,
    pub name: String,
}

pub unsafe fn create_wasapi(
    device: &Device,
    sample_rate: Option<u32>,
) -> (
    IAudioClient,
    IAudioRenderClient,
    WAVEFORMATEXTENSIBLE,
    *mut c_void,
) {
    let client: IAudioClient = device.inner.Activate(ExecutionContext::All).unwrap();
    let mut format =
        (client.GetMixFormat().unwrap() as *const _ as *const WAVEFORMATEXTENSIBLE).read();

    if format.Format.nChannels < 2 {
        todo!("Support mono devices.");
    }

    //Update format to desired sample rate.
    if let Some(sample_rate) = sample_rate {
        assert!(COMMON_SAMPLE_RATES.contains(&sample_rate));
        format.Format.nSamplesPerSec = sample_rate;
        format.Format.nAvgBytesPerSec = sample_rate * format.Format.nBlockAlign as u32;
    }

    let (default, _min) = client.GetDevicePeriod().unwrap();

    client
        .Initialize(
            ShareMode::Shared,
            AUDCLNT_STREAMFLAGS_EVENTCALLBACK
                | AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM
                | AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY,
            default,
            default,
            &format as *const _ as *const WAVEFORMATEX,
            None,
        )
        .unwrap();

    let event = CreateEventA(core::ptr::null_mut(), 0, 0, core::ptr::null_mut());
    assert!(!event.is_null());
    client.SetEventHandle(event as isize).unwrap();

    let render_client: IAudioRenderClient = client.GetService().unwrap();
    client.Start().unwrap();

    (client, render_client, format, event)
}

pub struct PacketRequest {
    pub sym: Symphonia,
    pub buffer: Option<SampleBuffer<f32>>,
    pub pos: usize,
}

impl PacketRequest {
    pub fn next_sample(&mut self) -> f32 {
        if self.buffer.is_none() {
            if let Some(packet) = self.sym.next_packet() {
                self.buffer = Some(packet);
            }
        }

        if let Some(buffer) = &self.buffer {
            if self.pos < buffer.samples().len() {
                let sample = buffer.samples()[self.pos];
                self.pos += 1;
                return sample;
            } else {
                self.pos = 0;
                self.buffer = None;
                return self.next_sample();
            }
        }

        panic!("No more samples");
    }
}

//TODO: Warn user when packets are dropped.
fn main() {
    defer_results!();
    unsafe {
        CoInitializeEx(ConcurrencyModel::MultiThreaded).unwrap();
        let enumerator = IMMDeviceEnumerator::new().unwrap();
        let device = enumerator
            .GetDefaultAudioEndpoint(DataFlow::Render, Role::Console)
            .unwrap();
        let device = Device {
            name: device.name(),
            inner: device,
        };
        let (mut audio, mut render, mut format, mut event) = create_wasapi(&device, None);
        let mut block_align = format.Format.nBlockAlign as u32;
        let mut sample_rate = format.Format.nSamplesPerSec;
        let mut gain = 0.5;

        let mut paused = false;
        let volume = 5.0 / 75.0;

        let mut sym = Symphonia::new(
            r"D:\OneDrive\Music\Otuka\still save a seat for you - don't worry about me\Otuka - still save a seat for you.flac",
        )
        .unwrap();

        let s = sym.sample_rate();
        if s != sample_rate {
            println!("Updating sample rate");
            sample_rate = s;

            //Set the new sample rate.
            audio.Stop().unwrap();
            (audio, render, format, event) = create_wasapi(&device, Some(sample_rate));
            //Doesn't need to be set since it's the same device.
            //I just did this to avoid any issues.
            block_align = format.Format.nBlockAlign as u32;
        }

        // let mut pr = PacketRequest {
        //     sym,
        //     buffer: None,
        //     pos: 0,
        // };

        let mut samples: VecDeque<f32> = VecDeque::new();

        while let Some(packet) = sym.next_packet() {
            for sample in packet.samples() {
                samples.push_back(*sample);
            }
        }

        // let (send, recv) = std::sync::mpsc::sync_channel(44100 * 1000);

        // std::thread::spawn(move || {
        //     while let Some(packet) = sym.next_packet() {
        //         for sample in packet.samples() {
        //             send.send(*sample).unwrap();
        //         }
        //     }
        // });

        let now = Instant::now();

        loop {
            // if now.elapsed() >= Duration::from_secs(5) {
            //     return;
            // }

            //Sample-rate probably changed if this fails.
            let padding = audio.GetCurrentPadding().unwrap();
            let buffer_size = audio.GetBufferSize().unwrap();
            let n_frames = buffer_size - 1 - padding;
            let size = (n_frames * block_align) as usize;
            let b = render.GetBuffer(n_frames).unwrap();
            let output = std::slice::from_raw_parts_mut(b, size);
            let channels = format.Format.nChannels as usize;
            let volume = volume * gain;

            for bytes in output.chunks_mut(std::mem::size_of::<f32>() * channels) {
                // let sample = pr.next_sample();
                let sample = samples.pop_front().unwrap_or_default();
                // let sample = recv.try_recv().unwrap_or_default();
                let sample = (sample * volume).to_le_bytes();
                bytes[0..4].copy_from_slice(&sample);

                if channels > 1 {
                    let sample = samples.pop_front().unwrap_or_default();
                    // let sample = pr.next_sample();
                    // let sample = recv.try_recv().unwrap_or_default();
                    let sample = (sample * volume).to_le_bytes();
                    bytes[4..8].copy_from_slice(&sample);
                }
            }

            render.ReleaseBuffer(n_frames, 0).unwrap();

            WaitForSingleObject(event, u32::MAX);
        }
    }
}
