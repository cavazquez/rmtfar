//! Shared protocol types for RMTFAR.
//!
//! Arma 3 → Bridge: [`PlayerState`] over UDP :9500
//! Bridge → Plugin:  [`RadioStateMessage`] over UDP :9501

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const BRIDGE_RECV_PORT: u16 = 9500;
pub const PLUGIN_RECV_PORT: u16 = 9501;

pub const PROTOCOL_VERSION: u8 = 1;

/// Maximum distance in meters for direct (non-radio) voice.
pub const LOCAL_VOICE_RANGE_M: f32 = 50.0;

/// Short-range radio reach in meters (5 km).
pub const RADIO_SR_RANGE_M: f32 = 5_000.0;

/// Long-range radio reach in meters (20 km).
pub const RADIO_LR_RANGE_M: f32 = 20_000.0;

// ---------------------------------------------------------------------------
// Arma 3 → Bridge message
// ---------------------------------------------------------------------------

/// State of one player's radio (short-range or long-range).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RadioConfig {
    /// Frequency string, e.g. "152.000"
    pub freq: String,
    /// Channel index within the frequency (1-indexed)
    pub channel: u8,
    /// Volume scalar 0.0–1.0
    pub volume: f32,
    pub enabled: bool,
}

impl Default for RadioConfig {
    fn default() -> Self {
        Self {
            freq: "152.000".into(),
            channel: 1,
            volume: 1.0,
            enabled: true,
        }
    }
}

/// Full state packet sent from Arma 3 (via extension) to the bridge.
///
/// Sent at ~20 Hz over UDP on loopback port 9500.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)] // Protocol fields, not flags
pub struct PlayerState {
    /// Protocol version — must be [`PROTOCOL_VERSION`].
    pub v: u8,
    #[serde(rename = "type")]
    pub msg_type: String,
    /// `SteamID64` of the local player.
    pub steam_id: String,
    /// `serverName` value from SQF (unique per session).
    pub server_id: String,
    /// Arma 3 game tick.
    pub tick: u64,
    /// Position [x, y, z] in metres (ASL).
    pub pos: [f32; 3],
    /// Heading in degrees 0–360 (0 = north).
    pub dir: f32,
    pub alive: bool,
    /// Player is conscious (false when ACE unconscious).
    pub conscious: bool,
    /// Class name of the current vehicle, empty string if on foot.
    #[serde(default)]
    pub vehicle: String,
    /// PTT for direct (local proximity) voice.
    #[serde(default)]
    pub ptt_local: bool,
    /// PTT for short-range radio.
    #[serde(default)]
    pub ptt_radio_sr: bool,
    /// PTT for long-range radio.
    #[serde(default)]
    pub ptt_radio_lr: bool,
    /// Short-range radio config (None = no radio equipped).
    pub radio_sr: Option<RadioConfig>,
    /// Long-range radio config (None = no radio equipped).
    pub radio_lr: Option<RadioConfig>,
}

impl PlayerState {
    /// Direction vector (unit) derived from heading.
    /// Mumble coordinate system: X right, Y up, Z forward.
    pub fn front_vector(&self) -> [f32; 3] {
        let rad = self.dir.to_radians();
        [rad.sin(), 0.0, rad.cos()]
    }

    pub fn is_transmitting_local(&self) -> bool {
        self.ptt_local && self.alive && self.conscious
    }

    pub fn is_transmitting_sr(&self) -> bool {
        self.ptt_radio_sr
            && self.alive
            && self.conscious
            && self.radio_sr.as_ref().is_some_and(|r| r.enabled)
    }

    pub fn is_transmitting_lr(&self) -> bool {
        self.ptt_radio_lr
            && self.alive
            && self.conscious
            && self.radio_lr.as_ref().is_some_and(|r| r.enabled)
    }
}

// ---------------------------------------------------------------------------
// Bridge → Plugin message
// ---------------------------------------------------------------------------

/// Per-player summary sent from the bridge to the Mumble plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)] // Protocol fields, not flags
pub struct PlayerSummary {
    pub steam_id: String,
    /// Position [x, y, z] in metres (ASL).
    pub pos: [f32; 3],
    pub dir: f32,
    pub alive: bool,
    pub conscious: bool,
    pub transmitting_local: bool,
    pub transmitting_radio: bool,
    /// Which radio type is transmitting: "sr", "lr", or ""
    #[serde(default)]
    pub radio_type: String,
    /// Frequency of the active transmission, e.g. "152.000"
    #[serde(default)]
    pub radio_freq: String,
    /// Channel of the active transmission
    #[serde(default)]
    pub radio_channel: u8,
    /// Reach in metres for the current radio
    #[serde(default)]
    pub radio_range_m: f32,
}

impl PlayerSummary {
    pub fn from_state(state: &PlayerState) -> Self {
        let (transmitting_radio, radio_type, radio_freq, radio_channel, radio_range_m) =
            if state.is_transmitting_sr() {
                let cfg = state.radio_sr.as_ref().unwrap();
                (
                    true,
                    "sr".into(),
                    cfg.freq.clone(),
                    cfg.channel,
                    RADIO_SR_RANGE_M,
                )
            } else if state.is_transmitting_lr() {
                let cfg = state.radio_lr.as_ref().unwrap();
                (
                    true,
                    "lr".into(),
                    cfg.freq.clone(),
                    cfg.channel,
                    RADIO_LR_RANGE_M,
                )
            } else {
                (false, String::new(), String::new(), 0, 0.0)
            };

        Self {
            steam_id: state.steam_id.clone(),
            pos: state.pos,
            dir: state.dir,
            alive: state.alive,
            conscious: state.conscious,
            transmitting_local: state.is_transmitting_local(),
            transmitting_radio,
            radio_type,
            radio_freq,
            radio_channel,
            radio_range_m,
        }
    }
}

/// Full state broadcast from bridge to Mumble plugin.
///
/// Sent after every [`PlayerState`] update on UDP port 9501.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadioStateMessage {
    pub v: u8,
    #[serde(rename = "type")]
    pub msg_type: String,
    pub server_id: String,
    pub tick: u64,
    /// `SteamID64` of the local player (the one running the bridge).
    pub local_player: String,
    pub players: Vec<PlayerSummary>,
}

impl RadioStateMessage {
    pub fn new(
        server_id: String,
        tick: u64,
        local_player: String,
        players: Vec<PlayerSummary>,
    ) -> Self {
        Self {
            v: PROTOCOL_VERSION,
            msg_type: "radio_state".into(),
            server_id,
            tick,
            local_player,
            players,
        }
    }

    pub fn find_player(&self, steam_id: &str) -> Option<&PlayerSummary> {
        self.players.iter().find(|p| p.steam_id == steam_id)
    }

    pub fn local(&self) -> Option<&PlayerSummary> {
        self.find_player(&self.local_player)
    }
}

// ---------------------------------------------------------------------------
// Geometry helpers
// ---------------------------------------------------------------------------

/// Euclidean distance between two 3D positions.
pub fn distance(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Convert Arma heading (degrees, 0=north, clockwise) to Mumble front vector.
/// Mumble: X right, Y up, Z forward.
pub fn heading_to_front(dir_deg: f32) -> [f32; 3] {
    let rad = dir_deg.to_radians();
    [rad.sin(), 0.0, rad.cos()]
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Unexpected protocol version: {0}")]
    WrongVersion(u8),
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_state() -> PlayerState {
        PlayerState {
            v: 1,
            msg_type: "player_state".into(),
            steam_id: "76561198000000000".into(),
            server_id: "192.168.1.100:2302".into(),
            tick: 1000,
            pos: [100.0, 50.0, 10.0],
            dir: 90.0,
            alive: true,
            conscious: true,
            vehicle: String::new(),
            ptt_local: false,
            ptt_radio_sr: false,
            ptt_radio_lr: false,
            radio_sr: Some(RadioConfig::default()),
            radio_lr: None,
        }
    }

    #[test]
    fn round_trip_player_state() {
        let state = sample_state();
        let json = serde_json::to_string(&state).unwrap();
        let back: PlayerState = serde_json::from_str(&json).unwrap();
        assert_eq!(back.steam_id, state.steam_id);
        // Float fields: compare element-wise with tolerance
        for (a, b) in back.pos.iter().zip(state.pos.iter()) {
            assert!((a - b).abs() < f32::EPSILON, "pos mismatch");
        }
        assert!((back.dir - state.dir).abs() < f32::EPSILON);
    }

    #[test]
    fn front_vector_east() {
        // 90 degrees = east → X positive, Z~0
        let [x, y, z] = heading_to_front(90.0);
        assert!((x - 1.0).abs() < 1e-5, "x={x}");
        assert!(y.abs() < 1e-5, "y={y}");
        assert!(z.abs() < 1e-5, "z={z}");
    }

    #[test]
    fn front_vector_north() {
        // 0 degrees = north → Z positive
        let [x, _y, z] = heading_to_front(0.0);
        assert!(x.abs() < 1e-5, "x={x}");
        assert!((z - 1.0).abs() < 1e-5, "z={z}");
    }

    #[test]
    fn distance_zero() {
        let p = [1.0f32, 2.0, 3.0];
        assert!(distance(&p, &p) < f32::EPSILON);
    }

    #[test]
    fn distance_known() {
        let a = [0.0f32, 0.0, 0.0];
        let b = [3.0, 4.0, 0.0];
        assert!((distance(&a, &b) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn player_summary_sr_transmit() {
        let mut state = sample_state();
        state.ptt_radio_sr = true;
        let summary = PlayerSummary::from_state(&state);
        assert!(summary.transmitting_radio);
        assert_eq!(summary.radio_type, "sr");
        assert_eq!(summary.radio_freq, "152.000");
        assert!((summary.radio_range_m - RADIO_SR_RANGE_M).abs() < f32::EPSILON);
    }

    #[test]
    fn radio_state_message_find_local() {
        let state = sample_state();
        let summary = PlayerSummary::from_state(&state);
        let msg = RadioStateMessage::new(
            state.server_id.clone(),
            state.tick,
            state.steam_id.clone(),
            vec![summary],
        );
        assert!(msg.local().is_some());
        assert_eq!(msg.local().unwrap().steam_id, state.steam_id);
    }

    #[test]
    fn deserialize_minimal_json() {
        // Simulate minimal JSON from SQF (no optional fields)
        let json = r#"{
            "v": 1,
            "type": "player_state",
            "steam_id": "123",
            "server_id": "srv",
            "tick": 42,
            "pos": [0.0, 0.0, 0.0],
            "dir": 0.0,
            "alive": true,
            "conscious": true,
            "radio_sr": null,
            "radio_lr": null
        }"#;
        let state: PlayerState = serde_json::from_str(json).unwrap();
        assert_eq!(state.steam_id, "123");
        assert!(!state.ptt_local);
    }
}
