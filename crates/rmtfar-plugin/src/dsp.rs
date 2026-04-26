// SPDX-License-Identifier: GPL-3.0

//! Radio DSP effects.
//!
//! ## Pipeline (in order)
//!
//! 1. **2nd-order high-pass 300 Hz** — cuts rumble / bass
//! 2. **2nd-order low-pass 3 400 Hz** — cuts everything above speech band
//! 3. **AGC compressor** — squashes dynamics like real radio hardware
//! 4. **Bitcrusher → 8 000 Hz** — sample-and-hold downsample, the single most
//!    recognisable "radio" artifact
//! 5. **Soft-clip (tanh)** — adds grit / saturation
//! 6. **Bandpassed static noise** — scales with distance
//! 7. **Gain** — scales with `signal_quality`
//! 8. **Dropout crackle** — random 2 ms chunk silencing when quality < 0.5
//!
//! ## Signal quality model
//!
//! `signal_quality` ∈ [0.0, 1.0]:
//! - `1.0` — transmitter at distance 0 (clean signal)
//! - `0.0` — transmitter at the edge of range (heavy static / dropout)

use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Apply the full radio effect chain to a buffer of PCM samples.
///
/// - `sample_rate`: in Hz (typically 48 000).
/// - `signal_quality`: `1.0` = perfect, `0.0` = edge of range.
pub fn apply_radio_effect(samples: &mut [f32], sample_rate: u32, signal_quality: f32) {
    let q = signal_quality.clamp(0.0, 1.0);

    // 1 & 2 — bandpass (speech band 300–3400 Hz)
    biquad_highpass(samples, sample_rate, 300.0);
    biquad_lowpass(samples, sample_rate, 3_400.0);

    // 3 — AGC: squash dynamics, ratio 4:1 above threshold 0.35
    compress(samples, 0.35, 4.0);

    // 4 — bitcrusher: resample to 8 kHz via sample-and-hold
    bitcrusher(samples, sample_rate, 8_000);

    // 5 — soft-clip with higher pre-gain for saturation / grit
    soft_clip_all(samples, 2.5);

    // 6 — noise: constant low floor + distance-dependent static
    //     at q=1.0 → level≈0.018 (barely audible carrier hiss)
    //     at q=0.0 → level≈0.268 (heavy static)
    let noise_level = 0.018 + (1.0 - q).powi(2) * 0.25;
    add_bandpassed_noise(samples, sample_rate, noise_level);

    // 7 — volume envelope: 0.80 at perfect → 0.30 at edge
    let gain = 0.30 + q * 0.50;
    for s in samples.iter_mut() {
        *s *= gain;
    }

    // 8 — dropout crackle: starts at q=0.5, maxes at q=0.0
    if q < 0.5 {
        let dropout_prob = 1.0 - (q / 0.5);
        apply_dropout(samples, dropout_prob);
    }
}

// ---------------------------------------------------------------------------
// DSP primitives
// ---------------------------------------------------------------------------

/// 2nd-order Butterworth high-pass biquad filter (Q = 0.707).
fn biquad_highpass(samples: &mut [f32], sample_rate: u32, cutoff_hz: f32) {
    #[allow(clippy::cast_precision_loss)]
    let w0 = 2.0 * PI * cutoff_hz / sample_rate as f32;
    let cos_w0 = w0.cos();
    let sin_w0 = w0.sin();
    let alpha = sin_w0 / (2.0 * 0.707_f32); // Q = 0.707 (Butterworth)

    let b0 = f32::midpoint(1.0, cos_w0);
    let b1 = -(1.0 + cos_w0);
    let b2 = f32::midpoint(1.0, cos_w0);
    let a0 = 1.0 + alpha;
    let a1 = -2.0 * cos_w0;
    let a2 = 1.0 - alpha;

    run_biquad(samples, [b0 / a0, b1 / a0, b2 / a0], [a1 / a0, a2 / a0]);
}

/// 2nd-order Butterworth low-pass biquad filter (Q = 0.707).
fn biquad_lowpass(samples: &mut [f32], sample_rate: u32, cutoff_hz: f32) {
    #[allow(clippy::cast_precision_loss)]
    let w0 = 2.0 * PI * cutoff_hz / sample_rate as f32;
    let cos_w0 = w0.cos();
    let sin_w0 = w0.sin();
    let alpha = sin_w0 / (2.0 * 0.707_f32);

    let b0 = (1.0 - cos_w0) / 2.0;
    let b1 = 1.0 - cos_w0;
    let b2 = (1.0 - cos_w0) / 2.0;
    let a0 = 1.0 + alpha;
    let a1 = -2.0 * cos_w0;
    let a2 = 1.0 - alpha;

    run_biquad(samples, [b0 / a0, b1 / a0, b2 / a0], [a1 / a0, a2 / a0]);
}

/// Direct-Form II transposed biquad processing.
/// `b` = [b0,b1,b2], `a` = [a1,a2] (a0 already normalised out).
fn run_biquad(samples: &mut [f32], b: [f32; 3], a: [f32; 2]) {
    let mut s1 = 0.0f32;
    let mut s2 = 0.0f32;
    for x in samples.iter_mut() {
        let y = b[0] * *x + s1;
        s1 = b[1] * *x - a[0] * y + s2;
        s2 = b[2] * *x - a[1] * y;
        *x = y;
    }
}

/// Simple peak-follower compressor.
/// Above `threshold` the gain is reduced by `ratio:1`.
fn compress(samples: &mut [f32], threshold: f32, ratio: f32) {
    let attack = 0.001_f32; // fast attack to catch peaks
    let release = 0.995_f32;
    let mut envelope = 0.0f32;

    for s in samples.iter_mut() {
        let abs = s.abs();
        if abs > envelope {
            envelope = envelope + attack * (abs - envelope);
        } else {
            envelope *= release;
        }
        if envelope > threshold {
            // Gain to apply: compress the excess above threshold
            let gain = (threshold + (envelope - threshold) / ratio) / envelope;
            *s *= gain;
        }
    }
}

/// Bitcrusher: sample-and-hold downsample to `target_rate`.
///
/// This is the single most recognisable "radio / walkie-talkie" artifact.
fn bitcrusher(samples: &mut [f32], sample_rate: u32, target_rate: u32) {
    let ratio = (sample_rate / target_rate).max(1) as usize;
    let mut held = 0.0f32;
    for (i, s) in samples.iter_mut().enumerate() {
        if i % ratio == 0 {
            held = *s;
        }
        *s = held;
    }
}

fn soft_clip_all(samples: &mut [f32], gain: f32) {
    for s in samples.iter_mut() {
        *s = (*s * gain).tanh();
    }
}

/// Add bandpass-filtered white noise (300–3400 Hz) to simulate radio static.
/// Running it through the same bandpass makes it sound like real squelch noise.
fn add_bandpassed_noise(samples: &mut [f32], sample_rate: u32, level: f32) {
    if level < 1e-4 {
        return;
    }

    // Generate noise into a temporary scratch buffer.
    let mut noise: Vec<f32> = {
        let mut rng = samples.as_ptr() as u64;
        rng ^= rng >> 33;
        rng = rng.wrapping_mul(0xff51_afd7_ed55_8ccd);
        rng ^= rng >> 33;
        samples
            .iter()
            .map(|_| {
                rng = rng.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                #[allow(clippy::cast_precision_loss)]
                let n = (rng >> 33) as f32 / u32::MAX as f32 * 2.0 - 1.0;
                n * level
            })
            .collect()
    };

    // Bandpass the noise so it sounds like static, not white hiss.
    biquad_highpass(&mut noise, sample_rate, 300.0);
    biquad_lowpass(&mut noise, sample_rate, 3_400.0);

    for (s, n) in samples.iter_mut().zip(noise.iter()) {
        *s += n;
    }
}

/// Silence random ~2 ms chunks with probability `dropout_prob`.
/// Creates the "breaking up" crackle of a weak signal.
fn apply_dropout(samples: &mut [f32], dropout_prob: f32) {
    const CHUNK: usize = 96; // ~2 ms at 48 kHz

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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radio_effect_finite() {
        let mut buf = vec![0.5f32; 480];
        apply_radio_effect(&mut buf, 48_000, 0.8);
        for s in &buf {
            assert!(s.is_finite(), "sample not finite: {s}");
            assert!(s.abs() < 2.0, "sample out of range: {s}");
        }
    }

    #[test]
    fn silence_stays_near_silent() {
        let mut buf = vec![0.0f32; 480];
        apply_radio_effect(&mut buf, 48_000, 0.5);
        for s in &buf {
            assert!(s.abs() < 0.5, "silence became loud: {s}");
        }
    }

    #[test]
    fn good_signal_louder_than_weak() {
        let signal = vec![0.5f32; 480];

        let mut good = signal.clone();
        apply_radio_effect(&mut good, 48_000, 1.0);

        let mut weak = signal.clone();
        apply_radio_effect(&mut weak, 48_000, 0.1);

        #[allow(clippy::cast_precision_loss)]
        let rms = |buf: &[f32]| -> f32 {
            (buf.iter().map(|s| s * s).sum::<f32>() / buf.len() as f32).sqrt()
        };

        assert!(
            rms(&good) > rms(&weak),
            "good ({:.3}) should be louder than weak ({:.3})",
            rms(&good),
            rms(&weak)
        );
    }

    #[test]
    fn weak_signal_has_more_dropouts() {
        let signal = vec![0.5f32; 960];

        let mut strong = signal.clone();
        apply_radio_effect(&mut strong, 48_000, 1.0);

        let mut fragile = signal.clone();
        apply_radio_effect(&mut fragile, 48_000, 0.05);

        let zero_chunks = |buf: &[f32]| -> usize {
            buf.chunks(96).filter(|c| c.iter().all(|&s| s == 0.0)).count()
        };

        assert!(
            zero_chunks(&fragile) > zero_chunks(&strong),
            "fragile signal should have more silent chunks"
        );
    }

    #[test]
    fn noise_increases_with_distance() {
        // Both qualities above the dropout threshold (> 0.5) so only noise
        // differs. quality=0.9 → noise≈0.019, quality=0.6 → noise≈0.034.
        let base = vec![0.0f32; 960];

        let energy = |buf: &[f32]| -> f32 { buf.iter().map(|s| s * s).sum::<f32>() };

        let mut near = base.clone();
        apply_radio_effect(&mut near, 48_000, 0.9);

        let mut far = base.clone();
        apply_radio_effect(&mut far, 48_000, 0.6);

        assert!(
            energy(&far) > energy(&near),
            "far (quality=0.6) should have more noise energy than near (quality=0.9)"
        );
    }

    #[test]
    fn bitcrusher_produces_steps() {
        // After a bitcrusher with ratio=6, consecutive samples within a chunk
        // must be identical.
        #[allow(clippy::cast_precision_loss)]
        let mut buf: Vec<f32> = (0..48).map(|i| i as f32 / 48.0).collect();
        bitcrusher(&mut buf, 48_000, 8_000); // ratio = 6
        for chunk in buf.chunks(6) {
            let first = chunk[0];
            for &s in chunk.iter().skip(1) {
                #[allow(clippy::float_cmp)]
                {
                    assert_eq!(s, first, "bitcrusher: samples within chunk differ");
                }
            }
        }
    }
}
