```rs
//Can be spawned on any thread.
let wasapi = Wasapi::new();

//Fill the audio buffer with zeroes.
wasapi.fill(|buffer: &mut [u8]| {
    for bytes in buffer.chunks_mut(4 * channels) {
        bytes.fill(0)
    }
});
```
