//! Arma 3 Extension DLL for RMTFAR.
//!
//! Exposes the `RVExtension` / `RVExtensionArgs` C ABI required by Arma 3.
//!
//! Instead of forwarding data to a separate bridge process, the extension
//! handles everything directly — just like TFAR does with `TeamSpeak`:
//!
//! 1. Parses incoming `PlayerState` payload from SQF (`v1|...`)
//! 2. Stores state for all players
//! 3. Writes `MumbleLink` shared memory (positional audio)
//! 4. Builds and sends `RadioStateMessage` to the Mumble plugin via UDP :9501

use std::ffi::{c_char, c_int, CStr};
use std::sync::{Mutex, OnceLock};

mod mumble_link;
mod sender;
mod state;

use mumble_link::MumbleLink;
use rmtfar_protocol::{
    PlayerState, PlayerSummary, RadioConfig, RadioStateMessage, PROTOCOL_VERSION,
};
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

/// Parser principal para `send`.
/// Formato único soportado: `v1|...`
fn parse_player_state(raw: &[u8]) -> Result<PlayerState, String> {
    let s = std::str::from_utf8(raw).map_err(|e| format!("utf8: {e}"))?;
    let payload = normalize_v1_payload(s)?;
    if !payload.starts_with("v1|") {
        return Err("unsupported payload format; expected v1|...".to_string());
    }
    parse_player_state_v1(&payload)
}

/// Arma `callExtension` a veces envuelve el argumento como string JSON
/// (`"v1|..."`) o como array de un elemento (`["v1|..."]`).
/// Seguimos aceptando SOLO protocolo v1, pero desempaquetamos esos wrappers.
fn normalize_v1_payload(s: &str) -> Result<String, String> {
    let s = s.trim();
    if s.starts_with("v1|") {
        return Ok(s.to_string());
    }

    // Caso 1: payload envuelto como string JSON.
    if s.starts_with('"') && s.ends_with('"') {
        let inner =
            serde_json::from_str::<String>(s).map_err(|e| format!("v1 wrapper string: {e}"))?;
        return Ok(inner);
    }

    // Caso 2: payload envuelto como array JSON de un solo string.
    if s.starts_with('[') && s.ends_with(']') {
        let arr =
            serde_json::from_str::<Vec<String>>(s).map_err(|e| format!("v1 wrapper array: {e}"))?;
        if arr.len() == 1 {
            return Ok(arr.into_iter().next().unwrap_or_default());
        }
    }

    Err("unsupported payload format; expected v1|...".to_string())
}

#[allow(clippy::similar_names)] // Protocol field names are intentionally parallel (sr/lr).
fn parse_player_state_v1(s: &str) -> Result<PlayerState, String> {
    let fields = split_escaped_pipe(s);
    if fields.len() != 18 {
        return Err(format!("v1: bad field count {}, expected 18", fields.len()));
    }
    if fields[0] != "v1" {
        return Err("v1: missing prefix".to_string());
    }
    let steam_id = unescape_pipe_field(&fields[1]);
    let server_id = unescape_pipe_field(&fields[2]);
    let tick = fields[3]
        .parse::<u64>()
        .map_err(|e| format!("v1: tick: {e}"))?;
    let x = fields[4]
        .parse::<f32>()
        .map_err(|e| format!("v1: x: {e}"))?;
    let y = fields[5]
        .parse::<f32>()
        .map_err(|e| format!("v1: y: {e}"))?;
    let z = fields[6]
        .parse::<f32>()
        .map_err(|e| format!("v1: z: {e}"))?;
    let dir = fields[7]
        .parse::<f32>()
        .map_err(|e| format!("v1: dir: {e}"))?;
    let alive = fields[8] == "1";
    let conscious = fields[9] == "1";
    let vehicle = unescape_pipe_field(&fields[10]);
    let ptt_local = fields[11] == "1";
    let ptt_radio_sr = fields[12] == "1";
    let ptt_radio_lr = fields[13] == "1";
    let sr_freq = unescape_pipe_field(&fields[14]);
    let sr_ch = fields[15]
        .parse::<u8>()
        .map_err(|e| format!("v1: sr_ch: {e}"))?;
    let lr_freq = unescape_pipe_field(&fields[16]);
    let lr_ch = fields[17]
        .parse::<u8>()
        .map_err(|e| format!("v1: lr_ch: {e}"))?;

    let radio_sr = Some(RadioConfig {
        freq: sr_freq,
        channel: sr_ch,
        volume: 1.0,
        enabled: true,
        range_m: None,
    });
    let radio_lr = if lr_freq.is_empty() {
        None
    } else {
        Some(RadioConfig {
            freq: lr_freq,
            channel: lr_ch,
            volume: 1.0,
            enabled: true,
            range_m: None,
        })
    };

    Ok(PlayerState {
        v: PROTOCOL_VERSION,
        msg_type: "player_state".to_string(),
        steam_id,
        server_id,
        tick,
        pos: [x, y, z],
        dir,
        alive,
        conscious,
        vehicle,
        ptt_local,
        ptt_radio_sr,
        ptt_radio_lr,
        radio_sr,
        radio_lr,
    })
}

fn split_escaped_pipe(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut esc = false;
    for ch in s.chars() {
        if esc {
            cur.push(ch);
            esc = false;
            continue;
        }
        match ch {
            '\\' => esc = true,
            '|' => {
                out.push(cur);
                cur = String::new();
            }
            _ => cur.push(ch),
        }
    }
    out.push(cur);
    out
}

fn unescape_pipe_field(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut esc = false;
    for ch in s.chars() {
        if esc {
            out.push(ch);
            esc = false;
        } else if ch == '\\' {
            esc = true;
        } else {
            out.push(ch);
        }
    }
    out
}

fn handle_send(json: &[u8]) -> Result<(), String> {
    let player_state = parse_player_state(json)?;

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
