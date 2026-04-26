//! Plugin-side state: the latest RadioStateMessage and identity mapping.

use rmtfar_protocol::RadioStateMessage;
use std::collections::HashMap;

#[derive(Default)]
pub struct PluginState {
    last: Option<RadioStateMessage>,
    /// Mumble numeric user ID (as string) → SteamID64
    mumble_to_steam: HashMap<String, String>,
}

impl PluginState {
    pub fn update(&mut self, msg: RadioStateMessage) {
        self.last = Some(msg);
    }

    pub fn last_message(&self) -> Option<&RadioStateMessage> {
        self.last.as_ref()
    }

    /// Register a mapping from Mumble's numeric user ID to a SteamID64.
    pub fn register_identity(&mut self, mumble_id: &str, steam_id: String) {
        self.mumble_to_steam.insert(mumble_id.to_string(), steam_id);
    }

    pub fn mumble_id_to_steam(&self, mumble_id: &str) -> Option<&String> {
        self.mumble_to_steam.get(mumble_id)
    }
}
