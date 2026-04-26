//! RMTFAR Test Client — simulates an Arma 3 player sending state to the bridge.
//!
//! Usage examples:
//!   rmtfar-test-client --steam-id 111 --pos 0,0,10
//!   rmtfar-test-client --steam-id 222 --pos 100,0,10 --freq 152.000 --ptt-radio
//!   rmtfar-test-client --steam-id 333 --pos 50,0,10 --freq 155.000

use anyhow::Result;
use clap::Parser;
use rmtfar_protocol::{PlayerState, RadioConfig, BRIDGE_RECV_PORT, PROTOCOL_VERSION};
use std::net::UdpSocket;
use std::time::{Duration, Instant};
use tracing::info;

#[derive(Parser)]
#[command(name = "rmtfar-test-client", version, about = "Simulates an Arma 3 player for RMTFAR testing")]
struct Cli {
    #[arg(long, default_value = "76561198000000001")]
    steam_id: String,

    #[arg(long, default_value = "test-server:2302")]
    server_id: String,

    /// Position as x,y,z in metres
    #[arg(long, default_value = "0,0,10")]
    pos: String,

    #[arg(long, default_value_t = 0.0)]
    dir: f32,

    #[arg(long, default_value = "152.000")]
    freq: String,

    #[arg(long, default_value_t = 1)]
    channel: u8,

    #[arg(long)]
    ptt_local: bool,

    #[arg(long)]
    ptt_radio: bool,

    #[arg(long)]
    dead: bool,

    /// Simulate circular movement with this radius in metres
    #[arg(long, default_value_t = 0.0)]
    circle_radius: f32,

    #[arg(long, default_value_t = 20.0)]
    hz: f32,

    #[arg(long, default_value = "127.0.0.1")]
    bridge_host: String,

    #[arg(long, default_value_t = BRIDGE_RECV_PORT)]
    bridge_port: u16,
}

fn parse_pos(s: &str) -> Result<[f32; 3]> {
    let parts: Vec<f32> = s
        .split(',')
        .map(|v| v.trim().parse::<f32>())
        .collect::<Result<_, _>>()?;
    anyhow::ensure!(parts.len() == 3, "pos must be x,y,z");
    Ok([parts[0], parts[1], parts[2]])
}

fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let cli = Cli::parse();
    let socket = UdpSocket::bind("127.0.0.1:0")?;
    let target = format!("{}:{}", cli.bridge_host, cli.bridge_port);
    socket.connect(&target)?;
    info!("Sending to bridge at {target}");

    let base_pos = parse_pos(&cli.pos)?;
    let interval = Duration::from_secs_f32(1.0 / cli.hz.max(1.0));
    let start = Instant::now();
    let mut tick: u64 = 0;

    info!(
        steam_id = %cli.steam_id,
        pos = ?base_pos,
        freq = %cli.freq,
        "Starting simulation"
    );

    loop {
        let elapsed = start.elapsed().as_secs_f32();
        let pos = if cli.circle_radius > 0.0 {
            let angle = elapsed * std::f32::consts::TAU / 30.0;
            [
                base_pos[0] + cli.circle_radius * angle.sin(),
                base_pos[1],
                base_pos[2] + cli.circle_radius * angle.cos(),
            ]
        } else {
            base_pos
        };

        let dir = if cli.circle_radius > 0.0 {
            let angle = elapsed * std::f32::consts::TAU / 30.0;
            angle.to_degrees().rem_euclid(360.0)
        } else {
            cli.dir
        };

        let state = PlayerState {
            v: PROTOCOL_VERSION,
            msg_type: "player_state".into(),
            steam_id: cli.steam_id.clone(),
            server_id: cli.server_id.clone(),
            tick,
            pos,
            dir,
            alive: !cli.dead,
            conscious: true,
            vehicle: String::new(),
            ptt_local: cli.ptt_local,
            ptt_radio_sr: cli.ptt_radio,
            ptt_radio_lr: false,
            radio_sr: Some(RadioConfig {
                freq: cli.freq.clone(),
                channel: cli.channel,
                volume: 1.0,
                enabled: true,
            }),
            radio_lr: None,
        };

        match serde_json::to_vec(&state) {
            Ok(json) => {
                if let Err(e) = socket.send(&json) {
                    eprintln!("Send error: {e}");
                }
            }
            Err(e) => eprintln!("Encode error: {e}"),
        }

        if tick % (cli.hz as u64 * 5) == 0 {
            info!(tick, pos = ?pos, dir, "heartbeat");
        }

        tick += 1;
        std::thread::sleep(interval);
    }
}
