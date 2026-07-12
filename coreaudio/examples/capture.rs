use coreaudio::*;
use std::ffi::c_void;

extern "C" fn audio_input_callback(
    _context: *mut c_void,
    buffer_ptr: *mut f32,
    total_samples: usize,
) {
    unsafe {
        let buffer = std::slice::from_raw_parts(buffer_ptr, total_samples);

        let mut rms = 0.0;
        for &sample in buffer {
            rms += sample * sample;
        }
        rms = (rms / total_samples as f32).sqrt();

        // Print a simple level meter
        let level = (rms * 50.0) as usize;
        let meter = "*".repeat(level.min(50));
        println!("|{:<50}| {:.3}", meter, rms);
    }
}

fn main() {
    let input = AudioDevice::default_input().expect("No input device");
    let name = input.name().unwrap_or_else(|_| "Unknown".to_string());
    let sample_rate = input.sample_rate().unwrap_or(44100.0);
    let channels = input.input_channel_count().unwrap_or(1);
    
    let channels = if channels == 0 { 1 } else { channels };
    
    println!("Capturing from: {} ({}Hz, {} channels)", name, sample_rate, channels);

    let _stream = unsafe {
        AudioStream::start_input(input, sample_rate, channels, audio_input_callback, std::ptr::null_mut())
            .expect("Failed to start stream")
    };

    println!("Capturing audio for 5 seconds... Speak now!");
    std::thread::sleep(std::time::Duration::from_secs(5));
}
