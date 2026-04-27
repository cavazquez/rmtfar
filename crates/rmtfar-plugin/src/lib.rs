// SPDX-License-Identifier: GPL-3.0

//! RMTFAR Mumble Plugin.
//!
//! Implements the Mumble Plugin API (v1.4.0+) to intercept audio streams and
//! apply radio logic:
//! - Phase 1: proximity muting only (positional handled by Mumble Link)
//! - Phase 2+: frequency matching, radio DSP effects
//!
//! The plugin receives [`RadioStateMessage`] from the Arma extension (or bridge) over UDP :9501
//! and uses that to decide mute/volume per user.
//!
//! Every UDP datagram is optionally appended to a log file (see `start()`): by default
//! `{TEMP}/rmtfar-plugin-udp.log` — disable with env `RMTFAR_UDP_LOG=0`, override path with
//! `RMTFAR_UDP_LOG_PATH`.

pub mod audio;
pub mod dsp;
pub mod ffi;
pub mod state;

#[cfg(test)]
mod tests;

use rmtfar_protocol::{distance, RadioStateMessage, LOCAL_VOICE_RANGE_M, PLUGIN_RECV_PORT};
use state::PluginState;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::net::UdpSocket;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// ---------------------------------------------------------------------------
// Global plugin singleton
// ---------------------------------------------------------------------------

static PLUGIN: OnceLock<Mutex<Plugin>> = OnceLock::new();

/// Log fijo de todo el tráfico UDP :9501 (UTF-8 lossy). Windows: `%TEMP%\\rmtfar-plugin-udp.log`.
fn udp_recv_log_path() -> PathBuf {
    std::env::var("RMTFAR_UDP_LOG_PATH").map_or_else(
        |_| std::env::temp_dir().join("rmtfar-plugin-udp.log"),
        PathBuf::from,
    )
}

fn plugin() -> std::sync::MutexGuard<'static, Plugin> {
    PLUGIN
        .get_or_init(|| Mutex::new(Plugin::new()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub(crate) struct Plugin {
    pub(crate) state: PluginState,
    socket: Option<UdpSocket>,
    buf: Vec<u8>,
    /// Cada datagrama recibido en :9501 (una línea por paquete). `None` si `RMTFAR_UDP_LOG=0`.
    udp_recv_log: Option<BufWriter<File>>,
    /// Throttle `tracing::info!` for each UDP `radio_state` (high rate from Arma).
    last_udp_info_log: Option<Instant>,
}

impl Plugin {
    fn new() -> Self {
        Self {
            state: PluginState::default(),
            socket: None,
            buf: vec![0u8; 65535],
            udp_recv_log: None,
            last_udp_info_log: None,
        }
    }

    pub(crate) fn start(&mut self) -> bool {
        // Inicializar tracing hacia stderr.
        // RMTFAR_LOG acepta "error", "warn", "info", "debug" o "trace".
        // journalctl: journalctl --user -f | grep RMTFAR
        let level = std::env::var("RMTFAR_LOG")
            .ok()
            .and_then(|s| s.parse::<tracing::Level>().ok())
            .unwrap_or(tracing::Level::DEBUG);
        let _ = tracing_subscriber::fmt()
            .with_max_level(level)
            .with_writer(std::io::stderr)
            .try_init();

        let log_enabled = std::env::var("RMTFAR_UDP_LOG")
            .map(|v| v != "0" && !v.eq_ignore_ascii_case("false"))
            .unwrap_or(true);

        if log_enabled {
            let path = udp_recv_log_path();
            match OpenOptions::new().create(true).append(true).open(&path) {
                Ok(f) => {
                    let mut w = BufWriter::new(f);
                    let ms = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_millis())
                        .unwrap_or(0);
                    let _ = writeln!(
                        w,
                        "[{ms}] --- RMTFAR UDP log start (every datagram on :{PLUGIN_RECV_PORT}) ---"
                    );
                    let _ = w.flush();
                    self.udp_recv_log = Some(w);
                    tracing::info!(
                        path = %path.display(),
                        "RMTFAR plugin UDP recv log (set RMTFAR_UDP_LOG=0 to disable)"
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        path = %path.display(),
                        error = %e,
                        "RMTFAR plugin: could not open UDP recv log file"
                    );
                }
            }
        }

        match UdpSocket::bind(format!("127.0.0.1:{PLUGIN_RECV_PORT}")) {
            Ok(sock) => {
                sock.set_nonblocking(true).ok();
                self.socket = Some(sock);
                tracing::info!("RMTFAR plugin started, listening on UDP :{PLUGIN_RECV_PORT}");
                true
            }
            Err(e) => {
                tracing::error!("RMTFAR plugin: failed to bind UDP :{PLUGIN_RECV_PORT}: {e}");
                false
            }
        }
    }

    pub(crate) fn stop(&mut self) {
        if let Some(mut w) = self.udp_recv_log.take() {
            let _ = w.flush();
        }
        self.socket = None;
    }

    fn append_udp_recv_log(&mut self, len: usize) {
        let Some(ref mut w) = self.udp_recv_log else {
            return;
        };
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let chunk = &self.buf[..len];
        let text = String::from_utf8_lossy(chunk);
        let _ = writeln!(w, "[{ms}] bytes={len} {text}");
        let _ = w.flush();
    }

    #[allow(clippy::too_many_arguments, clippy::similar_names)]
    fn append_audio_decision_log(
        &mut self,
        mumble_id: u32,
        local_id: Option<&str>,
        sender_id: Option<&str>,
        sender_radio_type: Option<&str>,
        sender_radio_freq: Option<&str>,
        sender_radio_channel: Option<u8>,
        local_tuned_sr: Option<&str>,
        local_tuned_sr_ch: Option<u8>,
        local_tuned_lr: Option<&str>,
        local_tuned_lr_ch: Option<u8>,
        dist: Option<f32>,
        sender_radio_range_m: Option<f32>,
        sender_radio_los: Option<f32>,
        allowed: bool,
        reason: &str,
    ) {
        let Some(ref mut w) = self.udp_recv_log else {
            return;
        };
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let _ = writeln!(
            w,
            "[{ms}] AUDIO mumble_id={} local={} sender={} allow={} reason={} dist_m={} tx_type={} tx_freq={} tx_ch={} tx_range_m={} los={} tuned_sr={}/ch{} tuned_lr={}/ch{}",
            mumble_id,
            local_id.unwrap_or(""),
            sender_id.unwrap_or(""),
            u8::from(allowed),
            reason,
            dist.map_or_else(String::new, |d| format!("{d:.2}")),
            sender_radio_type.unwrap_or(""),
            sender_radio_freq.unwrap_or(""),
            sender_radio_channel.map_or_else(String::new, |c| c.to_string()),
            sender_radio_range_m.map_or_else(String::new, |r| format!("{r:.1}")),
            sender_radio_los.map_or_else(String::new, |l| format!("{l:.2}")),
            local_tuned_sr.unwrap_or(""),
            local_tuned_sr_ch.map_or_else(String::new, |c| c.to_string()),
            local_tuned_lr.unwrap_or(""),
            local_tuned_lr_ch.map_or_else(String::new, |c| c.to_string()),
        );
        let _ = w.flush();
    }

    fn log_radio_state_throttled(&mut self, msg: &RadioStateMessage) {
        const MIN_INTERVAL: Duration = Duration::from_millis(900);
        let now = Instant::now();
        let ok = self
            .last_udp_info_log
            .is_none_or(|t| now.saturating_duration_since(t) >= MIN_INTERVAL);
        if !ok {
            return;
        }
        self.last_udp_info_log = Some(now);

        let mut parts = Vec::with_capacity(msg.players.len());
        for p in &msg.players {
            parts.push(format!(
                "{} alive={} veh={} tunedSR={}/ch{} txRadio={} type={} freq={}/ch{} range_m={:.0}",
                p.player_id,
                u8::from(p.alive),
                if p.in_vehicle { "y" } else { "n" },
                p.tuned_sr_freq,
                p.tuned_sr_channel,
                u8::from(p.transmitting_radio),
                p.radio_type,
                p.radio_freq,
                p.radio_channel,
                p.radio_range_m
            ));
        }
        tracing::info!(
            tick = msg.tick,
            local = %msg.local_player,
            n = msg.players.len(),
            players = %parts.join(" | "),
            "RMTFAR UDP recv radio_state"
        );
    }

    /// Drain the UDP socket, keeping only the latest message.
    pub(crate) fn poll(&mut self) {
        if self.socket.is_none() {
            return;
        }
        loop {
            let len = match self.socket.as_ref().unwrap().recv(self.buf.as_mut_slice()) {
                Ok(n) => n,
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(_) => break,
            };
            self.append_udp_recv_log(len);
            match serde_json::from_slice::<RadioStateMessage>(&self.buf[..len]) {
                Ok(msg) => {
                    self.log_radio_state_throttled(&msg);
                    self.state.update(msg);
                }
                Err(e) => {
                    if let Some(ref mut w) = self.udp_recv_log {
                        let ms = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map(|d| d.as_millis())
                            .unwrap_or(0);
                        let _ = writeln!(w, "[{ms}] PARSE_ERR {e}");
                        let _ = w.flush();
                    }
                    tracing::debug!(error = %e, bytes = len, "UDP payload not RadioStateMessage JSON");
                }
            }
        }
    }

    /// Decide whether user `mumble_id` should be heard.
    /// Returns `false` to mute the user entirely.
    /// Samples are modified in place when returning `true`.
    #[allow(
        clippy::similar_names,
        clippy::too_many_lines,
        clippy::single_match_else
    )]
    pub(crate) fn process_audio(
        &mut self,
        mumble_id: u32,
        samples: &mut [f32],
        sample_rate: u32,
    ) -> bool {
        self.poll();

        let radio = self.state.last_message().cloned();

        let (
            local_pos,
            local_alive,
            local_tuned_sr,
            local_tuned_sr_ch,
            local_tuned_lr,
            local_tuned_lr_ch,
        ) = match radio.as_ref().and_then(|m| m.local()) {
            Some(l) => (
                l.pos,
                l.alive && l.conscious,
                l.tuned_sr_freq.clone(),
                l.tuned_sr_channel,
                l.tuned_lr_freq.clone(),
                l.tuned_lr_channel,
            ),
            None => {
                self.append_audio_decision_log(
                    mumble_id, None, None, None, None, None, None, None, None, None, None, None,
                    None, true, "no_state",
                );
                return true; // no state yet, pass through
            }
        };

        if !local_alive {
            self.append_audio_decision_log(
                mumble_id,
                radio
                    .as_ref()
                    .and_then(|m| m.local())
                    .map(|l| l.player_id.as_str()),
                None,
                None,
                None,
                None,
                Some(&local_tuned_sr),
                Some(local_tuned_sr_ch),
                Some(&local_tuned_lr),
                Some(local_tuned_lr_ch),
                None,
                None,
                None,
                false,
                "local_dead_or_unconscious",
            );
            return false;
        }

        // Find the sender: Mumble session ID → username → player state.
        // The test-client / Arma 3 mod must register players keyed by their
        // Mumble username (--id <mumble-username> in rmtfar-test-client).
        let sender = radio.as_ref().and_then(|m| {
            let name = self.state.name_for_session(mumble_id)?;
            m.players.iter().find(|p| p.player_id == name)
        });

        let Some(sender) = sender else {
            tracing::debug!(mumble_id, "unknown user — pass through");
            self.append_audio_decision_log(
                mumble_id,
                radio
                    .as_ref()
                    .and_then(|m| m.local())
                    .map(|l| l.player_id.as_str()),
                None,
                None,
                None,
                None,
                Some(&local_tuned_sr),
                Some(local_tuned_sr_ch),
                Some(&local_tuned_lr),
                Some(local_tuned_lr_ch),
                None,
                None,
                None,
                true,
                "unknown_user_passthrough",
            );
            return true; // unknown user, pass through
        };

        let dist = distance(&local_pos, &sender.pos);

        if sender.transmitting_radio {
            // Match sender's transmission freq+channel against the local player's tuned
            // freq+channel for that radio type.
            let (local_freq, local_channel) = if sender.radio_type == "lr" {
                (&local_tuned_lr, local_tuned_lr_ch)
            } else {
                (&local_tuned_sr, local_tuned_sr_ch)
            };
            if sender.radio_freq.is_empty() || sender.radio_freq != *local_freq {
                tracing::debug!(
                    uid = %sender.player_id,
                    sender_freq = %sender.radio_freq,
                    local_freq = %local_freq,
                    "radio freq mismatch — muted"
                );
                self.append_audio_decision_log(
                    mumble_id,
                    radio
                        .as_ref()
                        .and_then(|m| m.local())
                        .map(|l| l.player_id.as_str()),
                    Some(&sender.player_id),
                    Some(&sender.radio_type),
                    Some(&sender.radio_freq),
                    Some(sender.radio_channel),
                    Some(&local_tuned_sr),
                    Some(local_tuned_sr_ch),
                    Some(&local_tuned_lr),
                    Some(local_tuned_lr_ch),
                    Some(dist),
                    Some(sender.radio_range_m),
                    Some(sender.radio_los_quality),
                    false,
                    "radio_freq_mismatch",
                );
                return false;
            }
            if sender.radio_channel != local_channel {
                tracing::debug!(
                    uid = %sender.player_id,
                    sender_ch = sender.radio_channel,
                    local_ch = local_channel,
                    "radio channel mismatch — muted"
                );
                self.append_audio_decision_log(
                    mumble_id,
                    radio
                        .as_ref()
                        .and_then(|m| m.local())
                        .map(|l| l.player_id.as_str()),
                    Some(&sender.player_id),
                    Some(&sender.radio_type),
                    Some(&sender.radio_freq),
                    Some(sender.radio_channel),
                    Some(&local_tuned_sr),
                    Some(local_tuned_sr_ch),
                    Some(&local_tuned_lr),
                    Some(local_tuned_lr_ch),
                    Some(dist),
                    Some(sender.radio_range_m),
                    Some(sender.radio_los_quality),
                    false,
                    "radio_channel_mismatch",
                );
                return false;
            }
            if dist > sender.radio_range_m {
                tracing::debug!(uid = %sender.player_id, dist, "out of radio range — muted");
                self.append_audio_decision_log(
                    mumble_id,
                    radio
                        .as_ref()
                        .and_then(|m| m.local())
                        .map(|l| l.player_id.as_str()),
                    Some(&sender.player_id),
                    Some(&sender.radio_type),
                    Some(&sender.radio_freq),
                    Some(sender.radio_channel),
                    Some(&local_tuned_sr),
                    Some(local_tuned_sr_ch),
                    Some(&local_tuned_lr),
                    Some(local_tuned_lr_ch),
                    Some(dist),
                    Some(sender.radio_range_m),
                    Some(sender.radio_los_quality),
                    false,
                    "radio_out_of_range",
                );
                return false;
            }
            let signal_quality = (1.0 - (dist / sender.radio_range_m).clamp(0.0, 1.0))
                * sender.radio_los_quality.clamp(0.0, 1.0);
            tracing::debug!(
                uid = %sender.player_id,
                dist,
                signal_quality,
                "radio — applying DSP"
            );
            dsp::apply_radio_effect(samples, sample_rate, signal_quality);
            self.append_audio_decision_log(
                mumble_id,
                radio
                    .as_ref()
                    .and_then(|m| m.local())
                    .map(|l| l.player_id.as_str()),
                Some(&sender.player_id),
                Some(&sender.radio_type),
                Some(&sender.radio_freq),
                Some(sender.radio_channel),
                Some(&local_tuned_sr),
                Some(local_tuned_sr_ch),
                Some(&local_tuned_lr),
                Some(local_tuned_lr_ch),
                Some(dist),
                Some(sender.radio_range_m),
                Some(sender.radio_los_quality),
                true,
                "radio_dsp",
            );
            true
        } else if sender.transmitting_local {
            if dist > LOCAL_VOICE_RANGE_M {
                tracing::debug!(uid = %sender.player_id, dist, "out of local range — muted");
                self.append_audio_decision_log(
                    mumble_id,
                    radio
                        .as_ref()
                        .and_then(|m| m.local())
                        .map(|l| l.player_id.as_str()),
                    Some(&sender.player_id),
                    Some("local"),
                    None,
                    None,
                    Some(&local_tuned_sr),
                    Some(local_tuned_sr_ch),
                    Some(&local_tuned_lr),
                    Some(local_tuned_lr_ch),
                    Some(dist),
                    Some(LOCAL_VOICE_RANGE_M),
                    None,
                    false,
                    "local_out_of_range",
                );
                return false;
            }
            let volume = 1.0 - (dist / LOCAL_VOICE_RANGE_M).clamp(0.0, 1.0);
            tracing::debug!(uid = %sender.player_id, dist, volume, "local voice");
            audio::apply_volume(samples, volume);
            self.append_audio_decision_log(
                mumble_id,
                radio
                    .as_ref()
                    .and_then(|m| m.local())
                    .map(|l| l.player_id.as_str()),
                Some(&sender.player_id),
                Some("local"),
                None,
                None,
                Some(&local_tuned_sr),
                Some(local_tuned_sr_ch),
                Some(&local_tuned_lr),
                Some(local_tuned_lr_ch),
                Some(dist),
                Some(LOCAL_VOICE_RANGE_M),
                None,
                true,
                "local_voice",
            );
            true
        } else {
            let reason = if !sender.alive {
                "sender_dead"
            } else if !sender.conscious {
                "sender_unconscious"
            } else if sender.in_vehicle && !sender.transmitting_radio {
                "sender_in_vehicle_no_radio_ptt"
            } else {
                "sender_not_transmitting"
            };
            if !sender.alive {
                tracing::debug!(uid = %sender.player_id, "dead — muted");
            } else if !sender.conscious {
                tracing::debug!(uid = %sender.player_id, "unconscious — muted");
            } else if sender.in_vehicle && !sender.transmitting_radio {
                tracing::debug!(uid = %sender.player_id, "in vehicle, no radio PTT — muted");
            } else {
                tracing::debug!(uid = %sender.player_id, "not transmitting — muted");
            }
            self.append_audio_decision_log(
                mumble_id,
                radio
                    .as_ref()
                    .and_then(|m| m.local())
                    .map(|l| l.player_id.as_str()),
                Some(&sender.player_id),
                Some(&sender.radio_type),
                Some(&sender.radio_freq),
                Some(sender.radio_channel),
                Some(&local_tuned_sr),
                Some(local_tuned_sr_ch),
                Some(&local_tuned_lr),
                Some(local_tuned_lr_ch),
                Some(dist),
                Some(sender.radio_range_m),
                Some(sender.radio_los_quality),
                false,
                reason,
            );
            false
        }
    }
}
