// SPDX-License-Identifier: GPL-3.0
// fn_loadProfileSettings.sqf - Carga ajustes locales de RMTFAR desde profileNamespace.

private _v = profileNamespace getVariable ["RMTFAR_profileSettings_v1", []];
if !(_v isEqualType [] && {count _v >= 6}) exitWith {};

private _active = toUpper (_v select 0);
if (_active != "SR" && _active != "LR") then { _active = "SR"; };
missionNamespace setVariable ["RMTFAR_activeRadio", _active];

RMTFAR_radioStereo = ((_v select 1) max 0) min 2;
RMTFAR_radioStereoLR = ((_v select 2) max 0) min 2;
RMTFAR_intercomEnabled = _v select 3;
RMTFAR_intercomChannel = ((_v select 4) max 1) min 3;
missionNamespace setVariable ["RMTFAR_showIntercomDebug", _v select 5];
