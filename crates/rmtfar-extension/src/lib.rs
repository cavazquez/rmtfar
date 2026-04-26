//! Arma 3 Extension DLL for RMTFAR.
//!
//! Exposes the `RVExtension` / `RVExtensionArgs` C ABI required by Arma 3.
//!
//! Instead of forwarding data to a separate bridge process, the extension
//! handles everything directly — just like TFAR does with `TeamSpeak`:
//!
//! 1. Parses incoming `PlayerState` JSON from SQF
//! 2. Stores state for all players
//! 3. Writes `MumbleLink` shared memory (positional audio)
//! 4. Builds and sends `RadioStateMessage` to the Mumble plugin via UDP :9501

use std::ffi::{c_char, c_int, CStr};
use std::sync::{Mutex, OnceLock};

mod mumble_link;
mod sender;
mod state;

use mumble_link::MumbleLink;
use rmtfar_protocol::{PlayerState, PlayerSummary, RadioStateMessage, PROTOCOL_VERSION};
use sender::PluginSender;
use state::PlayerStore;

const VERSION: &str = env!("CARGO_PKG_VERSION");

struct ExtensionState {
    store: PlayerStore,
    mumble: MumbleLink,
    sender: PluginSender,
    local_id: Option<String>,
}

static STATE: OnceLock<Mutex<ExtensionState>> = OnceLock::new();

fn get_state() -> &'static Mutex<ExtensionState> {
    STATE.get_or_init(|| {
        Mutex::new(ExtensionState {
            store: PlayerStore::new(),
            mumble: MumbleLink::new(),
            sender: PluginSender::new(),
            local_id: None,
        })
    })
}

// ---------------------------------------------------------------------------
// Core logic
// ---------------------------------------------------------------------------

fn handle_init(local_id: &str) -> &'static str {
    if let Ok(mut s) = get_state().lock() {
        s.local_id = Some(local_id.to_string());
        "0"
    } else {
        "ERR:lock"
    }
}

fn handle_send(json: &[u8]) -> Result<(), String> {
    let player_state: PlayerState =
        serde_json::from_slice(json).map_err(|e| format!("json: {e}"))?;

    if player_state.v != PROTOCOL_VERSION {
        return Err(format!("bad version: {}", player_state.v));
    }

    let mut s = get_state().lock().map_err(|e| format!("lock: {e}"))?;

    let is_local = s
        .local_id
        .as_ref()
        .is_some_and(|id| id == &player_state.steam_id);

    if is_local {
        s.mumble.update(&player_state);
    }

    s.store.update(player_state);

    let local_id = s.local_id.clone().unwrap_or_default();
    let msg = build_message(&s.store, &local_id);

    if let Ok(json) = serde_json::to_vec(&msg) {
        let _ = s.sender.send(&json);
    }

    Ok(())
}

fn build_message(store: &PlayerStore, local_id: &str) -> RadioStateMessage {
    let (server_id, tick) = store
        .get(local_id)
        .map(|s| (s.server_id.clone(), s.tick))
        .unwrap_or_default();

    let players = store.all().map(PlayerSummary::from_state).collect();

    RadioStateMessage::new(server_id, tick, local_id.to_string(), players)
}

// ---------------------------------------------------------------------------
// Arma 3 C ABI entry points
// ---------------------------------------------------------------------------

/// Called by Arma 3 for zero-argument extension calls: `"rmtfar" callExtension "cmd"`
///
/// # Safety
/// Pointers provided by Arma 3 are always valid within the call.
#[no_mangle]
pub unsafe extern "C" fn RVExtension(
    output: *mut c_char,
    output_size: c_int,
    function: *const c_char,
) {
    let func = CStr::from_ptr(function).to_string_lossy();
    let result = match func.as_ref() {
        "version" => VERSION,
        "ping" => "pong",
        _ => "",
    };
    write_output(output, output_size, result);
}

/// Called by Arma 3 for multi-argument extension calls:
/// `"rmtfar" callExtension ["cmd", [arg1, ...]]`
///
/// Returns 0 on success, negative on error.
///
/// # Safety
/// Pointers provided by Arma 3 are always valid within the call.
#[no_mangle]
pub unsafe extern "C" fn RVExtensionArgs(
    output: *mut c_char,
    output_size: c_int,
    function: *const c_char,
    args: *const *const c_char,
    arg_count: c_int,
) -> c_int {
    let func = CStr::from_ptr(function).to_string_lossy();

    match func.as_ref() {
        "init" if arg_count >= 1 => {
            let local_id = CStr::from_ptr(*args).to_string_lossy();
            let result = handle_init(&local_id);
            write_output(output, output_size, result);
            0
        }
        "send" if arg_count >= 1 => {
            let json = CStr::from_ptr(*args).to_string_lossy();
            match handle_send(json.as_bytes()) {
                Ok(()) => {
                    write_output(output, output_size, "0");
                    0
                }
                Err(e) => {
                    let msg = format!("ERR:{e}");
                    write_output(output, output_size, &msg);
                    -1
                }
            }
        }
        "version" => {
            write_output(output, output_size, VERSION);
            0
        }
        _ => {
            write_output(output, output_size, "ERR:unknown");
            -2
        }
    }
}

/// Version string callback (optional but recommended).
///
/// # Safety
/// Pointers provided by Arma 3 are always valid within the call.
#[no_mangle]
pub unsafe extern "C" fn RVExtensionVersion(output: *mut c_char, output_size: c_int) {
    write_output(output, output_size, VERSION);
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

unsafe fn write_output(output: *mut c_char, size: c_int, data: &str) {
    if output.is_null() || size <= 0 {
        return;
    }
    let bytes = data.as_bytes();
    #[allow(clippy::cast_sign_loss)]
    let capacity = (size as usize).saturating_sub(1);
    let len = bytes.len().min(capacity);
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), output.cast::<u8>(), len);
    *output.add(len) = 0;
}
