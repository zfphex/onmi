```rs
let output = Output::new();
//Fill the audio buffer with zeroes.
//Loops forever.
output.run(|buffer: &mut [u8]| {
    for bytes in buffer.chunks_mut(4 * channels) {
        bytes.fill(0)
    }
});
```
