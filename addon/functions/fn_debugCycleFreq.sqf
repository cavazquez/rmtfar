// SPDX-License-Identifier: GPL-3.0
// fn_debugCycleFreq.sqf — Rota frecuencia SR del jugador local (presets).

if (!hasInterface) exitWith {};

private _presets = ["152.000", "43.000", "50.000"];
private _i = missionNamespace getVariable ["RMTFAR_debugFreqIdx", -1];
_i = (_i + 1) mod (count _presets);
missionNamespace setVariable ["RMTFAR_debugFreqIdx", _i];

private _f = _presets select _i;
[_f] call RMTFAR_fnc_setFrequency;

diag_log format ["RMTFAR DEBUG: SR local -> %1", _f];
