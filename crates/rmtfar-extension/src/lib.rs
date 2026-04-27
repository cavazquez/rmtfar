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

use std::ffi::{CStr, c_char, c_int};
use std::sync::{Mutex, OnceLock};

mod mumble_link;
mod sender;
mod state;

use mumble_link::MumbleLink;
use rmtfar_protocol::{
    PROTOCOL_VERSION, PlayerState, PlayerSummary, RadioConfig, RadioStateMessage,
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

fn handle_init(local_id_raw: &str) -> Result<(), String> {
    let local_id = normalize_single_string_arg(local_id_raw)?;
    let mut s = get_state().lock().map_err(|e| format!("lock: {e}"))?;
    s.local_id = Some(local_id);
    Ok(())
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

/// Arma `callExtension` puede envolver un único string como JSON:
/// - `"valor"` o `["valor"]`
/// - `valor` plano
fn normalize_single_string_arg(s: &str) -> Result<String, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("empty string argument".to_string());
    }

    if s.starts_with('"') && s.ends_with('"') {
        return serde_json::from_str::<String>(s).map_err(|e| format!("string wrapper: {e}"));
    }

    if s.starts_with('[') && s.ends_with(']') {
        let arr =
            serde_json::from_str::<Vec<String>>(s).map_err(|e| format!("array wrapper: {e}"))?;
        if arr.len() == 1 {
            return Ok(arr.into_iter().next().unwrap_or_default());
        }
        return Err("array wrapper must contain exactly one string".to_string());
    }

    Ok(s.to_string())
}

fn parse_optional_radio_range_m(raw: &str, label: &str) -> Result<Option<f32>, String> {
    if raw.is_empty() || raw == "0" {
        return Ok(None);
    }
    let v: f32 = raw.parse().map_err(|e| format!("v1: {label}: {e}"))?;
    if v <= 0.0 { Ok(None) } else { Ok(Some(v)) }
}

#[allow(clippy::similar_names)] // Protocol field names are intentionally parallel (sr/lr).
#[allow(clippy::many_single_char_names)] // x, y, z are coordinate names; s and n are idiomatic.
#[allow(clippy::too_many_lines)] // v1 parser is verbose by design to keep field mapping explicit.
fn parse_player_state_v1(s: &str) -> Result<PlayerState, String> {
    let fields = split_escaped_pipe(s);
    if fields.is_empty() || fields[0] != "v1" {
        return Err("v1: missing prefix".to_string());
    }
    let n = fields.len();
    if !(n == 18 || n == 19 || n == 21 || n == 23 || n == 25 || n == 27 || n == 28) {
        return Err(format!(
            "v1: bad field count {n}, expected 18, 19, 21, 23, 25, 27, or 28 (radio_los + optional sr/lr range_m + optional stereo + optional code + optional intercom + optional intercom_vehicle_id)"
        ));
    }
    let radio_los_quality = match n {
        18 => 1.0,
        _ => fields[18]
            .parse::<f32>()
            .map_err(|e| format!("v1: radio_los: {e}"))?
            .clamp(0.0, 1.0),
    };
    let (sr_range_m, lr_range_m) = if n >= 21 {
        (
            parse_optional_radio_range_m(&fields[19], "sr_range_m")?,
            parse_optional_radio_range_m(&fields[20], "lr_range_m")?,
        )
    } else {
        (None, None)
    };
    let (sr_stereo, lr_stereo) = if n >= 23 {
        let srs = fields[21]
            .parse::<u8>()
            .map_err(|e| format!("v1: sr_stereo: {e}"))?;
        let lrs = fields[22]
            .parse::<u8>()
            .map_err(|e| format!("v1: lr_stereo: {e}"))?;
        (srs.min(2), lrs.min(2))
    } else {
        (0, 0)
    };
    let (sr_code, lr_code) = if n >= 25 {
        (
            unescape_pipe_field(&fields[23]),
            unescape_pipe_field(&fields[24]),
        )
    } else {
        (String::new(), String::new())
    };
    let (intercom_enabled, intercom_channel) = if n >= 27 {
        let en = fields[25] == "1";
        let ch = fields[26]
            .parse::<u8>()
            .map_err(|e| format!("v1: intercom_channel: {e}"))?;
        (en, ch.max(1))
    } else {
        (true, 1)
    };
    let intercom_vehicle_id = if n >= 28 {
        unescape_pipe_field(&fields[27])
    } else {
        String::new()
    };
    let player_id = unescape_pipe_field(&fields[1]);
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
        range_m: sr_range_m,
        stereo: sr_stereo,
        code: sr_code,
    });
    let radio_lr = if lr_freq.is_empty() {
        None
    } else {
        Some(RadioConfig {
            freq: lr_freq,
            channel: lr_ch,
            volume: 1.0,
            enabled: true,
            range_m: lr_range_m,
            stereo: lr_stereo,
            code: lr_code,
        })
    };

    Ok(PlayerState {
        v: PROTOCOL_VERSION,
        msg_type: "player_state".to_string(),
        player_id,
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
        radio_los_quality,
        intercom_enabled,
        intercom_channel,
        intercom_vehicle_id,
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
        .is_some_and(|id| id == &player_state.player_id);

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

fn handle_forget(player_id: &str) -> Result<(), String> {
    let mut s = get_state().lock().map_err(|e| format!("lock: {e}"))?;
    let _ = s.store.remove(player_id);
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RVExtension(
    output: *mut c_char,
    output_size: c_int,
    function: *const c_char,
) {
    let func = unsafe { CStr::from_ptr(function) }.to_string_lossy();
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RVExtensionArgs(
    output: *mut c_char,
    output_size: c_int,
    function: *const c_char,
    args: *const *const c_char,
    arg_count: c_int,
) -> c_int {
    let func = unsafe { CStr::from_ptr(function) }.to_string_lossy();

    match func.as_ref() {
        "init" if arg_count >= 1 => {
            let local_id = unsafe { CStr::from_ptr(*args) }.to_string_lossy();
            match handle_init(&local_id) {
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
        "send" if arg_count >= 1 => {
            let json = unsafe { CStr::from_ptr(*args) }.to_string_lossy();
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
        "forget" if arg_count >= 1 => {
            let id = unsafe { CStr::from_ptr(*args) }.to_string_lossy();
            match handle_forget(id.trim()) {
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RVExtensionVersion(output: *mut c_char, output_size: c_int) {
    write_output(output, output_size, VERSION);
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn write_output(output: *mut c_char, size: c_int, data: &str) {
    if output.is_null() || size <= 0 {
        return;
    }
    let bytes = data.as_bytes();
    #[allow(clippy::cast_sign_loss)]
    let capacity = (size as usize).saturating_sub(1);
    let len = bytes.len().min(capacity);
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), output.cast::<u8>(), len);
        *output.add(len) = 0;
    }
}

#[cfg(test)]
mod v1_tests {
    use super::{
        normalize_single_string_arg, normalize_v1_payload, parse_player_state,
        parse_player_state_v1, split_escaped_pipe, unescape_pipe_field,
    };

    // ── parse_player_state_v1 ────────────────────────────────────────────────

    #[test]
    fn v1_18_fields_defaults_los() {
        let s = "v1|76561198000000000|srv|1000|0|0|0|0|1|1||0|0|0|152.000|1||1";
        let p = parse_player_state_v1(s).unwrap();
        assert!((p.radio_los_quality - 1.0).abs() < f32::EPSILON);
        assert!(p.radio_sr.is_some());
        assert!(p.radio_lr.is_none());
    }

    #[test]
    fn v1_19_fields_parses_los() {
        let s = "v1|76561198000000000|srv|1000|0|0|0|0|1|1||0|0|0|152.000|1||1|0.35";
        let p = parse_player_state_v1(s).unwrap();
        assert!((p.radio_los_quality - 0.35).abs() < 1e-5);
        assert!(p.radio_sr.as_ref().unwrap().range_m.is_none());
    }

    #[test]
    fn v1_21_fields_parses_ranges() {
        let s = "v1|76561198000000000|srv|1000|0|0|0|0|1|1||0|0|0|152.000|1|30.0|1|1|3000|12000";
        let p = parse_player_state_v1(s).unwrap();
        assert!((p.radio_sr.as_ref().unwrap().range_m.unwrap() - 3000.0).abs() < 1e-3);
        assert!((p.radio_lr.as_ref().unwrap().range_m.unwrap() - 12000.0).abs() < 1e-3);
    }

    #[test]
    fn v1_23_fields_parses_stereo() {
        let s =
            "v1|76561198000000000|srv|1000|0|0|0|0|1|1||0|0|0|152.000|1|30.0|1|1|3000|12000|1|2";
        let p = parse_player_state_v1(s).unwrap();
        assert_eq!(p.radio_sr.as_ref().unwrap().stereo, 1);
        assert_eq!(p.radio_lr.as_ref().unwrap().stereo, 2);
    }

    #[test]
    fn v1_25_fields_parses_codes() {
        let s = "v1|76561198000000000|srv|1000|0|0|0|0|1|1||0|1|0|152.000|1||1|1|3000|0|1|0|ALFA|";
        let p = parse_player_state_v1(s).unwrap();
        assert_eq!(p.radio_sr.as_ref().unwrap().code, "ALFA");
        assert!(p.radio_lr.is_none());
    }

    #[test]
    fn v1_28_fields_parses_intercom_vehicle_id() {
        let s = "v1|id|srv|0|0|0|0|0|1|1|B_MRAP_01_F|1|0|0|43.0|1||1|1|0|0|0|0|||1|2|1:2";
        let p = parse_player_state_v1(s).unwrap();
        assert!(p.intercom_enabled);
        assert_eq!(p.intercom_channel, 2);
        assert_eq!(p.intercom_vehicle_id, "1:2");
    }

    #[test]
    fn v1_alive_conscious_flags() {
        let alive_dead = "v1|id|srv|0|0|0|0|0|0|1||0|0|0|43.0|1||1";
        let p = parse_player_state_v1(alive_dead).unwrap();
        assert!(!p.alive);
        assert!(p.conscious);

        let alive_unconscious = "v1|id|srv|0|0|0|0|0|1|0||0|0|0|43.0|1||1";
        let p2 = parse_player_state_v1(alive_unconscious).unwrap();
        assert!(p2.alive);
        assert!(!p2.conscious);
    }

    #[test]
    fn v1_vehicle_field() {
        let s = "v1|id|srv|0|0|0|0|0|1|1|B_MRAP_01_F|0|0|0|43.0|1||1";
        let p = parse_player_state_v1(s).unwrap();
        assert_eq!(p.vehicle, "B_MRAP_01_F");
    }

    #[test]
    fn v1_ptt_flags() {
        let s = "v1|id|srv|0|0|0|0|0|1|1||1|1|0|43.0|1||1";
        let p = parse_player_state_v1(s).unwrap();
        assert!(p.ptt_local);
        assert!(p.ptt_radio_sr);
        assert!(!p.ptt_radio_lr);
    }

    #[test]
    fn v1_lr_radio_present() {
        let s = "v1|id|srv|0|0|0|0|0|1|1||0|0|1|43.0|1|30.0|2|1";
        let p = parse_player_state_v1(s).unwrap();
        let lr = p.radio_lr.as_ref().unwrap();
        assert_eq!(lr.freq, "30.0");
        assert_eq!(lr.channel, 2);
    }

    #[test]
    fn v1_zero_range_treated_as_none() {
        let s = "v1|id|srv|0|0|0|0|0|1|1||0|0|0|43.0|1||1|1|0|0";
        let p = parse_player_state_v1(s).unwrap();
        assert!(p.radio_sr.as_ref().unwrap().range_m.is_none());
    }

    #[test]
    fn v1_bad_field_count_errors() {
        let s = "v1|only|three";
        assert!(parse_player_state_v1(s).is_err());
    }

    #[test]
    fn v1_bad_prefix_errors() {
        let s = "v2|id|srv|0|0|0|0|0|1|1||0|0|0|43.0|1||1";
        assert!(parse_player_state_v1(s).is_err());
    }

    // ── normalize_v1_payload ────────────────────────────────────────────────

    #[test]
    fn normalize_bare_v1() {
        let result = normalize_v1_payload("v1|a|b").unwrap();
        assert_eq!(result, "v1|a|b");
    }

    #[test]
    fn normalize_json_string_wrapper() {
        let result = normalize_v1_payload(r#""v1|a|b""#).unwrap();
        assert_eq!(result, "v1|a|b");
    }

    #[test]
    fn normalize_json_array_wrapper() {
        let result = normalize_v1_payload(r#"["v1|a|b"]"#).unwrap();
        assert_eq!(result, "v1|a|b");
    }

    #[test]
    fn normalize_unknown_format_errors() {
        assert!(normalize_v1_payload("{}").is_err());
        assert!(normalize_v1_payload("not_v1|x").is_err());
    }

    // ── normalize_single_string_arg ─────────────────────────────────────────

    #[test]
    fn normalize_single_string_bare() {
        let result = normalize_single_string_arg("Jugador1").unwrap();
        assert_eq!(result, "Jugador1");
    }

    #[test]
    fn normalize_single_string_json_string() {
        let result = normalize_single_string_arg(r#""Jugador1""#).unwrap();
        assert_eq!(result, "Jugador1");
    }

    #[test]
    fn normalize_single_string_json_array() {
        let result = normalize_single_string_arg(r#"["Jugador1"]"#).unwrap();
        assert_eq!(result, "Jugador1");
    }

    // ── parse_player_state (bytes entry point) ─────────────────────────────

    #[test]
    fn parse_bytes_accepts_bare_v1() {
        let s = "v1|id|srv|0|0|0|0|0|1|1||0|0|0|43.0|1||1";
        assert!(parse_player_state(s.as_bytes()).is_ok());
    }

    #[test]
    fn parse_bytes_rejects_empty() {
        assert!(parse_player_state(b"").is_err());
    }

    // ── split_escaped_pipe ──────────────────────────────────────────────────

    #[test]
    fn split_basic() {
        let parts = split_escaped_pipe("a|b|c");
        assert_eq!(parts, vec!["a", "b", "c"]);
    }

    #[test]
    fn split_empty_field() {
        let parts = split_escaped_pipe("a||c");
        assert_eq!(parts, vec!["a", "", "c"]);
    }

    #[test]
    fn split_escaped_pipe_char() {
        let parts = split_escaped_pipe(r"a\|b|c");
        assert_eq!(parts, vec!["a|b", "c"]);
    }

    // ── unescape_pipe_field ─────────────────────────────────────────────────

    #[test]
    fn unescape_no_escapes() {
        assert_eq!(unescape_pipe_field("hello"), "hello");
    }

    #[test]
    fn unescape_escaped_pipe() {
        assert_eq!(unescape_pipe_field(r"a\|b"), "a|b");
    }

    #[test]
    fn unescape_escaped_backslash() {
        assert_eq!(unescape_pipe_field(r"a\\b"), r"a\b");
    }
}
