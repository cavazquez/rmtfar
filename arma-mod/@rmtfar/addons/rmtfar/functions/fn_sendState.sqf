// SPDX-License-Identifier: GPL-3.0
// fn_sendState.sqf - Serializa el estado a JSON y lo envía via callExtension.
// params: [_state (HashMap)]

params ["_state"];

private _pos   = _state get "pos";
private _alive = if (_state get "alive")     then { "true" } else { "false" };
private _cons  = if (_state get "conscious") then { "true" } else { "false" };
private _pttL  = if (_state get "ptt_local")    then { "true" } else { "false" };
private _pttR  = if (_state get "ptt_radio_sr") then { "true" } else { "false" };

private _json = format [
    "{""v"":1,""type"":""player_state"",""steam_id"":""%1"",""server_id"":""%2"",""tick"":%3,""pos"":[%4,%5,%6],""dir"":%7,""alive"":%8,""conscious"":%9,""ptt_local"":%10,""ptt_radio_sr"":%11,""radio_sr"":{""freq"":""%12"",""channel"":%13,""volume"":1.0,""enabled"":true}}",
    _state get "uid",
    serverName,
    diag_tickTime,
    _pos select 0,
    _pos select 1,
    _pos select 2,
    _state get "dir",
    _alive,
    _cons,
    _pttL,
    _pttR,
    _state get "radio_freq",
    _state get "radio_channel"
];

"rmtfar" callExtension ["send", [_json]];
