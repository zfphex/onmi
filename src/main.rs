use onmi::*;

fn main() {
    mini::defer_results!();

    let mut player = Player::new();
    let user_request_play = false;

    player
        .play_song(r"D:\OneDrive\Music\black midi\Cavalcade\08. black midi - Ascending Forth.flac")
        .unwrap();

    player.pause();
    player.play();
    player.seek(Duration::from_secs(200));
    player.stop();
    player
        .play_song(r"D:\OneDrive\Music\kinoue64\日常消滅\01 被害者.flac")
        .unwrap();

    player.set_volume(2);
    println!("Volume: {}", player.volume());
    player.seek_forward();
    println!("{:#?}", player.duration());
    // player.seek_backward();

    //Main loop
    loop {
        if user_request_play {
            // player.play();
        }
        std::thread::park();
    }

    // let mut audio = WasapiOutput::new();
    // //Update the sample output sample rate to match the audio file.
    // audio.update_sample_rate(sym.sample_rate());

    // let volume = (10.0 / 75.0) * 0.5; //Volume / Volume Reduction * Gain
    // let channels = audio.channels() as usize;

    // let f = |buffer: &mut [u8]| {
    //     for bytes in buffer.chunks_mut(std::mem::size_of::<f32>() * channels) {
    //         //Only allow for stereo outputs.
    //         for c in 0..channels.max(2) {
    //             let sample = sym.next_sample();
    //             let value = (sample * volume).to_le_bytes();
    //             bytes[c * 4..c * 4 + 4].copy_from_slice(&value);
    //         }
    //     }
    // };

    // audio.run(f);
}
