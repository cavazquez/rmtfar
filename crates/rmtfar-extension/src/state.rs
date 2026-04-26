//! In-memory store of all known player states.

use rmtfar_protocol::PlayerState;
use std::collections::HashMap;

pub struct PlayerStore {
    players: HashMap<String, PlayerState>,
}

impl PlayerStore {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
        }
    }

    pub fn update(&mut self, state: PlayerState) {
        self.players.insert(state.player_id.clone(), state);
    }

    pub fn get(&self, player_id: &str) -> Option<&PlayerState> {
        self.players.get(player_id)
    }

    pub fn all(&self) -> impl Iterator<Item = &PlayerState> {
        self.players.values()
    }
}
