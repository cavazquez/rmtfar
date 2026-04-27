//! Raw C FFI exports required by the Mumble 1.4.0+ plugin API.

#![allow(non_snake_case, non_camel_case_types)]

use crate::plugin;
use std::ffi::{c_char, c_void};
use std::os::raw::c_int;
use std::sync::OnceLock;

pub type mumble_plugin_id_t = u32;
pub type mumble_userid_t = u32;
pub type mumble_connection_t = i32;
pub type mumble_error_t = c_int;

pub const MUMBLE_STATUS_OK: mumble_error_t = 0;
pub const MUMBLE_EC_GENERIC_ERROR: mumble_error_t = 1;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct mumble_version_t {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[repr(C)]
pub struct MumbleStringWrapper {
    pub data: *const c_char,
    pub size: usize,
    pub needs_release: bool,
}

// ---------------------------------------------------------------------------
// Minimal MumbleAPI struct — only the first 5 function pointers matter here.
// Layout must exactly match Mumble's MumbleAPI_v10 / MumbleAPI_v12 structs
// (all function pointers are pointer-sized; layout is identical across API
// versions because PARAM_v1_2 only changes signatures, not field count).
// ---------------------------------------------------------------------------

type MumbleFnFreeMemory =
    unsafe extern "C" fn(caller_id: mumble_plugin_id_t, ptr: *const c_void) -> mumble_error_t;

type MumbleFnGetUserName = unsafe extern "C" fn(
    caller_id: mumble_plugin_id_t,
    connection: mumble_connection_t,
    user_id: mumble_userid_t,
    user_name: *mut *const c_char,
) -> mumble_error_t;

/// Opaque placeholder for function pointers we don't call.
type OpaqueFnPtr = *const ();

/// Subset of `MumbleAPI_v10_x0` — only the fields we actually call.
/// Fields 0–4 must be in the exact order from MumblePlugin.h:
///   0: freeMemory
///   1: getActiveServerConnection
///   2: isConnectionSynchronized
///   3: getLocalUserID
///   4: getUserName
#[repr(C)]
struct MumbleAPI {
    free_memory: MumbleFnFreeMemory,            // index 0
    _get_active_server_connection: OpaqueFnPtr, // index 1
    _is_connection_synchronized: OpaqueFnPtr,   // index 2
    _get_local_user_id: OpaqueFnPtr,            // index 3
    get_user_name: MumbleFnGetUserName,         // index 4
}

// ---------------------------------------------------------------------------
// Globals: extracted function pointers (NOT the struct pointer — the struct
// is stack-allocated inside Mumble's Plugin::init() and becomes invalid after
// that function returns, so we copy what we need immediately).
// ---------------------------------------------------------------------------

static API_FREE_MEMORY: OnceLock<MumbleFnFreeMemory> = OnceLock::new();
static API_GET_USER_NAME: OnceLock<MumbleFnGetUserName> = OnceLock::new();
static PLUGIN_ID: OnceLock<mumble_plugin_id_t> = OnceLock::new();

fn plugin_id() -> mumble_plugin_id_t {
    *PLUGIN_ID.get().unwrap_or(&0)
}

// ---------------------------------------------------------------------------
// Required exports
// ---------------------------------------------------------------------------

/// # Safety
/// Signature: `mumble_error_t mumble_init(mumble_plugin_id_t id)`
/// Called by Mumble with just the assigned plugin ID — no extra args.
#[no_mangle]
pub unsafe extern "C" fn mumble_init(id: mumble_plugin_id_t) -> mumble_error_t {
    let _ = PLUGIN_ID.set(id);
    if plugin().start() {
        MUMBLE_STATUS_OK
    } else {
        MUMBLE_EC_GENERIC_ERROR
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn mumble_shutdown() {
    plugin().stop();
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn mumble_getName() -> MumbleStringWrapper {
    static NAME: &[u8] = b"RMTFAR\0";
    MumbleStringWrapper {
        data: NAME.as_ptr().cast::<c_char>(),
        size: 6,
        needs_release: false,
    }
}

/// # Safety
/// Must return the Mumble API version the plugin was built against.
/// Mumble 1.5.x only supports plugins requesting 1.0.x or 1.2.x.
/// Requesting 1.4.0 falls through to an unsupported branch and fails silently.
#[no_mangle]
pub unsafe extern "C" fn mumble_getAPIVersion() -> mumble_version_t {
    mumble_version_t {
        major: 1,
        minor: 0,
        patch: 0,
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn mumble_getVersion() -> mumble_version_t {
    mumble_version_t {
        major: 0,
        minor: 1,
        patch: 0,
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn mumble_getAuthor() -> MumbleStringWrapper {
    static A: &[u8] = b"RMTFAR Contributors\0";
    MumbleStringWrapper {
        data: A.as_ptr().cast::<c_char>(),
        size: A.len() - 1,
        needs_release: false,
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn mumble_getDescription() -> MumbleStringWrapper {
    static D: &[u8] = b"Radio communication mod for Arma 3 (RMTFAR)\0";
    MumbleStringWrapper {
        data: D.as_ptr().cast::<c_char>(),
        size: D.len() - 1,
        needs_release: false,
    }
}

/// Extracts and stores the function pointers we need from the Mumble API struct.
///
/// Mumble passes a **stack-local** API struct, so we must copy what we need
/// here rather than storing the pointer (the struct is gone after init returns).
///
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn mumble_registerAPIFunctions(api: *const c_void) {
    if api.is_null() {
        return;
    }
    let api_struct = &*(api.cast::<MumbleAPI>());
    let _ = API_FREE_MEMORY.set(api_struct.free_memory);
    let _ = API_GET_USER_NAME.set(api_struct.get_user_name);
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn mumble_releaseResource(_pointer: *const c_void) {}

// ---------------------------------------------------------------------------
// Feature declaration — REQUIRED for audio callbacks to fire
// ---------------------------------------------------------------------------

/// Tells Mumble which features this plugin uses.
/// `MUMBLE_FEATURE_AUDIO` (1 << 1 = 2) enables `mumble_onAudioSourceFetched`.
/// Without this export Mumble never routes audio through the plugin.
///
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn mumble_getFeatures() -> u32 {
    2 // MUMBLE_FEATURE_AUDIO
}

/// Called by Mumble if it wants to deactivate a feature.
/// We don't support deactivating audio processing mid-session.
/// Returns `MUMBLE_FEATURE_NONE`.
///
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn mumble_deactivateFeatures(_features: u32) -> u32 {
    0 // MUMBLE_FEATURE_NONE — nothing deactivated
}

// ---------------------------------------------------------------------------
// Audio callback
// ---------------------------------------------------------------------------

/// Called by Mumble for every decoded audio frame about to be played.
/// Returns `true` to allow playback, `false` to silence the frame.
///
/// # Safety
/// `output_pcm` points to `sample_count * channel_count` valid floats.
#[no_mangle]
pub unsafe extern "C" fn mumble_onAudioSourceFetched(
    output_pcm: *mut f32,
    sample_count: u32,
    channel_count: u16,
    sample_rate: u32,
    _is_speech: bool,
    user_id: mumble_userid_t,
) -> bool {
    if output_pcm.is_null() {
        return false;
    }
    let total = (sample_count as usize) * (channel_count as usize);
    let samples = std::slice::from_raw_parts_mut(output_pcm, total);

    // process_audio returns:
    //   true  = pass audio through (possibly with DSP applied in-place)
    //   false = mute this user
    //
    // Mumble API: returning true from this callback means "I modified the buffer,
    // use my version". Returning false means "I didn't touch it, use original audio".
    // So when we want to mute, we must zero the buffer AND return true.
    let pass = plugin().process_audio(user_id, samples, sample_rate);
    if pass {
        // DSP was applied (or pass-through for unknown users) — samples may be
        // modified; tell Mumble to use our buffer.
        true
    } else {
        // Mute: zero the buffer so Mumble plays silence.
        samples.fill(0.0);
        true
    }
}

// ---------------------------------------------------------------------------
// Identity mapping callbacks
// ---------------------------------------------------------------------------

/// Called when a new user appears on the server.
/// We query the Mumble API for their username and cache the mapping
/// `mumble_session_id → username` so the audio callback can look them up.
///
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn mumble_onUserAdded(
    connection: mumble_connection_t,
    user_id: mumble_userid_t,
) {
    tracing::info!(user_id, connection, "RMTFAR: mumble_onUserAdded called");

    let (Some(get_user_name), Some(free_memory)) = (API_GET_USER_NAME.get(), API_FREE_MEMORY.get())
    else {
        tracing::warn!(
            user_id,
            "RMTFAR: API fn ptrs not set — skipping name lookup"
        );
        return;
    };

    let mut name_ptr: *const c_char = std::ptr::null();
    let rc = get_user_name(
        plugin_id(),
        connection,
        user_id,
        std::ptr::addr_of_mut!(name_ptr),
    );
    tracing::info!(
        user_id,
        rc,
        null = name_ptr.is_null(),
        "RMTFAR: getUserName returned"
    );

    if rc == MUMBLE_STATUS_OK && !name_ptr.is_null() {
        if let Ok(name_str) = std::ffi::CStr::from_ptr(name_ptr).to_str() {
            let name = name_str.to_string();
            tracing::info!(user_id, %name, "RMTFAR: registering identity");
            let mut p = plugin();
            p.log_mumble_user_registered(user_id, &name);
            p.state.register_session(user_id, name);
        }
        free_memory(plugin_id(), name_ptr.cast::<c_void>());
    } else {
        tracing::warn!(user_id, rc, "RMTFAR: getUserName failed or null ptr");
    }
}

/// Called when a user leaves the server. Cleans up the cached mapping.
///
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn mumble_onUserRemoved(
    _connection: mumble_connection_t,
    user_id: mumble_userid_t,
) {
    plugin().state.unregister_session(user_id);
    tracing::info!(user_id, "RMTFAR: user removed");
}

// ---------------------------------------------------------------------------
// Optional callbacks
// ---------------------------------------------------------------------------

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn mumble_onUserIdentityChanged(
    _user_id: mumble_userid_t,
    _identity: *const c_char,
) {
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn mumble_onServerConnected(_conn: mumble_connection_t) {
    tracing::info!("RMTFAR: connected to server");
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn mumble_onServerDisconnected(_conn: mumble_connection_t) {
    tracing::info!("RMTFAR: disconnected from server");
    let mut p = plugin();
    p.state = crate::state::PluginState::default();
    p.clear_map_fail_throttle();
}

/// Fired by Mumble whenever any user starts or stops talking.
/// `talking_state`: 0 = passive, 1 = talking, 2 = shouting, 3 = whispering.
///
/// We use this as a **lazy identity registration** point: the first time we see
/// a user talk we query the Mumble API for their username and cache the mapping.
/// This is more reliable than `mumble_onUserAdded` (whose Qt signal connection
/// varies across Mumble builds).
///
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn mumble_onUserTalkingStateChanged(
    conn: mumble_connection_t,
    user_id: mumble_userid_t,
    talking_state: c_int,
) {
    // Lazy registration: look up name the first time we see this user talk.
    let mut p = plugin();
    if p.state.name_for_session(user_id).is_none() {
        if let (Some(get_user_name), Some(free_memory)) =
            (API_GET_USER_NAME.get(), API_FREE_MEMORY.get())
        {
            let mut name_ptr: *const c_char = std::ptr::null();
            let rc = get_user_name(plugin_id(), conn, user_id, std::ptr::addr_of_mut!(name_ptr));
            if rc == MUMBLE_STATUS_OK && !name_ptr.is_null() {
                if let Ok(name_str) = std::ffi::CStr::from_ptr(name_ptr).to_str() {
                    let name = name_str.to_string();
                    tracing::info!(user_id, %name, "RMTFAR: lazy identity registered");
                    p.log_mumble_user_registered(user_id, &name);
                    p.state.register_session(user_id, name);
                }
                free_memory(plugin_id(), name_ptr.cast::<c_void>());
            } else {
                tracing::warn!(
                    user_id,
                    rc,
                    "RMTFAR: getUserName failed in talking callback"
                );
            }
        }
    }

    tracing::info!(user_id, talking_state, "RMTFAR: talking state changed");
}
