Yeah you can play some music I guess

```rust
  let outputs = OutputDevices::new();
  let player = Player::new(outputs.default_device());
  let song = r"D:\OneDrive\Music\BADBADNOTGOOD\Talk Memory\8. Talk Meaning.flac";
  player.set_volume(2);
  player.play_song(song, None, true).unwrap();
```
