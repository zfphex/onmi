#![allow(unused)]
use onmi::*;
use std::time::Duration;

fn main() {
    let outputs = OutputDevices::new();

    let output = outputs
        .find("OUT 1-2 (3- BEHRINGER UMC 404HD 192k)")
        .unwrap();
    let player = Player::new(output);

    let song1 = r"D:\OneDrive\Music\black midi\Cavalcade\08. black midi - Ascending Forth.flac";
    let song2 = r"D:\OneDrive\Music\kinoue64\日常消滅\01 被害者.flac";
    let song3 = r"D:\OneDrive\Music\BADBADNOTGOOD\Talk Memory\8. Talk Meaning.flac";
    player.play_song(song1, None, true).unwrap();

    // player.play();
    // player.stop();
    dbg!(metadata(song1, false));

    player.play_song(song3, None, true).unwrap();
    player.seek_to(std::time::Duration::from_secs(80));

    player.set_volume(2);

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
