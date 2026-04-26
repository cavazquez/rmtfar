// SPDX-License-Identifier: GPL-3.0

//! Basic audio utilities for the RMTFAR Mumble plugin.

/// Scale every sample by `volume` (0.0 = silence, 1.0 = unchanged).
pub fn apply_volume(samples: &mut [f32], volume: f32) {
    for s in samples.iter_mut() {
        *s *= volume;
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
