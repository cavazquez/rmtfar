// SPDX-License-Identifier: GPL-3.0

//! Radio DSP effects: bandpass filter, soft-clip, and noise injection.
//!
//! Pipeline: high-pass (300 Hz) → low-pass (3400 Hz) → soft-clip → noise

use std::f32::consts::PI;

/// Apply the full radio effect chain to a buffer of PCM samples.
///
/// - `sample_rate`: in Hz (typically 48000).
/// - `dist_m`: distance to transmitter in metres.
/// - `range_m`: maximum radio range in metres (used to scale noise level).
pub fn apply_radio_effect(samples: &mut [f32], sample_rate: u32, dist_m: f32, range_m: f32) {
    apply_highpass(samples, sample_rate, 300.0);
    apply_lowpass(samples, sample_rate, 3400.0);
    soft_clip_all(samples, 1.2);
    let noise_level = (dist_m / range_m).clamp(0.0, 1.0) * 0.25;
    add_noise(samples, noise_level);
    for s in samples.iter_mut() {
        *s *= 0.85;
    }
}

fn apply_highpass(samples: &mut [f32], sample_rate: u32, cutoff_hz: f32) {
    let rc = 1.0 / (2.0 * PI * cutoff_hz);
    #[allow(clippy::cast_precision_loss)]
    let dt = 1.0 / sample_rate as f32;
    let alpha = rc / (rc + dt);
    let mut prev_in = 0.0f32;
    let mut prev_out = 0.0f32;
    for s in samples.iter_mut() {
        let x = *s;
        let y = alpha * (prev_out + x - prev_in);
        prev_in = x;
        prev_out = y;
        *s = y;
    }
}

fn apply_lowpass(samples: &mut [f32], sample_rate: u32, cutoff_hz: f32) {
    let rc = 1.0 / (2.0 * PI * cutoff_hz);
    #[allow(clippy::cast_precision_loss)]
    let dt = 1.0 / sample_rate as f32;
    let alpha = dt / (rc + dt);
    let mut prev = 0.0f32;
    for s in samples.iter_mut() {
        prev += alpha * (*s - prev);
        *s = prev;
    }
}

fn soft_clip_all(samples: &mut [f32], gain: f32) {
    for s in samples.iter_mut() {
        *s = (*s * gain).tanh();
    }
}

fn add_noise(samples: &mut [f32], level: f32) {
    if level < 1e-4 {
        return;
    }
    // Simple LCG seeded from the slice address for per-frame variation.
    let mut rng = samples.as_ptr() as u64;
    rng ^= rng >> 33;
    rng = rng.wrapping_mul(0xff51_afd7_ed55_8ccd);
    rng ^= rng >> 33;
    for s in samples.iter_mut() {
        rng = rng.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        #[allow(clippy::cast_precision_loss)]
        let noise = ((rng >> 33) as f32 / u32::MAX as f32 * 2.0 - 1.0) * level;
        *s += noise;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radio_effect_finite() {
        let mut buf = vec![0.5f32; 480];
        apply_radio_effect(&mut buf, 48000, 1000.0, 5000.0);
        for s in &buf {
            assert!(s.is_finite());
            assert!(s.abs() < 2.0);
        }
    }

    #[test]
    fn silence_stays_near_silent() {
        let mut buf = vec![0.0f32; 480];
        apply_radio_effect(&mut buf, 48000, 100.0, 5000.0);
        for s in &buf {
            assert!(s.abs() < 0.5);
        }
    }
}
