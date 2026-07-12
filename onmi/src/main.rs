use onmi::*;

fn main() {
    let outputs = OutputDevices::new();

    let output = outputs
        .find("OUT 1-2 (3- BEHRINGER UMC 404HD 192k)")
        .unwrap();
    let other = outputs
        .find("OUT 1-4 (3- BEHRINGER UMC 404HD 192k)")
        .unwrap();
    let mut player = Player::new(output);

    let _song1 = r"D:\OneDrive\Music\black midi\Cavalcade\08. black midi - Ascending Forth.flac";
    let _song2 = r"D:\OneDrive\Music\kinoue64\日常消滅\01 被害者.flac";
    let _song3 = r"D:\OneDrive\Music\BADBADNOTGOOD\Talk Memory\8. Talk Meaning.flac";
    let _song4 = r"D:\OneDrive\Music\Iglooghost\Lei Disk 「Radio•Broadcast」\03 Oblique Cell 『1982 _ RESCANNED•MOD』.flac";
    let _song5 = r"D:\OneDrive\Music\EDEN\DDAARRKK\02 EDEN - Twiin.flac";

    player.set_volume(2);
    player.play_song(_song5, None, true).unwrap();
    std::thread::sleep_ms(1000);
    println!("{:?}", player.current_song_sample_rate);
    println!("swapped device");
    player.set_output_device(other);

    //TODO: Seeking past a certain point should set the player into the finished state.
    // player.seek(Duration::from_secs_f32(900.0));

    // println!("Volume: {}", player.volume());
    // player.seek_forward();
    // println!("{:#?}", player.duration());

    // player.pause();
    // player.stop();
    // player.pause();
    // player.pause();

    // loop {
    //     dbg!(player.state());
    // }

    std::thread::park();
}
