use onmi::*;

fn main() {
    mini::defer_results!();

    let mut player = Player::new();
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

    std::thread::park();
}
