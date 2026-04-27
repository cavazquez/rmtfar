// SPDX-License-Identifier: GPL-3.0
// fn_setChannelActive.sqf - Cambia canal (1..9) de la radio activa SR/LR.
// params: [_channel (Number)]
//
// Uso:
//   [3] call RMTFAR_fnc_setChannelActive;

params [["_channel", 1, [0]]];

private _ch = round _channel;
if (_ch < 1) then { _ch = 1; };
if (_ch > 9) then { _ch = 9; };

private _active = toUpper (missionNamespace getVariable ["RMTFAR_activeRadio", "SR"]);
if (_active isEqualTo "LR") then {
    RMTFAR_radioChannelLR = _ch;
    [format ["Canal LR: %1", _ch], true] call CBA_fnc_notify;
    diag_log format ["RMTFAR: active LR channel changed to %1", _ch];
} else {
    RMTFAR_radioChannel = _ch;
    [format ["Canal SR: %1", _ch], true] call CBA_fnc_notify;
    diag_log format ["RMTFAR: active SR channel changed to %1", _ch];
};
