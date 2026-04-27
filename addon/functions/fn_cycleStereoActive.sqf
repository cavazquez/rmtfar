// SPDX-License-Identifier: GPL-3.0
// fn_cycleStereoActive.sqf - Rota estereo (Both->Left->Right->Both) de radio activa.
// params: [_direction (String): "next" | "prev"]

params [["_direction", "next", [""]]];

private _active = toUpper (missionNamespace getVariable ["RMTFAR_activeRadio", "SR"]);
private _cur = if (_active isEqualTo "LR") then { RMTFAR_radioStereoLR } else { RMTFAR_radioStereo };
private _delta = if (toLower _direction isEqualTo "prev") then { -1 } else { 1 };

private _next = (_cur + _delta) mod 3;
if (_next < 0) then { _next = _next + 3; };

[_next] call RMTFAR_fnc_setStereoActive;
