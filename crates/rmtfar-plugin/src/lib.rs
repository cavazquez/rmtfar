// SPDX-License-Identifier: GPL-3.0

//! RMTFAR Mumble Plugin.
//!
//! Implements the Mumble Plugin API (v1.4.0+) to intercept audio streams and
//! apply radio logic:
//! - Phase 1: proximity muting only (positional handled by Mumble Link)
//! - Phase 2+: frequency matching, radio DSP effects
//!
//! The plugin receives [`RadioStateMessage`] from the bridge over UDP :9501
//! and uses that to decide mute/volume per user.

pub mod audio;
pub mod dsp;
pub mod ffi;
pub mod state;

use rmtfar_protocol::{distance, RadioStateMessage, LOCAL_VOICE_RANGE_M, PLUGIN_RECV_PORT};
use state::PluginState;
use std::net::UdpSocket;
use std::sync::{Mutex, OnceLock};

// ---------------------------------------------------------------------------
// Global plugin singleton
// ---------------------------------------------------------------------------

static PLUGIN: OnceLock<Mutex<Plugin>> = OnceLock::new();

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
}

impl Plugin {
    fn new() -> Self {
        Self {
            state: PluginState::default(),
            socket: None,
            buf: vec![0u8; 65535],
        }
    }

    pub(crate) fn start(&mut self) -> bool {
        // Inicializar tracing hacia stderr para que los logs aparezcan en
        // journalctl: journalctl --user -f | grep RMTFAR
        let _ = tracing_subscriber::fmt()
            .with_env_filter(
                std::env::var("RMTFAR_LOG").unwrap_or_else(|_| "rmtfar_plugin=debug".into()),
            )
            .with_writer(std::io::stderr)
            .try_init();

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
        self.socket = None;
    }

    /// Drain the UDP socket, keeping only the latest message.
    pub(crate) fn poll(&mut self) {
        let Some(ref sock) = self.socket else { return };
        loop {
            match sock.recv(self.buf.as_mut_slice()) {
                Ok(len) => {
                    if let Ok(msg) = serde_json::from_slice::<RadioStateMessage>(&self.buf[..len]) {
                        self.state.update(msg);
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(_) => break,
            }
        }
    }

    /// Decide whether user `mumble_id` should be heard.
    /// Returns `false` to mute the user entirely.
    /// Samples are modified in place when returning `true`.
    pub(crate) fn process_audio(
        &mut self,
        mumble_id: u32,
        samples: &mut [f32],
        sample_rate: u32,
    ) -> bool {
        self.poll();

        let radio = self.state.last_message();

        let (local_pos, local_alive, local_tuned_sr, local_tuned_sr_ch, local_tuned_lr, local_tuned_lr_ch) =
            match radio.as_ref().and_then(|m| m.local()) {
                Some(l) => (
                    l.pos,
                    l.alive && l.conscious,
                    l.tuned_sr_freq.clone(),
                    l.tuned_sr_channel,
                    l.tuned_lr_freq.clone(),
                    l.tuned_lr_channel,
                ),
                None => return true, // no state yet, pass through
            };

        if !local_alive {
            return false;
        }

        // Find the sender: Mumble session ID → username → player state.
        // The test-client / Arma 3 mod must register players keyed by their
        // Mumble username (--id <mumble-username> in rmtfar-test-client).
        let sender = radio.as_ref().and_then(|m| {
            let name = self.state.name_for_session(mumble_id)?;
            m.players.iter().find(|p| p.steam_id == name)
        });

        let Some(sender) = sender else {
            tracing::debug!(mumble_id, "unknown user — pass through");
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
                    uid = %sender.steam_id,
                    sender_freq = %sender.radio_freq,
                    local_freq = %local_freq,
                    "radio freq mismatch — muted"
                );
                return false;
            }
            if sender.radio_channel != local_channel {
                tracing::debug!(
                    uid = %sender.steam_id,
                    sender_ch = sender.radio_channel,
                    local_ch = local_channel,
                    "radio channel mismatch — muted"
                );
                return false;
            }
            if dist > sender.radio_range_m {
                tracing::debug!(uid = %sender.steam_id, dist, "out of radio range — muted");
                return false;
            }
            tracing::debug!(uid = %sender.steam_id, dist, "radio — applying DSP");
            dsp::apply_radio_effect(samples, sample_rate, dist, sender.radio_range_m);
            true
        } else if sender.transmitting_local {
            if dist > LOCAL_VOICE_RANGE_M {
                tracing::debug!(uid = %sender.steam_id, dist, "out of local range — muted");
                return false;
            }
            let volume = 1.0 - (dist / LOCAL_VOICE_RANGE_M).clamp(0.0, 1.0);
            tracing::debug!(uid = %sender.steam_id, dist, volume, "local voice");
            audio::apply_volume(samples, volume);
            true
        } else {
            tracing::debug!(uid = %sender.steam_id, "not transmitting — muted");
            false
        }
    }
}
