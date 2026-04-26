// SPDX-License-Identifier: GPL-3.0

//! Radio DSP effects: bandpass filter, soft-clip, noise and distance crackle.
//!
//! Pipeline: high-pass (300 Hz) → low-pass (3400 Hz) → soft-clip → noise → dropout
//!
//! ## Signal quality model
//!
//! `signal_quality` is a value in `[0.0, 1.0]`:
//! - `1.0` — transmitter at distance 0 (perfect signal, subtle effect)
//! - `0.0` — transmitter at the edge of range (heavy static, frequent dropouts)
//!
//! The degradation zones are:
//! | Quality | Distance | Effect |
//! |---------|----------|--------|
//! | > 0.4   | < 60% range | Clean radio effect, low noise |
//! | 0.1–0.4 | 60–90% range | Increasing static, starts dropping out |
//! | < 0.1   | > 90% range | Heavy crackle, frequent silence bursts |

use std::f32::consts::PI;

/// Apply the full radio effect chain to a buffer of PCM samples.
///
/// - `sample_rate`: in Hz (typically 48000).
/// - `signal_quality`: `1.0` = perfect (close), `0.0` = edge of range.
pub fn apply_radio_effect(samples: &mut [f32], sample_rate: u32, signal_quality: f32) {
    let q = signal_quality.clamp(0.0, 1.0);

    apply_highpass(samples, sample_rate, 300.0);
    apply_lowpass(samples, sample_rate, 3400.0);
    soft_clip_all(samples, 1.2);

    // Noise: exponential growth as quality drops.
    // 0.02 at full quality → 0.55 at edge of range.
    let noise_level = (1.0 - q).powi(2) * 0.55;
    add_noise(samples, noise_level);

    // Volume: 0.85 at perfect signal → 0.25 near the edge.
    let gain = 0.25 + q * 0.60;
    for s in samples.iter_mut() {
        *s *= gain;
    }

    // Dropout crackle: random per-chunk silencing when quality < 0.5.
    // Probability rises from 0 at quality=0.5 to 1 at quality=0.
    if q < 0.5 {
        let dropout_prob = 1.0 - (q / 0.5);
        apply_dropout(samples, dropout_prob);
    }
}

/// Silence random ~2ms chunks of the buffer with probability `dropout_prob`.
///
/// Creates the characteristic "breaking up" crackle of a weak radio signal.
fn apply_dropout(samples: &mut [f32], dropout_prob: f32) {
    // ~2ms chunks at 48kHz = 96 samples; keeps crackle grain audible.
    const CHUNK: usize = 96;

    // Per-frame LCG seeded from the buffer address for deterministic but
    // varied output across consecutive frames.
    let mut rng = samples.as_ptr() as u64;
    rng ^= rng >> 33;
    rng = rng.wrapping_mul(0xff51_afd7_ed55_8ccd);
    rng ^= rng >> 33;

    for chunk in samples.chunks_mut(CHUNK) {
        rng = rng.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        #[allow(clippy::cast_precision_loss)]
        let r = (rng >> 33) as f32 / u32::MAX as f32;
        if r < dropout_prob {
            chunk.fill(0.0);
        }
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
        apply_radio_effect(&mut buf, 48000, 0.8);
        for s in &buf {
            assert!(s.is_finite());
            assert!(s.abs() < 2.0);
        }
    }

    #[test]
    fn silence_stays_near_silent() {
        let mut buf = vec![0.0f32; 480];
        apply_radio_effect(&mut buf, 48000, 0.5);
        for s in &buf {
            assert!(s.abs() < 0.5);
        }
    }

    #[test]
    fn good_signal_louder_than_weak() {
        let signal = vec![0.5f32; 480];

        let mut good = signal.clone();
        apply_radio_effect(&mut good, 48000, 1.0); // perfect signal

        let mut weak = signal.clone();
        apply_radio_effect(&mut weak, 48000, 0.1); // near edge

        #[allow(clippy::cast_precision_loss)]
        let rms = |buf: &[f32]| -> f32 {
            (buf.iter().map(|s| s * s).sum::<f32>() / buf.len() as f32).sqrt()
        };

        assert!(
            rms(&good) > rms(&weak),
            "good signal ({:.3}) should be louder than weak ({:.3})",
            rms(&good),
            rms(&weak)
        );
    }

    #[test]
    fn weak_signal_has_more_dropouts() {
        // A weak signal buffer will have more zero-chunks than a strong one.
        let signal = vec![0.5f32; 960]; // 20ms at 48kHz

        let mut strong = signal.clone();
        apply_radio_effect(&mut strong, 48000, 1.0);

        let mut fragile = signal.clone();
        apply_radio_effect(&mut fragile, 48000, 0.05);

        let zero_chunks = |buf: &[f32]| -> usize {
            buf.chunks(96)
                .filter(|c| c.iter().all(|&s| s == 0.0))
                .count()
        };

        assert!(
            zero_chunks(&fragile) > zero_chunks(&strong),
            "fragile signal should have more silent chunks"
        );
    }

    #[test]
    fn noise_increases_with_distance() {
        // Compare two qualities both above the dropout threshold (> 0.5),
        // so only noise differences affect energy on a silent input.
        // quality=0.9 → noise_level ≈ 0.006
        // quality=0.6 → noise_level ≈ 0.088
        let base = vec![0.0f32; 960];

        let energy = |buf: &[f32]| -> f32 { buf.iter().map(|s| s * s).sum::<f32>() };

        let mut near = base.clone();
        apply_radio_effect(&mut near, 48000, 0.9);

        let mut far = base.clone();
        apply_radio_effect(&mut far, 48000, 0.6);

        assert!(
            energy(&far) > energy(&near),
            "far signal (quality=0.6) should have more noise energy than near (quality=0.9)"
        );
    }
}
