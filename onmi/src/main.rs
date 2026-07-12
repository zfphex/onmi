use onmi::*;

fn main() {
    let devices = OutputDevices::new();
    let list = devices.devices();
    println!("Output devices:");
    for d in &list {
        println!("  - {}", d.name);
    }

    let device = devices.default_device();
    println!("Using default: {}", device.name);

    let mut player = Player::new(device);
    player.set_volume(10);

    let path = std::env::args().nth(1);
    let Some(path) = path else {
        println!("Usage: onmi <path-to-audio-file>");
        println!("No file given; shutting down.");
        return;
    };

    println!("Playing: {path}");
    player.play_song(&path, None, true).unwrap();
    player.set_volume(50);

    std::thread::park();
}
