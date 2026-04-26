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
// Globals: Mumble API pointer and our own plugin ID
// ---------------------------------------------------------------------------

static MUMBLE_API: OnceLock<usize> = OnceLock::new(); // raw ptr stored as usize
static PLUGIN_ID: OnceLock<mumble_plugin_id_t> = OnceLock::new();

fn api() -> Option<&'static MumbleAPI> {
    let addr = MUMBLE_API.get()?;
    // Safety: Mumble guarantees the API struct outlives the plugin.
    Some(unsafe { &*(*addr as *const MumbleAPI) })
}

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

/// Stores the Mumble API function-pointer table for later use.
///
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn mumble_registerAPIFunctions(api: *const c_void) {
    if !api.is_null() {
        let _ = MUMBLE_API.set(api as usize);
    }
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
        return true;
    }
    let total = (sample_count as usize) * (channel_count as usize);
    let samples = std::slice::from_raw_parts_mut(output_pcm, total);
    plugin().process_audio(user_id, samples, sample_rate)
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
    let Some(api) = api() else { return };
    let mut name_ptr: *const c_char = std::ptr::null();
    let rc = (api.get_user_name)(plugin_id(), connection, user_id, &raw mut name_ptr);
    if rc == MUMBLE_STATUS_OK && !name_ptr.is_null() {
        if let Ok(name) = std::ffi::CStr::from_ptr(name_ptr).to_str() {
            let name = name.to_string();
            tracing::info!(user_id, %name, "RMTFAR: user added — registering identity");
            plugin().state.register_session(user_id, name);
        }
        // Free the string allocated by Mumble
        (api.free_memory)(plugin_id(), name_ptr.cast::<c_void>());
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
    plugin().state = crate::state::PluginState::default();
}

/// Fired by Mumble whenever any user starts or stops talking.
/// `talking_state` values: 0 = passive, 1 = talking, 2 = shouting, 3 = whispering.
/// This is the simplest callback to verify that Mumble is calling our plugin at all.
///
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn mumble_onUserTalkingStateChanged(
    _conn: mumble_connection_t,
    user_id: mumble_userid_t,
    talking_state: c_int,
) {
    tracing::info!(user_id, talking_state, "RMTFAR: talking state changed");
}
