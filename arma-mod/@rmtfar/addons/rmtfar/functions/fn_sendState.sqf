// SPDX-License-Identifier: GPL-3.0
// fn_sendState.sqf - Serializa el estado a string delimitado y lo envía via callExtension.
// params: [_state (HashMap)]

params ["_state"];

private _pos   = _state get "pos";
private _alive = if (_state get "alive") then { 1 } else { 0 };
private _cons  = if (_state get "conscious") then { 1 } else { 0 };
private _pttL  = if (_state get "ptt_local") then { 1 } else { 0 };
private _pttR  = if (_state get "ptt_radio_sr") then { 1 } else { 0 };
private _pttLR = if (_state get "ptt_radio_lr") then { 1 } else { 0 };

private _uidEsc  = [_state get "uid"] call RMTFAR_fnc_escapeJsonString;
private _srvEsc  = [serverName] call RMTFAR_fnc_escapeJsonString;
private _vehEsc  = [_state get "vehicle"] call RMTFAR_fnc_escapeJsonString;
private _freqEsc = [_state get "radio_freq"] call RMTFAR_fnc_escapeJsonString;
private _freqLREsc = [_state get "radio_freq_lr"] call RMTFAR_fnc_escapeJsonString;
private _tick = floor diag_tickTime;

private _los = 1;
if (!isNil {_state get "radio_los"}) then {
    _los = _state get "radio_los";
};

private _srRm = 0;
private _lrRm = 0;
if (!isNil {_state get "radio_sr_range_m"}) then { _srRm = _state get "radio_sr_range_m"; };
if (!isNil {_state get "radio_lr_range_m"}) then { _lrRm = _state get "radio_lr_range_m"; };

// Formato estable (tipo TFAR, sin JSON):
// ...|radio_los|sr_range_m|lr_range_m  (0 = usar alcance por defecto del protocolo)
// v1|steam_id|server_id|tick|x|y|z|dir|alive|conscious|vehicle|ptt_local|ptt_sr|ptt_lr|sr_freq|sr_ch|lr_freq|lr_ch|radio_los|sr_range_m|lr_range_m
private _payload = format [
    "v1|%1|%2|%3|%4|%5|%6|%7|%8|%9|%10|%11|%12|%13|%14|%15|%16|%17|%18|%19|%20",
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
    _lrRm
];

private _ret = "rmtfar" callExtension ["send", [_payload]];

if (missionNamespace getVariable ["RMTFAR_logSends", false]) then {
    private _uid = _state get "uid";
    private _anyPtt = (_state get "ptt_local") || {_state get "ptt_radio_sr"} || {_state get "ptt_radio_lr"};
    if (_uid isEqualTo getPlayerUID player && _anyPtt) then {
        private _t = diag_tickTime;
        private _last = missionNamespace getVariable ["RMTFAR_lastSendLog", 0];
        if (_t - _last >= 2) then {
            missionNamespace setVariable ["RMTFAR_lastSendLog", _t];
            diag_log format ["RMTFAR: send local PTT callExtension=%1 bytes=%2", _ret, count _payload];
        };
    };
};
