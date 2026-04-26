//! RMTFAR Bridge — receives player state from the Arma 3 extension and
//! forwards it to the Mumble plugin via two channels:
//!
//! 1. **Mumble Link** shared memory → Mumble's built-in positional audio
//! 2. **UDP :9501** → RMTFAR plugin for radio frequency / mute logic

mod mumble_link;
mod radio;
mod state;

use anyhow::Result;
use clap::Parser;
use rmtfar_protocol::{
    PlayerState, PlayerSummary, RadioStateMessage, BRIDGE_RECV_PORT, PLUGIN_RECV_PORT,
    PROTOCOL_VERSION,
};
use state::PlayerStore;
use std::net::{SocketAddr, UdpSocket};
use tracing::{debug, error, info, warn};

#[derive(Parser)]
#[command(name = "rmtfar-bridge", version, about = "RMTFAR local bridge")]
struct Cli {
    /// UDP port to receive Arma state on
    #[arg(long, default_value_t = BRIDGE_RECV_PORT)]
    recv_port: u16,

    /// UDP port to send radio state to plugin
    #[arg(long, default_value_t = PLUGIN_RECV_PORT)]
    send_port: u16,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| cli.log_level.parse().unwrap_or_default()),
        )
        .init();

    info!("RMTFAR Bridge v{} starting", env!("CARGO_PKG_VERSION"));

    let recv_socket = UdpSocket::bind(format!("127.0.0.1:{}", cli.recv_port))?;
    info!("Listening for Arma 3 on 127.0.0.1:{}", cli.recv_port);

    let send_socket = UdpSocket::bind("127.0.0.1:0")?;
    let plugin_addr: SocketAddr = format!("127.0.0.1:{}", cli.send_port).parse()?;
    info!("Sending radio state to plugin at {plugin_addr}");

    let mut store = PlayerStore::new();
    let mut mumble = mumble_link::MumbleLink::new();

    let mut buf = [0u8; 8192];

    loop {
        let (len, src) = match recv_socket.recv_from(&mut buf) {
            Ok(v) => v,
            Err(e) => {
                error!("recv_from error: {e}");
                continue;
            }
        };

        let state: PlayerState = match serde_json::from_slice(&buf[..len]) {
            Ok(s) => s,
            Err(e) => {
                warn!("Bad JSON from {src}: {e}");
                continue;
            }
        };

        if state.v != PROTOCOL_VERSION {
            warn!("Unexpected protocol version {} from {src}", state.v);
            continue;
        }

        debug!(
            uid = %state.steam_id,
            pos = ?state.pos,
            dir = state.dir,
            alive = state.alive,
            "state update"
        );

        // Update Mumble Link positional data for the local player.
        mumble.update(&state);

        let local_id = state.steam_id.clone();
        store.update(state);

        let msg = build_message(&store, &local_id);
        match serde_json::to_vec(&msg) {
            Ok(json) => {
                if let Err(e) = send_socket.send_to(&json, plugin_addr) {
                    debug!("send_to plugin: {e}");
                }
            }
            Err(e) => error!("JSON encode error: {e}"),
        }
    }
}

fn build_message(store: &PlayerStore, local_id: &str) -> RadioStateMessage {
    let (server_id, tick) = store
        .get(local_id)
        .map(|s| (s.server_id.clone(), s.tick))
        .unwrap_or_default();

    let players = store
        .all()
        .map(PlayerSummary::from_state)
        .collect::<Vec<_>>();

    RadioStateMessage::new(server_id, tick, local_id.to_string(), players)
}
