use crate::{PlayerState, State, Symphonia};
use std::sync::atomic::Ordering::Relaxed;

pub fn fill_f32_le(
    state: &PlayerState,
    decoder: &mut Option<Symphonia>,
    buffer: &mut [u8],
    channels: usize,
) {
    buffer.fill(0);

    if state.state.load(Relaxed) != State::Playing as u8
        || state.finished.load(Relaxed)
        || state.decoder_pending.load(Relaxed)
        || decoder.is_none()
    {
        return;
    }

    let volume = f32::from_bits(state.volume.load(Relaxed));
    let gain = f32::from_bits(state.gain.load(Relaxed));
    let frame_bytes = std::mem::size_of::<f32>() * channels;

    'outer: for bytes in buffer.chunks_mut(frame_bytes) {
        if state.finished.load(Relaxed) {
            break;
        }

        for c in 0..channels.min(2) {
            let Some(dec) = decoder.as_mut() else {
                break 'outer;
            };

            match dec.next_sample(state) {
                Some(sample) => {
                    let value = (sample * volume * gain).to_le_bytes();
                    bytes[c * 4..c * 4 + 4].copy_from_slice(&value);
                }
                None => {
                    state.finished.store(true, Relaxed);
                    break 'outer;
                }
            }
        }
    }
}
