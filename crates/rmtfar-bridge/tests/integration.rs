// SPDX-License-Identifier: GPL-3.0

//! Integration tests for rmtfar-bridge.
//!
//! Spawns the bridge binary, sends a `PlayerState` via UDP, and verifies the
//! `RadioStateMessage` that the bridge broadcasts to the plugin port.
//!
//! Ports are allocated dynamically (binding to :0) to avoid conflicts when
//! tests run in parallel or alongside a real bridge.

use rmtfar_protocol::{PlayerState, RadioConfig, RadioStateMessage, PROTOCOL_VERSION};
use std::{
    net::UdpSocket,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Bind to :0, get the OS-assigned port, then drop immediately so the bridge
/// can bind to it.  Used only for the bridge's *receive* port where we cannot
/// keep the socket alive (the bridge itself needs to bind it).
fn free_port() -> u16 {
    UdpSocket::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

/// Bind a UDP socket to :0 and return it together with the assigned port.
/// Keeps the socket alive so the port is reserved — no TOCTOU race.
/// Use this for the *plugin* socket that the test itself owns.
fn bound_socket() -> (UdpSocket, u16) {
    let sock = UdpSocket::bind("127.0.0.1:0").unwrap();
    let port = sock.local_addr().unwrap().port();
    (sock, port)
}

struct BridgeProcess(Child);

impl Drop for BridgeProcess {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Spawn the bridge binary pre-built by Cargo with configurable ports.
fn spawn_bridge(recv_port: u16, send_port: u16, local_id: Option<&str>) -> BridgeProcess {
    let bin = env!("CARGO_BIN_EXE_rmtfar-bridge");
    let mut cmd = Command::new(bin);
    cmd.arg("--recv-port")
        .arg(recv_port.to_string())
        .arg("--send-port")
        .arg(send_port.to_string())
        .arg("--log-level")
        .arg("error") // quiet in tests
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    if let Some(id) = local_id {
        cmd.arg("--local-id").arg(id);
    }

    BridgeProcess(cmd.spawn().expect("failed to spawn rmtfar-bridge"))
}

fn make_state(id: &str, pos: [f32; 3], ptt_sr: bool) -> PlayerState {
    PlayerState {
        v: PROTOCOL_VERSION,
        msg_type: "player_state".into(),
        player_id: id.into(),
        server_id: "test-server:2302".into(),
        tick: 42,
        pos,
        dir: 90.0,
        alive: true,
        conscious: true,
        vehicle: String::new(),
        ptt_local: false,
        ptt_radio_sr: ptt_sr,
        ptt_radio_lr: false,
        radio_sr: Some(RadioConfig {
            freq: "43.0".into(),
            channel: 1,
            volume: 1.0,
            enabled: true,
            range_m: None,
            stereo: 0,
            code: String::new(),
        }),
        radio_lr: None,
        radio_los_quality: 1.0,
        intercom_enabled: true,
        intercom_channel: 1,
        intercom_vehicle_id: String::new(),
    }
}

/// Send `state` as JSON to `addr` via UDP.
fn send_state(sock: &UdpSocket, state: &PlayerState, addr: &str) {
    let json = serde_json::to_vec(state).unwrap();
    sock.send_to(&json, addr).unwrap();
}

/// Keep sending `state` until a valid `RadioStateMessage` arrives on `plugin_sock`,
/// or `timeout` elapses.  Returns the first valid message received.
fn poll_for_response(
    plugin_sock: &UdpSocket,
    sender_sock: &UdpSocket,
    state: &PlayerState,
    bridge_addr: &str,
    timeout: Duration,
) -> RadioStateMessage {
    let deadline = Instant::now() + timeout;
    let mut buf = vec![0u8; 65535];

    loop {
        send_state(sender_sock, state, bridge_addr);

        // Give the bridge a moment to process and reply.
        plugin_sock
            .set_read_timeout(Some(Duration::from_millis(100)))
            .unwrap();

        if let Ok((len, _)) = plugin_sock.recv_from(&mut buf) {
            if let Ok(msg) = serde_json::from_slice::<RadioStateMessage>(&buf[..len]) {
                return msg;
            }
        }

        assert!(
            Instant::now() < deadline,
            "timed out waiting for RadioStateMessage from bridge"
        );
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// A single player sends SR radio state; the bridge must echo a
/// `RadioStateMessage` with one player summary reflecting that state.
#[test]
fn bridge_echoes_radio_state_message() {
    let bridge_recv = free_port();
    let (plugin_sock, plugin_port) = bound_socket();
    let sender_sock = UdpSocket::bind("127.0.0.1:0").unwrap();

    let _bridge = spawn_bridge(bridge_recv, plugin_port, Some("p1"));

    let state = make_state("p1", [0.0, 0.0, 0.0], true);
    let bridge_addr = format!("127.0.0.1:{bridge_recv}");

    let msg = poll_for_response(
        &plugin_sock,
        &sender_sock,
        &state,
        &bridge_addr,
        Duration::from_secs(5),
    );

    assert_eq!(msg.local_player, "p1");
    assert_eq!(msg.players.len(), 1);

    let player = &msg.players[0];
    assert_eq!(player.player_id, "p1");
    assert!(player.transmitting_radio, "SR PTT should be active");
    assert_eq!(player.radio_type, "sr");
    assert_eq!(player.radio_freq, "43.0");
    assert_eq!(player.radio_channel, 1);
}

/// Two players: one transmitting, one listening.
/// Both must appear in the `RadioStateMessage`.
#[test]
fn bridge_tracks_multiple_players() {
    let bridge_recv = free_port();
    let (plugin_sock, plugin_port) = bound_socket();
    let sender_sock = UdpSocket::bind("127.0.0.1:0").unwrap();

    let _bridge = spawn_bridge(bridge_recv, plugin_port, Some("p1"));

    let tx = make_state("p1", [0.0, 0.0, 0.0], true);
    let rx = make_state("p2", [100.0, 0.0, 0.0], false);
    let bridge_addr = format!("127.0.0.1:{bridge_recv}");

    // Keep sending both states until we receive a message containing p2.
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut buf = vec![0u8; 65535];
    plugin_sock
        .set_read_timeout(Some(Duration::from_millis(100)))
        .unwrap();

    let msg = loop {
        send_state(&sender_sock, &tx, &bridge_addr);
        send_state(&sender_sock, &rx, &bridge_addr);

        if let Ok((len, _)) = plugin_sock.recv_from(&mut buf) {
            if let Ok(m) = serde_json::from_slice::<RadioStateMessage>(&buf[..len]) {
                if m.players.iter().any(|p| p.player_id == "p2") {
                    break m;
                }
            }
        }

        assert!(
            Instant::now() < deadline,
            "timed out waiting for p2 in RadioStateMessage"
        );
    };

    assert!(
        msg.players.len() >= 2,
        "expected >=2 players, got {}",
        msg.players.len()
    );

    let p2 = msg.players.iter().find(|p| p.player_id == "p2").unwrap();
    assert!(!p2.transmitting_radio, "p2 is not pressing PTT");
    assert_eq!(p2.tuned_sr_freq, "43.0");
}

/// Player sends a dead state; summary must reflect alive=false.
#[test]
fn bridge_reflects_dead_player() {
    let bridge_recv = free_port();
    let (plugin_sock, plugin_port) = bound_socket();
    let sender_sock = UdpSocket::bind("127.0.0.1:0").unwrap();

    let _bridge = spawn_bridge(bridge_recv, plugin_port, Some("p1"));

    let mut state = make_state("p1", [0.0, 0.0, 0.0], true);
    state.alive = false;
    state.ptt_radio_sr = true; // PTT pressed but player is dead
    let bridge_addr = format!("127.0.0.1:{bridge_recv}");

    let msg = poll_for_response(
        &plugin_sock,
        &sender_sock,
        &state,
        &bridge_addr,
        Duration::from_secs(5),
    );

    let player = msg.players.iter().find(|p| p.player_id == "p1").unwrap();
    assert!(!player.alive);
    assert!(!player.transmitting_radio, "dead player must not transmit");
}

/// Player inside a vehicle pressing local PTT: summary must block local PTT.
#[test]
fn bridge_vehicle_blocks_local_ptt() {
    let bridge_recv = free_port();
    let (plugin_sock, plugin_port) = bound_socket();
    let sender_sock = UdpSocket::bind("127.0.0.1:0").unwrap();

    let _bridge = spawn_bridge(bridge_recv, plugin_port, Some("p1"));

    let mut state = make_state("p1", [0.0, 0.0, 0.0], false);
    state.ptt_local = true;
    state.vehicle = "B_MRAP_01_F".into();
    let bridge_addr = format!("127.0.0.1:{bridge_recv}");

    let msg = poll_for_response(
        &plugin_sock,
        &sender_sock,
        &state,
        &bridge_addr,
        Duration::from_secs(5),
    );

    let player = msg.players.iter().find(|p| p.player_id == "p1").unwrap();
    assert!(player.in_vehicle);
    assert!(
        !player.transmitting_local,
        "local PTT must be blocked by vehicle"
    );
}

/// Player uses LR radio; summary must reflect LR type and 20km default range.
#[test]
fn bridge_lr_radio_state() {
    let bridge_recv = free_port();
    let (plugin_sock, plugin_port) = bound_socket();
    let sender_sock = UdpSocket::bind("127.0.0.1:0").unwrap();

    let _bridge = spawn_bridge(bridge_recv, plugin_port, Some("p1"));

    let mut state = make_state("p1", [0.0, 0.0, 0.0], false);
    state.ptt_radio_lr = true;
    state.radio_lr = Some(RadioConfig {
        freq: "30.0".into(),
        channel: 2,
        volume: 1.0,
        enabled: true,
        range_m: None,
        stereo: 0,
        code: String::new(),
    });
    let bridge_addr = format!("127.0.0.1:{bridge_recv}");

    let msg = poll_for_response(
        &plugin_sock,
        &sender_sock,
        &state,
        &bridge_addr,
        Duration::from_secs(5),
    );

    let player = msg.players.iter().find(|p| p.player_id == "p1").unwrap();
    assert!(player.transmitting_radio);
    assert_eq!(player.radio_type, "lr");
    assert_eq!(player.radio_freq, "30.0");
    assert_eq!(player.radio_channel, 2);
    assert!(
        player.radio_range_m > 15_000.0,
        "LR default range should be ~20km, got {}",
        player.radio_range_m
    );
}
