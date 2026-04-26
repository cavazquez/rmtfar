//! DSP processing for the radio audio effect.
//!
//! Pipeline (applied in order):
//! 1. Bandpass filter (300–3400 Hz)
//! 2. Soft-clip distortion
//! 3. White noise proportional to distance
//! 4. Output volume reduction

use std::f32::consts::PI;

/// Apply the full radio effect to a buffer of mono or interleaved samples.
///
/// - `samples`:    audio buffer (modified in place)
/// - `sample_rate`: e.g. 48000
/// - `dist_m`:     distance from transmitter to receiver in metres
/// - `range_m`:    maximum range of the radio in metres
pub fn apply_radio_effect(samples: &mut [f32], sample_rate: u32, dist_m: f32, range_m: f32) {
    apply_bandpass(samples, sample_rate, 300.0, 3_400.0);

    for s in samples.iter_mut() {
        *s = soft_clip(*s * 1.15);
    }

    let noise_level = ((dist_m / range_m) * 0.30).clamp(0.0, 0.30);
    if noise_level > 0.001 {
        add_noise(samples, noise_level);
    }

    for s in samples.iter_mut() {
        *s *= 0.85;
    }
}

fn apply_bandpass(samples: &mut [f32], sample_rate: u32, low_hz: f32, high_hz: f32) {
    let sr = sample_rate as f32;

    // First-order high-pass at low_hz
    let hp_rc = 1.0 / (2.0 * PI * low_hz);
    let hp_dt = 1.0 / sr;
    let hp_alpha = hp_rc / (hp_rc + hp_dt);
    let mut prev_in = 0.0f32;
    let mut prev_out = 0.0f32;
    for s in samples.iter_mut() {
        let out = hp_alpha * (prev_out + *s - prev_in);
        prev_in = *s;
        prev_out = out;
        *s = out;
    }

    // First-order low-pass at high_hz
    let lp_rc = 1.0 / (2.0 * PI * high_hz);
    let lp_dt = 1.0 / sr;
    let lp_alpha = lp_dt / (lp_rc + lp_dt);
    let mut acc = 0.0f32;
    for s in samples.iter_mut() {
        acc += lp_alpha * (*s - acc);
        *s = acc;
    }
}

#[inline]
fn soft_clip(x: f32) -> f32 {
    x.tanh()
}

fn add_noise(samples: &mut [f32], level: f32) {
    let mut state: u64 = samples.as_ptr() as u64 ^ 0xDEAD_BEEF_CAFE_BABE;
    for s in samples.iter_mut() {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let noise = ((state >> 33) as f32 / (u32::MAX as f32) * 2.0 - 1.0) * level;
        *s = (*s + noise).clamp(-1.0, 1.0);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn soft_clip_bounded() {
        for v in [-10.0f32, -1.5, -1.0, 0.0, 1.0, 1.5, 10.0] {
            let out = soft_clip(v);
            assert!(out >= -1.0 && out <= 1.0, "soft_clip({v}) = {out}");
        }
        // At large values tanh approaches but never exactly exceeds ±1
        assert!(soft_clip(0.0).abs() < 1e-6);
        assert!(soft_clip(1.0) > 0.0 && soft_clip(1.0) < 1.0);
    }

    #[test]
    fn bandpass_doesnt_blow_up() {
        let mut buf: Vec<f32> = (0..480).map(|i| (i as f32 * 0.01).sin()).collect();
        apply_bandpass(&mut buf, 48_000, 300.0, 3_400.0);
        for s in &buf {
            assert!(s.is_finite(), "NaN/Inf in output");
        }
    }

    #[test]
    fn radio_effect_finite() {
        let mut buf: Vec<f32> = (0..480).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
        apply_radio_effect(&mut buf, 48_000, 1_000.0, 5_000.0);
        for s in &buf {
            assert!(s.is_finite(), "NaN/Inf after radio effect");
        }
    }

    #[test]
    fn radio_effect_at_zero_dist() {
        let original: Vec<f32> = (0..480).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
        let mut buf = original.clone();
        apply_radio_effect(&mut buf, 48_000, 0.0, 5_000.0);
        // At distance 0, noise_level = 0 — output should still differ due to bandpass
        for s in &buf {
            assert!(s.is_finite(), "NaN/Inf");
        }
    }
}
