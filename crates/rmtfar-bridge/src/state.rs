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
        self.players.insert(state.steam_id.clone(), state);
    }

    pub fn get(&self, steam_id: &str) -> Option<&PlayerState> {
        self.players.get(steam_id)
    }

    pub fn all(&self) -> impl Iterator<Item = &PlayerState> {
        self.players.values()
    }

    /// Remove players whose tick is older than `max_age_ticks` behind the newest.
    pub fn evict_stale(&mut self, max_age_ticks: u64) {
        if self.players.is_empty() {
            return;
        }
        let newest = self.players.values().map(|p| p.tick).max().unwrap_or(0);
        self.players
            .retain(|_, p| newest.saturating_sub(p.tick) <= max_age_ticks);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmtfar_protocol::PlayerState;

    fn make(id: &str, tick: u64) -> PlayerState {
        PlayerState {
            v: 1,
            msg_type: "player_state".into(),
            steam_id: id.to_string(),
            server_id: "srv".into(),
            tick,
            pos: [0.0; 3],
            dir: 0.0,
            alive: true,
            conscious: true,
            vehicle: String::new(),
            ptt_local: false,
            ptt_radio_sr: false,
            ptt_radio_lr: false,
            radio_sr: None,
            radio_lr: None,
        }
    }

    #[test]
    fn update_and_retrieve() {
        let mut store = PlayerStore::new();
        store.update(make("A", 1));
        assert!(store.get("A").is_some());
        assert!(store.get("B").is_none());
    }

    #[test]
    fn evict_stale() {
        let mut store = PlayerStore::new();
        store.update(make("A", 100));
        store.update(make("B", 10));
        store.evict_stale(50);
        assert!(store.get("A").is_some());
        assert!(store.get("B").is_none());
    }
}
