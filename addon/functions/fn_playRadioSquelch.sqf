// SPDX-License-Identifier: GPL-3.0
// fn_playRadioSquelch.sqf - Reproduce un "squelch" local al presionar/soltar PTT SR.
// params: [["_phase", "on", [""]]];  _phase: "on" | "off"

params [["_phase", "on", [""]]];

if !(missionNamespace getVariable ["RMTFAR_radioSquelchEnabled", true]) exitWith {};

// Priorizamos sonidos de TFAR si existen; si no, usamos fallback vanilla.
private _onCandidates = [
    "TFAR_rotatorPush",
    "TFAR_default_radio_start",
    "TFAR_microDagrOn",
    "ClickSoft"
];
private _offCandidates = [
    "TFAR_rotatorRelease",
    "TFAR_default_radio_end",
    "TFAR_microDagrOff",
    "Click"
];

private _candidates = if (_phase isEqualTo "off") then { _offCandidates } else { _onCandidates };

{
    if (isClass (configFile >> "CfgSounds" >> _x)) exitWith {
        playSound _x;
    };
} forEach _candidates;
