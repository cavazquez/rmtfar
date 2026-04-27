// SPDX-License-Identifier: GPL-3.0
// fn_buildStatePayload.sqf - Serializa un estado (HashMap de getPlayerState) al formato v1|...
// params: [_state (HashMap)]
// returns: String — payload listo para enviar via callExtension

params ["_state"];

private _pos   = _state get "pos";
private _alive = if (_state get "alive") then { 1 } else { 0 };
private _cons  = if (_state get "conscious") then { 1 } else { 0 };
private _pttL  = if (_state get "ptt_local") then { 1 } else { 0 };
private _pttR  = if (_state get "ptt_radio_sr") then { 1 } else { 0 };
private _pttLR = if (_state get "ptt_radio_lr") then { 1 } else { 0 };

private _uidEsc    = [_state get "uid"]                call RMTFAR_fnc_escapeJsonString;
private _srvEsc    = [serverName]                      call RMTFAR_fnc_escapeJsonString;
private _vehEsc    = [_state get "vehicle"]            call RMTFAR_fnc_escapeJsonString;
private _freqEsc   = [_state get "radio_freq"]         call RMTFAR_fnc_escapeJsonString;
private _freqLREsc = [_state get "radio_freq_lr"]      call RMTFAR_fnc_escapeJsonString;
private _codeEsc   = [_state get "radio_code"]         call RMTFAR_fnc_escapeJsonString;
private _codeLREsc = [_state get "radio_code_lr"]      call RMTFAR_fnc_escapeJsonString;
private _icVehEsc  = [_state get "intercom_vehicle_id"] call RMTFAR_fnc_escapeJsonString;

private _st   = _state get "radio_stereo";
private _stLR = _state get "radio_stereo_lr";
private _icEn = if (_state get "intercom_enabled") then { 1 } else { 0 };
private _icCh = _state get "intercom_channel";
private _tick = floor diag_tickTime;

private _los  = 1;
if (!isNil {_state get "radio_los"}) then { _los = _state get "radio_los"; };

private _srRm = 0;
private _lrRm = 0;
if (!isNil {_state get "radio_sr_range_m"}) then { _srRm = _state get "radio_sr_range_m"; };
if (!isNil {_state get "radio_lr_range_m"}) then { _lrRm = _state get "radio_lr_range_m"; };

// v1|player_id|server_id|tick|x|y|z|dir|alive|conscious|vehicle|ptt_local|ptt_sr|ptt_lr|
//    sr_freq|sr_ch|lr_freq|lr_ch|radio_los|sr_range_m|lr_range_m|sr_stereo|lr_stereo|
//    sr_code|lr_code|intercom_enabled|intercom_channel|intercom_vehicle_id
format [
    "v1|%1|%2|%3|%4|%5|%6|%7|%8|%9|%10|%11|%12|%13|%14|%15|%16|%17|%18|%19|%20|%21|%22|%23|%24|%25|%26|%27",
    _uidEsc,
    _srvEsc,
    _tick,
    _pos select 0,
    _pos select 1,
    _pos select 2,
    _state get "dir",
    _alive,
    _cons,
    _vehEsc,
    _pttL,
    _pttR,
    _pttLR,
    _freqEsc,
    _state get "radio_channel",
    _freqLREsc,
    _state get "radio_channel_lr",
    _los,
    _srRm,
    _lrRm,
    _st,
    _stLR,
    _codeEsc,
    _codeLREsc,
    _icEn,
    _icCh,
    _icVehEsc
]
