//! Raw C FFI exports required by the Mumble 1.4.0+ plugin API.

#![allow(non_snake_case, non_camel_case_types)]

use crate::plugin;
use std::ffi::{c_char, c_void};
use std::os::raw::{c_int, c_uint};

pub type mumble_plugin_id_t = u32;
pub type mumble_userid_t = u32;
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
// Required exports
// ---------------------------------------------------------------------------

/// # Safety
/// Signature: `mumble_error_t mumble_init(mumble_plugin_id_t id)`
/// Called by Mumble with just the assigned plugin ID — no extra args.
#[no_mangle]
pub unsafe extern "C" fn mumble_init(_id: mumble_plugin_id_t) -> mumble_error_t {
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
#[no_mangle]
pub unsafe extern "C" fn mumble_getAPIVersion() -> mumble_version_t {
    mumble_version_t {
        major: 1,
        minor: 4,
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

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn mumble_registerAPIFunctions(_api: *const c_void) {}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn mumble_releaseResource(_pointer: *const c_void) {}

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
// Optional callbacks
// ---------------------------------------------------------------------------

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn mumble_onUserIdentityChanged(
    user_id: mumble_userid_t,
    identity: *const c_char,
) {
    if identity.is_null() {
        return;
    }
    if let Ok(s) = std::ffi::CStr::from_ptr(identity).to_str() {
        if !s.is_empty() {
            plugin()
                .state
                .register_identity(&user_id.to_string(), s.to_string());
        }
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn mumble_onServerConnected(_conn: c_uint) {}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn mumble_onServerDisconnected(_conn: c_uint) {
    plugin().state = crate::state::PluginState::default();
}
