#![allow(unused)]
use onmi::*;
use std::time::Duration;

fn main() {
    let outputs = OutputDevices::new();
    let device = outputs.default_device();
    let player = Player::new(device);
    player
        .play_song(
            r"D:\OneDrive\Music\black midi\Cavalcade\08. black midi - Ascending Forth.flac",
            true,
        )
        .unwrap();

    // player.play();
    // player.seek(std::time::Duration::from_secs(20));
    // player.stop();

    // player
    //     .play_song(r"D:\OneDrive\Music\kinoue64\日常消滅\01 被害者.flac")
    //     .unwrap();

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
