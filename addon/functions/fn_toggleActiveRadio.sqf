// SPDX-License-Identifier: GPL-3.0
// fn_toggleActiveRadio.sqf - Alterna la radio activa entre SR y LR.
// Uso:
//   [] call RMTFAR_fnc_toggleActiveRadio;

private _current = toUpper (missionNamespace getVariable ["RMTFAR_activeRadio", "SR"]);
private _next = if (_current isEqualTo "LR") then { "SR" } else { "LR" };

missionNamespace setVariable ["RMTFAR_activeRadio", _next];
[format ["Radio activa: %1", _next], true] call CBA_fnc_notify;
diag_log format ["RMTFAR: active radio set to %1", _next];
[] call RMTFAR_fnc_saveProfileSettings;
