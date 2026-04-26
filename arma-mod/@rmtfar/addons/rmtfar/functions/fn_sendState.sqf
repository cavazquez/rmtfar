// SPDX-License-Identifier: GPL-3.0
// fn_sendState.sqf - Serializa el estado a JSON y lo envía via callExtension.
// params: [_state (HashMap)]

params ["_state"];

private _pos   = _state get "pos";
private _alive = if (_state get "alive")     then { "true" } else { "false" };
private _cons  = if (_state get "conscious") then { "true" } else { "false" };
private _pttL  = if (_state get "ptt_local")    then { "true" } else { "false" };
private _pttR  = if (_state get "ptt_radio_sr") then { "true" } else { "false" };
private _pttLR = if (_state get "ptt_radio_lr") then { "true" } else { "false" };

private _freqLR  = _state get "radio_freq_lr";
private _chLR    = _state get "radio_channel_lr";
private _vehicle = _state get "vehicle";

private _radioLR = "null";
if (_freqLR != "") then {
    _radioLR = format [
        "{""freq"":""%1"",""channel"":%2,""volume"":1.0,""enabled"":true}",
        _freqLR, _chLR
    ];
};

private _vehicleJson = if (_vehicle == "") then { """""" } else {
    format ["""%1""", _vehicle]
};

private _json = format [
    "{""v"":1,""type"":""player_state"",""steam_id"":""%1"",""server_id"":""%2"",""tick"":%3,""pos"":[%4,%5,%6],""dir"":%7,""alive"":%8,""conscious"":%9,""vehicle"":%10,""ptt_local"":%11,""ptt_radio_sr"":%12,""ptt_radio_lr"":%13,""radio_sr"":{""freq"":""%14"",""channel"":%15,""volume"":1.0,""enabled"":true},""radio_lr"":%16}",
    _state get "uid",
    serverName,
    diag_tickTime,
    _pos select 0,
    _pos select 1,
    _pos select 2,
    _state get "dir",
    _alive,
    _cons,
    _vehicleJson,
    _pttL,
    _pttR,
    _pttLR,
    _state get "radio_freq",
    _state get "radio_channel",
    _radioLR
];

private _ret = "rmtfar" callExtension ["send", [_json]];

if (missionNamespace getVariable ["RMTFAR_logSends", false]) then {
    private _uid = _state get "uid";
    private _anyPtt = (_state get "ptt_local") || {_state get "ptt_radio_sr"} || {_state get "ptt_radio_lr"};
    if (_uid isEqualTo getPlayerUID player && _anyPtt) then {
        private _t = diag_tickTime;
        private _last = missionNamespace getVariable ["RMTFAR_lastSendLog", 0];
        if (_t - _last >= 2) then {
            missionNamespace setVariable ["RMTFAR_lastSendLog", _t];
            diag_log format ["RMTFAR: send local PTT callExtension=%1 bytes=%2", _ret, count _json];
        };
    };
};
