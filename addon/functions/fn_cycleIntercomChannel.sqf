// SPDX-License-Identifier: GPL-3.0
// fn_cycleIntercomChannel.sqf - Rota canal intercom (1..3).
// params: [_direction (String): "next" | "prev"]

params [["_direction", "next", [""]]];

private _delta = if (toLower _direction isEqualTo "prev") then { -1 } else { 1 };
private _ch = RMTFAR_intercomChannel + _delta;
if (_ch > 3) then { _ch = 1; };
if (_ch < 1) then { _ch = 3; };
RMTFAR_intercomChannel = _ch;

[format ["Intercom canal: %1", _ch], true] call CBA_fnc_notify;
diag_log format ["RMTFAR: intercom channel=%1", _ch];
[] call RMTFAR_fnc_saveProfileSettings;
