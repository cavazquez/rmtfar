// SPDX-License-Identifier: GPL-3.0
// fn_loadProfileSettings.sqf - Carga ajustes locales de RMTFAR desde profileNamespace.

private _v = profileNamespace getVariable ["RMTFAR_profileSettings_v1", []];
if !(_v isEqualType [] && {count _v >= 6}) exitWith {};

private _active = toUpper (_v select 0);
if (_active != "SR" && _active != "LR") then { _active = "SR"; };
missionNamespace setVariable ["RMTFAR_activeRadio", _active];

RMTFAR_radioStereo = ((_v select 1) max 0) min 2;
RMTFAR_radioStereoLR = ((_v select 2) max 0) min 2;
private _icEnabled = _v select 3;
RMTFAR_intercomEnabled = if (_icEnabled isEqualType true) then { _icEnabled } else { false };
RMTFAR_intercomChannel = ((_v select 4) max 1) min 3;
private _showDbg = _v select 5;
missionNamespace setVariable ["RMTFAR_showIntercomDebug", if (_showDbg isEqualType true) then { _showDbg } else { false }];

if (count _v >= 8) then {
    private _code = _v select 6;
    private _codeLR = _v select 7;
    RMTFAR_radioCode = if (_code isEqualType "") then { _code } else { "" };
    RMTFAR_radioCodeLR = if (_codeLR isEqualType "") then { _codeLR } else { "" };
};
