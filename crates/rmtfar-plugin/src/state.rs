// SPDX-License-Identifier: GPL-3.0

//! Plugin-side state: the latest `RadioStateMessage` and identity mapping.
//!
//! ## Identity mapping
//!
//! Mumble identifies users by a numeric session ID that changes every connection.
//! Arma 3 / the test-client identify players by their **Arma 3 profile name**
//! (the value of `name player` in SQF, e.g. "Cristian").
//!
//! We bridge the two via the Mumble username:
//! - `mumble_onUserAdded` queries the API for the username and stores
//!   `session_id → username` via [`PluginState::register_session`].
//! - Each player must set their **Mumble nickname** to match their Arma 3 profile name.

use rmtfar_protocol::RadioStateMessage;
use std::collections::HashMap;

#[derive(Default)]
pub struct PluginState {
    last: Option<RadioStateMessage>,
    /// Mumble numeric session ID → Mumble username.
    session_to_name: HashMap<u32, String>,
}

impl PluginState {
    pub fn update(&mut self, msg: RadioStateMessage) {
        self.last = Some(msg);
    }

    pub fn last_message(&self) -> Option<&RadioStateMessage> {
        self.last.as_ref()
    }

    /// Cache `mumble_session_id → username` obtained from the Mumble API.
    pub fn register_session(&mut self, session_id: u32, name: String) {
        self.session_to_name.insert(session_id, name);
    }

    /// Remove a session when the user disconnects.
    pub fn unregister_session(&mut self, session_id: u32) {
        self.session_to_name.remove(&session_id);
    }

    /// Resolve a Mumble session ID to a username.
    pub fn name_for_session(&self, session_id: u32) -> Option<&str> {
        self.session_to_name.get(&session_id).map(String::as_str)
    }

    // Legacy stub — kept so ffi.rs compiles. Not needed with username-based mapping.
    pub fn register_identity(&mut self, _mumble_id: &str, _player_id: String) {}
}
