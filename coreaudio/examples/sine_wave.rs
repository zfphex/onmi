use coreaudio::*;
use std::ffi::c_void;

struct Synth {
    phase: f32,
    frequency: f32,
    sample_rate: f32,
    channels: u32,
}

extern "C" fn audio_callback(context: *mut c_void, buffer_ptr: *mut f32, total_samples: usize) {
    unsafe {
        let synth = &mut *(context as *mut Synth);
        let buffer = std::slice::from_raw_parts_mut(buffer_ptr, total_samples);

        let phase_increment = synth.frequency / synth.sample_rate;
        let channels = synth.channels as usize;

        for chunk in buffer.chunks_exact_mut(channels) {
            let sample = (synth.phase * 2.0 * std::f32::consts::PI).sin() * 0.1; // 10% volume
            synth.phase += phase_increment;
            if synth.phase >= 1.0 {
                synth.phase -= 1.0;
            }

            for sample_out in chunk {
                *sample_out = sample;
            }
        }
    }
}

fn main() {
    let output = AudioDevice::default_output().expect("No output device");
    let name = output.name().unwrap_or_else(|_| "Unknown".to_string());
    let sample_rate = output.sample_rate().unwrap_or(44100.0);
    let channels = output.output_channel_count().unwrap_or(2);
    
    let channels = if channels == 0 { 2 } else { channels };
    
    println!("Playing on: {} ({}Hz, {} channels)", name, sample_rate, channels);

    let mut synth = Box::new(Synth {
        phase: 0.0,
        frequency: 440.0, // A4
        sample_rate: sample_rate as f32,
        channels,
    });

    let context_ptr = synth.as_mut() as *mut Synth as *mut c_void;

    let _stream = unsafe {
        AudioStream::start_output(output, sample_rate, channels, audio_callback, context_ptr)
            .expect("Failed to start stream")
    };

    println!("Playing a 440Hz sine wave for 3 seconds...");
    std::thread::sleep(std::time::Duration::from_secs(3));
}
