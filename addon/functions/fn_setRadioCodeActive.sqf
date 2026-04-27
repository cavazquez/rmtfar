// SPDX-License-Identifier: GPL-3.0
// fn_setRadioCodeActive.sqf - Ajusta codigo logico (encryption) de radio activa.
// params: [_code (String)]

params [["_code", "", [""]]];

private _active = toUpper (missionNamespace getVariable ["RMTFAR_activeRadio", "SR"]);
if (_active isEqualTo "LR") then {
    RMTFAR_radioCodeLR = _code;
    [format ["Codigo LR: %1", _code], true] call CBA_fnc_notify;
    diag_log format ["RMTFAR: active LR code set to %1", _code];
} else {
    RMTFAR_radioCode = _code;
    [format ["Codigo SR: %1", _code], true] call CBA_fnc_notify;
    diag_log format ["RMTFAR: active SR code set to %1", _code];
};

[] call RMTFAR_fnc_saveProfileSettings;
