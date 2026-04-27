// SPDX-License-Identifier: GPL-3.0
// fn_toggleIntercomDebug.sqf - Toggle visual debug de intercom en HUD.

private _next = !(missionNamespace getVariable ["RMTFAR_showIntercomDebug", false]);
missionNamespace setVariable ["RMTFAR_showIntercomDebug", _next];
[format ["Intercom debug HUD: %1", if (_next) then {"ON"} else {"OFF"}], true] call CBA_fnc_notify;
[] call RMTFAR_fnc_saveProfileSettings;
