// SPDX-License-Identifier: GPL-3.0

//! Primitivas de procesamiento de audio para el plugin.

/// Acción que el plugin debe aplicar al audio de un usuario.
#[derive(Debug, Clone, PartialEq)]
pub enum AudioAction {
    /// Dejar el audio sin modificar.
    PassThrough,
    /// Silenciar completamente.
    Mute,
    /// Ajustar volumen (0.0 - 1.0).
    Volume(f32),
    /// Aplicar efecto de radio con volumen y distancia.
    RadioEffect { volume: f32, distance: f32 },
}

/// Multiplica cada muestra por `volume`.
pub fn apply_volume(samples: &mut [f32], volume: f32) {
    for s in samples.iter_mut() {
        *s *= volume;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_volume_halves() {
        let mut buf = vec![1.0f32, -1.0, 0.5];
        apply_volume(&mut buf, 0.5);
        assert!((buf[0] - 0.5).abs() < 1e-6);
        assert!((buf[1] + 0.5).abs() < 1e-6);
        assert!((buf[2] - 0.25).abs() < 1e-6);
    }

    #[test]
    fn apply_volume_zero_silences() {
        let mut buf = vec![1.0f32, -0.5, 0.3];
        apply_volume(&mut buf, 0.0);
        for s in &buf {
            assert_eq!(*s, 0.0);
        }
    }
}
