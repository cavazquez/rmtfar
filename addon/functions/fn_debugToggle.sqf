// SPDX-License-Identifier: GPL-3.0
// fn_debugToggle.sqf — Activa/desactiva modo test (ghosts + logs RPT selectos).

if (!hasInterface) exitWith {};

private _on = !(missionNamespace getVariable ["RMTFAR_debugMode", false]);
if (!_on) then {
    [] call RMTFAR_fnc_debugGhostClear;
};
missionNamespace setVariable ["RMTFAR_debugMode", _on];

private _msg = if (_on) then {
    "RMTFAR DEBUG: ON (Ctrl+Shift+F8 ghost | F9 PTT | F10 freq)"
} else {
    "RMTFAR DEBUG: OFF"
};
diag_log _msg;
systemChat _msg;
