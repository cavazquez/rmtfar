//! Arma 3 Extension DLL for RMTFAR.
//!
//! Exposes the `RVExtension` / `RVExtensionArgs` C ABI required by Arma 3.
//! All communication with the bridge is fire-and-forget UDP on loopback.

use std::ffi::{c_char, c_int, CStr};
use std::sync::OnceLock;

mod sender;
use sender::BridgeSender;

static SENDER: OnceLock<BridgeSender> = OnceLock::new();

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn get_sender() -> &'static BridgeSender {
    SENDER.get_or_init(BridgeSender::new)
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
        "send" if arg_count >= 1 => {
            let json = CStr::from_ptr(*args).to_string_lossy();
            match get_sender().send(json.as_bytes()) {
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
    let capacity = (size as usize).saturating_sub(1);
    let len = bytes.len().min(capacity);
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), output as *mut u8, len);
    *output.add(len) = 0;
}
