use onmi::*;

fn main() {
    mini::defer_results!();

    let mut sym = Symphonia::new(
        r"D:\OneDrive\Music\black midi\Cavalcade\08. black midi - Ascending Forth.flac",
    )
    .unwrap();

    let mut audio = Audio::new();
    //Update the sample output sample rate to match the audio file.
    audio.update_sample_rate(sym.sample_rate());

    let volume = (10.0 / 75.0) * 0.5; //Volume / Volume Reduction * Gain
    let channels = audio.channels() as usize;

    let f = |buffer: &mut [u8]| {
        for bytes in buffer.chunks_mut(std::mem::size_of::<f32>() * channels) {
            //Only allow for stereo outputs.
            for c in 0..channels.max(2) {
                let sample = sym.next_sample();
                let value = (sample * volume).to_le_bytes();
                bytes[c * 4..c * 4 + 4].copy_from_slice(&value);
            }
        }
    };

    audio.run(f);
}
