// SPDX-License-Identifier: GPL-3.0
// fn_setStereoActive.sqf - Ajusta estéreo (both/left/right) de la radio activa.
// params: [_mode (0..2)] 0=both, 1=left, 2=right

params [["_mode", 0, [0]]];

private _m = round _mode;
if (_m < 0) then { _m = 0; };
if (_m > 2) then { _m = 2; };

private _active = toUpper (missionNamespace getVariable ["RMTFAR_activeRadio", "SR"]);
private _label = ["Both", "Left", "Right"] select _m;
if (_active isEqualTo "LR") then {
    RMTFAR_radioStereoLR = _m;
    [format ["Estereo LR: %1", _label], true] call CBA_fnc_notify;
    diag_log format ["RMTFAR: active LR stereo set to %1", _m];
} else {
    RMTFAR_radioStereo = _m;
    [format ["Estereo SR: %1", _label], true] call CBA_fnc_notify;
    diag_log format ["RMTFAR: active SR stereo set to %1", _m];
};
[] call RMTFAR_fnc_saveProfileSettings;
