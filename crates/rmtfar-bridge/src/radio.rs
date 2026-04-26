//! Radio matching logic.

use rmtfar_protocol::{distance, PlayerState, RADIO_LR_RANGE_M, RADIO_SR_RANGE_M};

/// Returns true if `receiver` can hear `transmitter` on radio.
#[allow(dead_code)]
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
#[allow(dead_code)]
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
            player_id: id.to_string(),
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
                range_m: None,
            }),
            radio_lr: None,
            radio_los_quality: 1.0,
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

    // -----------------------------------------------------------------------
    // LR radio
    // -----------------------------------------------------------------------

    fn player_lr(id: &str, pos: [f32; 3], ptt_lr: bool, freq: &str, channel: u8) -> PlayerState {
        PlayerState {
            v: 1,
            msg_type: "player_state".into(),
            player_id: id.to_string(),
            server_id: "srv".into(),
            tick: 0,
            pos,
            dir: 0.0,
            alive: true,
            conscious: true,
            vehicle: String::new(),
            ptt_local: false,
            ptt_radio_sr: false,
            ptt_radio_lr: ptt_lr,
            radio_sr: None,
            radio_lr: Some(RadioConfig {
                freq: freq.into(),
                channel,
                volume: 1.0,
                enabled: true,
                range_m: None,
            }),
            radio_los_quality: 1.0,
        }
    }

    #[test]
    fn lr_same_freq_in_range() {
        let tx = player_lr("A", [0.0; 3], true, "30.0", 1);
        let rx = player_lr("B", [1000.0, 0.0, 0.0], false, "30.0", 1);
        assert!(can_hear_radio(&tx, &rx));
    }

    #[test]
    fn lr_out_of_range() {
        let tx = player_lr("A", [0.0; 3], true, "30.0", 1);
        let rx = player_lr("B", [25000.0, 0.0, 0.0], false, "30.0", 1);
        assert!(!can_hear_radio(&tx, &rx));
    }

    #[test]
    fn lr_freq_mismatch() {
        let tx = player_lr("A", [0.0; 3], true, "30.0", 1);
        let rx = player_lr("B", [1000.0, 0.0, 0.0], false, "40.0", 1);
        assert!(!can_hear_radio(&tx, &rx));
    }

    #[test]
    fn lr_channel_mismatch() {
        let tx = player_lr("A", [0.0; 3], true, "30.0", 1);
        let rx = player_lr("B", [1000.0, 0.0, 0.0], false, "30.0", 2);
        assert!(!can_hear_radio(&tx, &rx));
    }

    // -----------------------------------------------------------------------
    // Vehicle
    // -----------------------------------------------------------------------

    #[test]
    fn vehicle_blocks_local_ptt_in_can_hear() {
        // TX in vehicle pressing local PTT — must NOT be heard
        let mut tx = player("A", [0.0; 3], false);
        tx.ptt_local = true;
        tx.vehicle = "B_MRAP_01_F".into();
        let _rx = player("B", [10.0, 0.0, 0.0], false);
        // can_hear_radio checks radio PTT only, but PlayerState::is_transmitting_local
        // returns false for vehicle — verify that at the protocol level
        assert!(!tx.is_transmitting_local(), "local PTT blocked by vehicle");
    }

    #[test]
    fn vehicle_does_not_block_sr_radio() {
        let mut tx = player("A", [0.0; 3], true); // ptt_radio_sr = true
        tx.vehicle = "B_MRAP_01_F".into();
        let rx = player("B", [100.0, 0.0, 0.0], false);
        assert!(can_hear_radio(&tx, &rx), "radio should work inside vehicle");
    }
}
