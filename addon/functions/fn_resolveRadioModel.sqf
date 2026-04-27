// SPDX-License-Identifier: GPL-3.0
// fn_resolveRadioModel.sqf - Alcance SR/LR según CfgRMTFAR >> RadioItems y multiplicador RadioFactions.
// Sin coincidencia de ítem usa la clase Default. Salida: RMTFAR_radioSrRangeM / RMTFAR_radioLrRangeM (metros).

if (!hasInterface) exitWith {};

private _bp = backpackContainer player;
private _backItems = if (isNull _bp) then {
    ""
} else {
    str (itemCargo _bp)
};
private _fp = format [
    "%1|%2|%3|%4|%5|%6|%7",
    str (assignedItems player),
    str (uniformItems player),
    str (vestItems player),
    str (weaponItems player),
    _backItems,
    str (items player),
    faction player
];
if (_fp isEqualTo (missionNamespace getVariable ["rmtfar_radio_fingerprint", ""])) exitWith {};
missionNamespace setVariable ["rmtfar_radio_fingerprint", _fp];

private _cfgItems = configFile >> "CfgRMTFAR" >> "RadioItems";
private _defSr = if (isClass (_cfgItems >> "Default")) then { getNumber (_cfgItems >> "Default" >> "rangeSR") } else { 5000 };
private _defLr = if (isClass (_cfgItems >> "Default")) then { getNumber (_cfgItems >> "Default" >> "rangeLR") } else { 20000 };
if (_defSr <= 0) then { _defSr = 5000; };
if (_defLr < 0) then { _defLr = 0; };

private _candidates = [];
{ _candidates pushBack _x } forEach (assignedItems player);
{ _candidates pushBack _x } forEach (weaponItems player);
{ _candidates pushBack _x } forEach (vestItems player);
{ _candidates pushBack _x } forEach (uniformItems player);
private _bc = backpackContainer player;
if (!isNull _bc) then {
    { _candidates pushBack _x } forEach (itemCargo _bc);
};
{ _candidates pushBack _x } forEach (items player);

private _sr = _defSr;
private _lr = _defLr;
private _hit = "";
{
    if (_hit != "") exitWith {};
    private _c = _cfgItems >> _x;
    if (isClass _c) then {
        _hit = _x;
        private _rs = getNumber (_c >> "rangeSR");
        private _rl = getNumber (_c >> "rangeLR");
        if (_rs > 0) then { _sr = _rs; } else { _sr = _defSr; };
        if (_rl >= 0) then { _lr = _rl; } else { _lr = _defLr; };
    };
} forEach _candidates;

private _facCfg = configFile >> "CfgRMTFAR" >> "RadioFactions" >> faction player;
private _mult = 1;
if (isClass _facCfg) then {
    private _m = getNumber (_facCfg >> "rangeMult");
    if (_m > 0) then { _mult = _m; };
};
RMTFAR_radioSrRangeM = (_sr * _mult) max 1;
RMTFAR_radioLrRangeM = (_lr * _mult) max 0;
