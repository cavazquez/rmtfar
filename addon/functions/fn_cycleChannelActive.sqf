// SPDX-License-Identifier: GPL-3.0
// fn_cycleChannelActive.sqf - Rota canal (1..9) de la radio activa SR/LR.
// params: [_direction (String): "next" | "prev"]
//
// Uso:
//   ["next"] call RMTFAR_fnc_cycleChannelActive;
//   ["prev"] call RMTFAR_fnc_cycleChannelActive;

params [["_direction", "next", [""]]];

private _active = toUpper (missionNamespace getVariable ["RMTFAR_activeRadio", "SR"]);
private _ch = if (_active isEqualTo "LR") then { RMTFAR_radioChannelLR } else { RMTFAR_radioChannel };
private _delta = if (toLower _direction isEqualTo "prev") then { -1 } else { 1 };

_ch = _ch + _delta;
if (_ch > 9) then { _ch = 1; };
if (_ch < 1) then { _ch = 9; };

[_ch] call RMTFAR_fnc_setChannelActive;
