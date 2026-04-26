// SPDX-License-Identifier: GPL-3.0
// fn_debugGhostSpawn.sqf — Crea un jugador sintético (solo UDP), sin unidad en el mundo.
// params: [["_offsetASL", [50, 0, 0]]]
//
// El ghost usa el mismo formato v1 que allPlayers; la extensión lo guarda por player_id.

params [["_offsetASL", [50, 0, 0], [[]]]];

if (!hasInterface) exitWith {};

if !(missionNamespace getVariable ["RMTFAR_debugMode", false]) exitWith {
    systemChat "RMTFAR: activá DEBUG primero (Ctrl+Shift+F7)";
};

private _ghosts = missionNamespace getVariable ["RMTFAR_ghosts", []];
private _n = (count _ghosts) + 1;
private _id = format ["RMTFAR_ghost_%1", _n];

private _f = RMTFAR_radioFreq;
private _ch = RMTFAR_radioChannel;

private _base = getPosASL player;
private _pos = _base vectorAdd _offsetASL;

private _st = createHashMapFromArray [
    ["uid", _id],
    ["pos", _pos],
    ["dir", 0],
    ["alive", true],
    ["conscious", true],
    ["vehicle", ""],
    ["ptt_local", false],
    ["ptt_radio_sr", false],
    ["ptt_radio_lr", false],
    ["radio_freq", _f],
    ["radio_channel", _ch],
    ["radio_freq_lr", RMTFAR_radioFreqLR],
    ["radio_channel_lr", RMTFAR_radioChannelLR],
    ["radio_los", 1],
    ["radio_sr_range_m", 500],
    ["radio_lr_range_m", 0]
];

_ghosts pushBack _st;
missionNamespace setVariable ["RMTFAR_ghosts", _ghosts];

diag_log format ["RMTFAR DEBUG: ghost %1 pos=%2 SR=%3 ch%4", _id, _pos, _f, _ch];
systemChat format ["Ghost %1 @ SR %2 ch%3", _id, _f, _ch];
