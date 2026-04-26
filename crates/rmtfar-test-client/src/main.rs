// SPDX-License-Identifier: GPL-3.0

//! Test client that simulates an Arma 3 player sending state to the bridge.
//!
//! Usage:
//! ```
//! # Static player at [100, 0, 200]
//! rmtfar-test-client --id player1 --pos 100,0,200
//!
//! # Player orbiting the origin with 50 m radius
//! rmtfar-test-client --id player1 --orbit --orbit-radius 50
//!
//! # Player with local PTT active
//! rmtfar-test-client --id player1 --ptt-local
//!
//! # Player transmitting on SR radio at 152.000
//! rmtfar-test-client --id player1 --ptt-radio --freq 152.000
//! ```

use anyhow::{Context, Result};
use rmtfar_protocol::{PlayerState, RadioConfig};
use std::net::UdpSocket;
use std::time::{Duration, Instant};

const DEFAULT_BRIDGE_ADDR: &str = "127.0.0.1:9500";
const SEND_INTERVAL: Duration = Duration::from_millis(50); // 20 Hz

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let config = parse_args(&args)?;

    let bridge_addr = &config.bridge_addr;
    let socket = UdpSocket::bind("0.0.0.0:0").context("bind")?;
    socket.connect(bridge_addr.as_str()).context("connect")?;

    println!("RMTFAR Test Client");
    println!("  Player : {}", config.steam_id);
    println!("  Server : {}", config.server_id);
    println!("  Mode   : {}", config.mode_label());
    println!("  Target : {bridge_addr} @ 20 Hz  (Ctrl-C to stop)\n");

    let start = Instant::now();
    let mut tick: u64 = 0;

    loop {
        let elapsed = start.elapsed().as_secs_f32();
        let pos = config.current_pos(elapsed);
        let dir = config.current_dir(elapsed);

        let msg = build_state(&config, tick, pos, dir);
        let json = serde_json::to_vec(&msg)?;
        socket.send(&json).context("send")?;

        println!(
            "[{:>8.2}s] tick={:<6} pos=[{:>8.1}, {:>8.1}, {:>6.1}] dir={:>5.1}° \
             ptt_local={} ptt_sr={} ptt_lr={} alive={} conscious={} vehicle={:?}",
            elapsed,
            tick,
            pos[0],
            pos[1],
            pos[2],
            dir,
            config.ptt_local,
            config.ptt_radio_sr,
            config.ptt_radio_lr,
            config.alive,
            config.conscious,
            config.vehicle,
        );

        tick += 1;
        std::thread::sleep(SEND_INTERVAL);
    }
}

fn build_state(cfg: &Config, tick: u64, pos: [f32; 3], dir: f32) -> PlayerState {
    PlayerState {
        v: 1,
        msg_type: "player_state".into(),
        steam_id: cfg.steam_id.clone(),
        server_id: cfg.server_id.clone(),
        tick,
        pos,
        dir,
        alive: cfg.alive,
        conscious: cfg.conscious,
        vehicle: cfg.vehicle.clone(),
        ptt_local: cfg.ptt_local,
        ptt_radio_sr: cfg.ptt_radio_sr,
        ptt_radio_lr: cfg.ptt_radio_lr,
        radio_sr: Some(RadioConfig {
            freq: cfg.freq.clone(),
            channel: cfg.channel,
            volume: 1.0,
            enabled: true,
            range_m: cfg.radio_range_m,
        }),
        radio_lr: Some(RadioConfig {
            freq: cfg.freq_lr.clone(),
            channel: cfg.channel_lr,
            volume: 1.0,
            enabled: !cfg.freq_lr.is_empty(),
            range_m: cfg.radio_range_lr_m,
        }),
    }
}

#[allow(clippy::struct_excessive_bools)]
struct Config {
    steam_id: String,
    server_id: String,
    bridge_addr: String,
    base_pos: [f32; 3],
    orbit: bool,
    orbit_radius: f32,
    orbit_period_s: f32,
    ptt_local: bool,
    // SR radio
    ptt_radio_sr: bool,
    freq: String,
    channel: u8,
    radio_range_m: Option<f32>,
    // LR radio
    ptt_radio_lr: bool,
    freq_lr: String,
    channel_lr: u8,
    radio_range_lr_m: Option<f32>,
    // Player state
    alive: bool,
    conscious: bool,
    vehicle: String,
}

impl Config {
    fn mode_label(&self) -> String {
        if self.orbit {
            format!(
                "orbit (r={:.0} m, T={:.0} s)",
                self.orbit_radius, self.orbit_period_s
            )
        } else {
            format!(
                "static @ [{:.1}, {:.1}, {:.1}]",
                self.base_pos[0], self.base_pos[1], self.base_pos[2]
            )
        }
    }

    fn current_pos(&self, t: f32) -> [f32; 3] {
        if self.orbit {
            let angle = 2.0 * std::f32::consts::PI * t / self.orbit_period_s;
            [
                self.base_pos[0] + self.orbit_radius * angle.sin(),
                self.base_pos[1],
                self.base_pos[2] + self.orbit_radius * angle.cos(),
            ]
        } else {
            self.base_pos
        }
    }

    fn current_dir(&self, t: f32) -> f32 {
        if self.orbit {
            let angle = 2.0 * std::f32::consts::PI * t / self.orbit_period_s;
            (angle.to_degrees() + 90.0).rem_euclid(360.0)
        } else {
            0.0
        }
    }
}

#[allow(clippy::too_many_lines, clippy::similar_names)]
fn parse_args(args: &[String]) -> Result<Config> {
    let mut steam_id = "76561198000000001".to_string();
    let mut server_id = "127.0.0.1:2302".to_string();
    let mut base_pos = [0.0f32; 3];
    let mut orbit = false;
    let mut orbit_radius = 50.0f32;
    let mut orbit_period_s = 30.0f32;
    let mut ptt_local = false;
    let mut ptt_radio_sr = false;
    let mut freq = "152.000".to_string();
    let mut channel: u8 = 1;
    let mut radio_range_m: Option<f32> = None;
    let mut ptt_radio_lr = false;
    let mut freq_lr = String::new();
    let mut channel_lr: u8 = 1;
    let mut radio_range_lr_m: Option<f32> = None;
    let mut alive = true;
    let mut conscious = true;
    let mut vehicle = String::new();
    let mut bridge_addr = DEFAULT_BRIDGE_ADDR.to_string();

    let mut i = 1usize;
    while i < args.len() {
        match args[i].as_str() {
            "--id" => {
                steam_id = next_arg(args, &mut i)?;
            }
            "--server" => {
                server_id = next_arg(args, &mut i)?;
            }
            "--pos" => {
                let s = next_arg(args, &mut i)?;
                let parts: Vec<f32> = s.split(',').filter_map(|x| x.parse().ok()).collect();
                anyhow::ensure!(parts.len() == 3, "--pos requires x,y,z e.g. 100,0,200");
                base_pos = [parts[0], parts[1], parts[2]];
            }
            "--orbit" => {
                orbit = true;
                i += 1;
            }
            "--orbit-radius" => {
                orbit_radius = next_arg(args, &mut i)?.parse().context("--orbit-radius")?;
            }
            "--orbit-period" => {
                orbit_period_s = next_arg(args, &mut i)?.parse().context("--orbit-period")?;
            }
            "--ptt-local" => {
                ptt_local = true;
                i += 1;
            }
            "--ptt-radio" => {
                ptt_radio_sr = true;
                i += 1;
            }
            "--freq" => {
                freq = next_arg(args, &mut i)?;
            }
            "--channel" => {
                channel = next_arg(args, &mut i)?
                    .parse()
                    .context("--channel expects a channel number (1-8)")?;
            }
            "--ptt-radio-lr" => {
                ptt_radio_lr = true;
                i += 1;
            }
            "--freq-lr" => {
                freq_lr = next_arg(args, &mut i)?;
            }
            "--channel-lr" => {
                channel_lr = next_arg(args, &mut i)?
                    .parse()
                    .context("--channel-lr expects a channel number (1-8)")?;
            }
            "--radio-range-lr" => {
                radio_range_lr_m = Some(
                    next_arg(args, &mut i)?
                        .parse()
                        .context("--radio-range-lr expects metres as float")?,
                );
            }
            "--dead" => {
                alive = false;
                i += 1;
            }
            "--unconscious" => {
                conscious = false;
                i += 1;
            }
            "--vehicle" => {
                vehicle = next_arg(args, &mut i)?;
            }
            "--radio-range" => {
                radio_range_m = Some(
                    next_arg(args, &mut i)?
                        .parse()
                        .context("--radio-range expects metres as float")?,
                );
            }
            "--bridge-addr" => {
                bridge_addr = next_arg(args, &mut i)?;
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            other => anyhow::bail!("Unknown argument: {other}. Use --help."),
        }
    }

    Ok(Config {
        steam_id,
        server_id,
        bridge_addr,
        base_pos,
        orbit,
        orbit_radius,
        orbit_period_s,
        ptt_local,
        ptt_radio_sr,
        freq,
        channel,
        radio_range_m,
        ptt_radio_lr,
        freq_lr,
        channel_lr,
        radio_range_lr_m,
        alive,
        conscious,
        vehicle,
    })
}

fn next_arg(args: &[String], i: &mut usize) -> Result<String> {
    *i += 1;
    let v = args
        .get(*i)
        .cloned()
        .context("expected argument after flag")?;
    *i += 1;
    Ok(v)
}

fn print_help() {
    println!("rmtfar-test-client — Simulate an Arma 3 player for RMTFAR testing\n");
    println!("OPTIONS:");
    println!("  --id <steam_id>       Player SteamID64      (default: 76561198000000001)");
    println!("  --server <ip:port>    Server identifier      (default: 127.0.0.1:2302)");
    println!("  --pos <x,y,z>         Static position (m)    (default: 0,0,0)");
    println!("  --orbit               Circular movement around --pos");
    println!("  --orbit-radius <m>    Orbit radius in metres (default: 50)");
    println!("  --orbit-period <s>    Orbit period in seconds (default: 30)");
    println!("  --ptt-local           Activate local PTT (direct voice)");
    println!("  --ptt-radio           Activate SR radio PTT");
    println!("  --freq <freq>         SR radio frequency     (default: 152.000)");
    println!("  --channel <n>         SR radio channel 1-8              (default: 1)");
    println!("  --radio-range <m>     Override SR radio range in metres  (default: 5000)");
    println!("  --ptt-radio-lr        Activate LR radio PTT");
    println!("  --freq-lr <freq>      LR radio frequency     (no default — disables LR)");
    println!("  --channel-lr <n>      LR radio channel 1-8              (default: 1)");
    println!("  --radio-range-lr <m>  Override LR radio range in metres (default: 20000)");
    println!("  --vehicle <classname> Simulate being inside a vehicle (blocks local PTT)");
    println!("  --dead                Simulate dead player (all PTT blocked)");
    println!("  --unconscious         Simulate ACE unconscious (all PTT blocked)");
    println!("  --bridge-addr <h:p>   Bridge address          (default: {DEFAULT_BRIDGE_ADDR})");
    println!("  --help                Print this help\n");
    println!("EXAMPLE - test proximity audio with two terminals:");
    println!("  Terminal 1: rmtfar-test-client --id p1 --pos 0,0,0 --ptt-local");
    println!("  Terminal 2: rmtfar-test-client --id p2 --orbit --ptt-local");
}
