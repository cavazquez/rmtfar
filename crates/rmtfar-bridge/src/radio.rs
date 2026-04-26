//! Radio matching logic.

use rmtfar_protocol::{distance, PlayerState, RADIO_LR_RANGE_M, RADIO_SR_RANGE_M};

/// Returns true if `receiver` can hear `transmitter` on radio.
pub fn can_hear_radio(transmitter: &PlayerState, receiver: &PlayerState) -> bool {
    if !transmitter.alive || !receiver.alive {
        return false;
    }
    if !transmitter.conscious || !receiver.conscious {
        return false;
    }

    if transmitter.is_transmitting_sr() {
        if let (Some(tx_cfg), Some(rx_cfg)) = (&transmitter.radio_sr, &receiver.radio_sr) {
            if tx_cfg.freq == rx_cfg.freq && tx_cfg.channel == rx_cfg.channel {
                let dist = distance(&transmitter.pos, &receiver.pos);
                return dist <= RADIO_SR_RANGE_M;
            }
        }
    }

    if transmitter.is_transmitting_lr() {
        if let (Some(tx_cfg), Some(rx_cfg)) = (&transmitter.radio_lr, &receiver.radio_lr) {
            if tx_cfg.freq == rx_cfg.freq && tx_cfg.channel == rx_cfg.channel {
                let dist = distance(&transmitter.pos, &receiver.pos);
                return dist <= RADIO_LR_RANGE_M;
            }
        }
    }

    false
}

/// Signal quality 0.0–1.0 based on distance vs max range.
pub fn signal_quality(dist_m: f32, range_m: f32) -> Option<f32> {
    if dist_m > range_m {
        return None;
    }
    Some(1.0 - (dist_m / range_m).clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmtfar_protocol::{PlayerState, RadioConfig};

    fn player(id: &str, pos: [f32; 3], ptt_sr: bool) -> PlayerState {
        PlayerState {
            v: 1,
            msg_type: "player_state".into(),
            steam_id: id.to_string(),
            server_id: "srv".into(),
            tick: 0,
            pos,
            dir: 0.0,
            alive: true,
            conscious: true,
            vehicle: String::new(),
            ptt_local: false,
            ptt_radio_sr: ptt_sr,
            ptt_radio_lr: false,
            radio_sr: Some(RadioConfig {
                freq: "152.000".into(),
                channel: 1,
                volume: 1.0,
                enabled: true,
            }),
            radio_lr: None,
        }
    }

    #[test]
    fn same_freq_in_range() {
        let tx = player("A", [0.0; 3], true);
        let rx = player("B", [100.0, 0.0, 0.0], false);
        assert!(can_hear_radio(&tx, &rx));
    }

    #[test]
    fn out_of_range() {
        let tx = player("A", [0.0; 3], true);
        let rx = player("B", [6000.0, 0.0, 0.0], false);
        assert!(!can_hear_radio(&tx, &rx));
    }

    #[test]
    fn different_freq() {
        let tx = player("A", [0.0; 3], true);
        let mut rx = player("B", [100.0, 0.0, 0.0], false);
        rx.radio_sr.as_mut().unwrap().freq = "155.000".into();
        assert!(!can_hear_radio(&tx, &rx));
    }

    #[test]
    fn dead_cannot_transmit() {
        let mut tx = player("A", [0.0; 3], true);
        tx.alive = false;
        let rx = player("B", [10.0, 0.0, 0.0], false);
        assert!(!can_hear_radio(&tx, &rx));
    }

    #[test]
    fn signal_quality_full() {
        assert_eq!(signal_quality(0.0, 5000.0), Some(1.0));
    }

    #[test]
    fn signal_quality_beyond() {
        assert_eq!(signal_quality(5001.0, 5000.0), None);
    }
}
