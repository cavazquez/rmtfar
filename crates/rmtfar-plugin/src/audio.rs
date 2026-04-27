// SPDX-License-Identifier: GPL-3.0

//! Basic audio utilities for the RMTFAR Mumble plugin.

/// Scale every sample by `volume` (0.0 = silence, 1.0 = unchanged).
pub fn apply_volume(samples: &mut [f32], volume: f32) {
    for s in samples.iter_mut() {
        *s *= volume;
    }
}

/// Apply ear preference to interleaved audio.
/// `stereo_mode`: 0=both, 1=left, 2=right.
pub fn apply_stereo_mode(samples: &mut [f32], channel_count: usize, stereo_mode: u8) {
    if channel_count < 2 || stereo_mode == 0 || stereo_mode > 2 {
        return;
    }
    for frame in samples.chunks_mut(channel_count) {
        if frame.len() < 2 {
            continue;
        }
        match stereo_mode {
            1 => {
                frame[1] = 0.0;
            }
            2 => {
                frame[0] = 0.0;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half_volume() {
        let mut buf = vec![1.0f32, -1.0, 0.5];
        apply_volume(&mut buf, 0.5);
        assert!((buf[0] - 0.5).abs() < 1e-6);
        assert!((buf[1] + 0.5).abs() < 1e-6);
    }

    #[test]
    fn zero_volume_silences() {
        let mut buf = vec![1.0f32, -0.5];
        apply_volume(&mut buf, 0.0);
        for s in &buf {
            assert!(s.abs() < f32::EPSILON);
        }
    }
}
