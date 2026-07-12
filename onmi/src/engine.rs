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
    let scale = volume * gain;
    let frame_bytes = size_of::<f32>() * channels;
    let mut src_frame = [0f32; 16];

    'outer: for bytes in buffer.chunks_mut(frame_bytes) {
        if state.finished.load(Relaxed) {
            break;
        }

        let Some(dec) = decoder.as_mut() else {
            break;
        };

        let src_ch = (dec.channels as usize).clamp(1, 16);
        for i in 0..src_ch {
            match dec.next_sample(state) {
                Some(sample) => src_frame[i] = sample,
                None => {
                    state.mark_finished();
                    break 'outer;
                }
            }
        }

        if channels == 1 {
            let sample = if src_ch >= 2 {
                (src_frame[0] + src_frame[1]) * 0.5
            } else {
                src_frame[0]
            };
            bytes[0..4].copy_from_slice(&(sample * scale).to_le_bytes());
        } else {
            let left = src_frame[0] * scale;
            let right = if src_ch >= 2 {
                src_frame[1] * scale
            } else {
                left
            };
            bytes[0..4].copy_from_slice(&left.to_le_bytes());
            if channels >= 2 {
                bytes[4..8].copy_from_slice(&right.to_le_bytes());
            }
            for c in 2..channels {
                let sample = if c % 2 == 0 { left } else { right };
                let off = c * 4;
                bytes[off..off + 4].copy_from_slice(&sample.to_le_bytes());
            }
        }
    }
}
