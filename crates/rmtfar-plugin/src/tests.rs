// SPDX-License-Identifier: GPL-3.0

//! Unit tests for `Plugin::process_audio`.
//!
//! We create a Plugin directly (no Mumble socket) and manually drive its state,
//! then verify that samples are zeroed (muted) or non-zero (pass / DSP applied).

use rmtfar_protocol::{PlayerSummary, RadioStateMessage};

use crate::Plugin;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const LOCAL_ID: &str = "jugador2";
const SENDER_ID: &str = "Jugador1";
const SENDER_SESSION: u32 = 42;

/// Create a minimal `RadioStateMessage`.
fn make_msg(local: PlayerSummary, sender: PlayerSummary) -> RadioStateMessage {
    RadioStateMessage::new("srv".into(), 1, LOCAL_ID.into(), vec![local, sender])
}

/// Build a minimal `PlayerSummary` for the local listener.
fn local_player(tuned_sr_freq: &str, tuned_sr_channel: u8) -> PlayerSummary {
    PlayerSummary {
        player_id: LOCAL_ID.into(),
        pos: [0.0, 0.0, 0.0],
        dir: 0.0,
        alive: true,
        conscious: true,
        in_vehicle: false,
        transmitting_local: false,
        transmitting_radio: false,
        radio_type: String::new(),
        radio_freq: String::new(),
        radio_code: String::new(),
        radio_channel: 1,
        radio_range_m: 0.0,
        tuned_sr_freq: tuned_sr_freq.into(),
        tuned_sr_channel,
        tuned_sr_stereo: 0,
        tuned_sr_code: String::new(),
        tuned_lr_freq: String::new(),
        tuned_lr_channel: 1,
        tuned_lr_stereo: 0,
        tuned_lr_code: String::new(),
        radio_los_quality: 1.0,
        intercom_enabled: true,
        intercom_channel: 1,
        intercom_vehicle_id: String::new(),
        ptt_local_raw: false,
    }
}

/// Build a minimal `PlayerSummary` for a remote sender on SR radio.
#[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
fn sender_sr(
    freq: &str,
    channel: u8,
    range_m: f32,
    pos: [f32; 3],
    transmitting: bool,
    alive: bool,
    conscious: bool,
    in_vehicle: bool,
) -> PlayerSummary {
    PlayerSummary {
        player_id: SENDER_ID.into(),
        pos,
        dir: 0.0,
        alive,
        conscious,
        in_vehicle,
        transmitting_local: false,
        transmitting_radio: transmitting,
        radio_type: "sr".into(),
        radio_freq: freq.into(),
        radio_code: String::new(),
        radio_channel: channel,
        radio_range_m: range_m,
        tuned_sr_freq: freq.into(),
        tuned_sr_channel: channel,
        tuned_sr_stereo: 0,
        tuned_sr_code: String::new(),
        tuned_lr_freq: String::new(),
        tuned_lr_channel: 1,
        tuned_lr_stereo: 0,
        tuned_lr_code: String::new(),
        radio_los_quality: 1.0,
        intercom_enabled: true,
        intercom_channel: 1,
        intercom_vehicle_id: String::new(),
        ptt_local_raw: false,
    }
}

/// Build a minimal `PlayerSummary` for proximity (local) voice.
fn sender_local(pos: [f32; 3], transmitting: bool) -> PlayerSummary {
    PlayerSummary {
        player_id: SENDER_ID.into(),
        pos,
        dir: 0.0,
        alive: true,
        conscious: true,
        in_vehicle: false,
        transmitting_local: transmitting,
        transmitting_radio: false,
        radio_type: String::new(),
        radio_freq: String::new(),
        radio_code: String::new(),
        radio_channel: 1,
        radio_range_m: 0.0,
        tuned_sr_freq: String::new(),
        tuned_sr_channel: 1,
        tuned_sr_stereo: 0,
        tuned_sr_code: String::new(),
        tuned_lr_freq: String::new(),
        tuned_lr_channel: 1,
        tuned_lr_stereo: 0,
        tuned_lr_code: String::new(),
        radio_los_quality: 1.0,
        intercom_enabled: true,
        intercom_channel: 1,
        intercom_vehicle_id: String::new(),
        ptt_local_raw: transmitting,
    }
}

/// Create a Plugin with pre-loaded state (no UDP socket).
fn plugin_with_state(msg: RadioStateMessage) -> Plugin {
    let mut plugin = Plugin::new();
    plugin.state.update(msg);
    plugin
        .state
        .register_session(SENDER_SESSION, SENDER_ID.into());
    plugin
}

fn nonzero_samples() -> Vec<f32> {
    vec![0.5f32; 480]
}

// ---------------------------------------------------------------------------
// Radio frequency tests
// ---------------------------------------------------------------------------

#[test]
fn radio_freq_match_applies_dsp() {
    let msg = make_msg(
        local_player("43.0", 1),
        sender_sr("43.0", 1, 500.0, [200.0, 0.0, 0.0], true, true, true, false),
    );
    let mut plugin = plugin_with_state(msg);
    let mut samples = nonzero_samples();
    let pass = plugin.process_audio(SENDER_SESSION, &mut samples, 48000, 1);
    assert!(pass, "should return true (intercepted)");
    // DSP modifies samples, so they may change — but should not be zeroed
    // (we only check the return value for mute decision here since DSP is opaque)
}

#[test]
fn radio_freq_mismatch_mutes() {
    let msg = make_msg(
        local_player("50.0", 1),
        sender_sr("43.0", 1, 500.0, [200.0, 0.0, 0.0], true, true, true, false),
    );
    let mut plugin = plugin_with_state(msg);
    let mut samples = nonzero_samples();
    let pass = plugin.process_audio(SENDER_SESSION, &mut samples, 48000, 1);
    assert!(!pass, "freq mismatch should mute");
}

// ---------------------------------------------------------------------------
// Channel tests
// ---------------------------------------------------------------------------

#[test]
fn radio_channel_mismatch_mutes() {
    let msg = make_msg(
        local_player("43.0", 1),
        sender_sr("43.0", 2, 500.0, [200.0, 0.0, 0.0], true, true, true, false),
    );
    let mut plugin = plugin_with_state(msg);
    let mut samples = nonzero_samples();
    let pass = plugin.process_audio(SENDER_SESSION, &mut samples, 48000, 1);
    assert!(!pass, "channel mismatch should mute");
}

#[test]
fn radio_channel_match_passes() {
    let msg = make_msg(
        local_player("43.0", 3),
        sender_sr("43.0", 3, 500.0, [200.0, 0.0, 0.0], true, true, true, false),
    );
    let mut plugin = plugin_with_state(msg);
    let mut samples = nonzero_samples();
    let pass = plugin.process_audio(SENDER_SESSION, &mut samples, 48000, 1);
    assert!(pass, "matching channel should pass");
}

// ---------------------------------------------------------------------------
// Range tests
// ---------------------------------------------------------------------------

#[test]
fn radio_out_of_range_mutes() {
    let msg = make_msg(
        local_player("43.0", 1),
        sender_sr("43.0", 1, 500.0, [800.0, 0.0, 0.0], true, true, true, false),
    );
    let mut plugin = plugin_with_state(msg);
    let mut samples = nonzero_samples();
    let pass = plugin.process_audio(SENDER_SESSION, &mut samples, 48000, 1);
    assert!(!pass, "out of range should mute");
}

#[test]
fn radio_within_range_passes() {
    let msg = make_msg(
        local_player("43.0", 1),
        sender_sr("43.0", 1, 500.0, [200.0, 0.0, 0.0], true, true, true, false),
    );
    let mut plugin = plugin_with_state(msg);
    let mut samples = nonzero_samples();
    let pass = plugin.process_audio(SENDER_SESSION, &mut samples, 48000, 1);
    assert!(pass, "within range should pass");
}

// ---------------------------------------------------------------------------
// Dead / unconscious
// ---------------------------------------------------------------------------

#[test]
fn dead_sender_mutes() {
    let msg = make_msg(
        local_player("43.0", 1),
        sender_sr(
            "43.0",
            1,
            500.0,
            [200.0, 0.0, 0.0],
            false,
            false,
            true,
            false,
        ),
    );
    let mut plugin = plugin_with_state(msg);
    let mut samples = nonzero_samples();
    let pass = plugin.process_audio(SENDER_SESSION, &mut samples, 48000, 1);
    assert!(!pass, "dead sender should mute");
}

#[test]
fn unconscious_sender_mutes() {
    let msg = make_msg(
        local_player("43.0", 1),
        sender_sr(
            "43.0",
            1,
            500.0,
            [200.0, 0.0, 0.0],
            false,
            true,
            false,
            false,
        ),
    );
    let mut plugin = plugin_with_state(msg);
    let mut samples = nonzero_samples();
    let pass = plugin.process_audio(SENDER_SESSION, &mut samples, 48000, 1);
    assert!(!pass, "unconscious sender should mute");
}

// ---------------------------------------------------------------------------
// Vehicle
// ---------------------------------------------------------------------------

#[test]
fn sender_in_vehicle_no_radio_ptt_mutes() {
    // In vehicle but NOT pressing radio PTT — transmitting_radio = false
    let msg = make_msg(
        local_player("43.0", 1),
        sender_sr("43.0", 1, 500.0, [200.0, 0.0, 0.0], false, true, true, true),
    );
    let mut plugin = plugin_with_state(msg);
    let mut samples = nonzero_samples();
    let pass = plugin.process_audio(SENDER_SESSION, &mut samples, 48000, 1);
    assert!(!pass, "in vehicle without radio PTT should mute");
}

// ---------------------------------------------------------------------------
// Proximity (local voice)
// ---------------------------------------------------------------------------

#[test]
fn local_voice_within_range_passes() {
    let msg = make_msg(local_player("", 1), sender_local([20.0, 0.0, 0.0], true));
    let mut plugin = plugin_with_state(msg);
    let mut samples = nonzero_samples();
    let pass = plugin.process_audio(SENDER_SESSION, &mut samples, 48000, 1);
    assert!(pass, "local voice within 50m should pass");
}

#[test]
fn local_voice_out_of_range_mutes() {
    let msg = make_msg(local_player("", 1), sender_local([60.0, 0.0, 0.0], true));
    let mut plugin = plugin_with_state(msg);
    let mut samples = nonzero_samples();
    let pass = plugin.process_audio(SENDER_SESSION, &mut samples, 48000, 1);
    assert!(!pass, "local voice beyond 50m should mute");
}

#[test]
fn local_voice_attenuation_applied() {
    let msg = make_msg(local_player("", 1), sender_local([25.0, 0.0, 0.0], true));
    let mut plugin = plugin_with_state(msg);
    let mut samples = vec![1.0f32; 480];
    plugin.process_audio(SENDER_SESSION, &mut samples, 48000, 1);
    // At 25m / 50m range: volume ~0.5 → samples should be around 0.5, not 1.0
    #[allow(clippy::cast_precision_loss)]
    let avg: f32 = samples.iter().sum::<f32>() / samples.len() as f32;
    assert!(avg < 0.9, "volume should be attenuated at 25m (avg={avg})");
    assert!(avg > 0.0, "volume should not be zero at 25m (avg={avg})");
}

// ---------------------------------------------------------------------------
// Unknown user passthrough
// ---------------------------------------------------------------------------

#[test]
fn unknown_user_passes_through() {
    let msg = make_msg(
        local_player("43.0", 1),
        sender_sr("43.0", 1, 500.0, [0.0, 0.0, 0.0], true, true, true, false),
    );
    let mut plugin = plugin_with_state(msg);
    // Use a session ID that was NOT registered
    let unknown_session = 999u32;
    let mut samples = nonzero_samples();
    let pass = plugin.process_audio(unknown_session, &mut samples, 48000, 1);
    assert!(pass, "unknown user should pass through");
}

// ---------------------------------------------------------------------------
// No state yet
// ---------------------------------------------------------------------------

#[test]
fn no_state_passes_through() {
    let mut plugin = Plugin::new(); // no state loaded
    let mut samples = nonzero_samples();
    let pass = plugin.process_audio(SENDER_SESSION, &mut samples, 48000, 1);
    assert!(pass, "with no state, all audio should pass through");
}
