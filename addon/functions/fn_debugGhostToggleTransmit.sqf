// SPDX-License-Identifier: GPL-3.0
// fn_debugGhostToggleTransmit.sqf — PTT SR del primer ghost (on/off).
// params: [["_ghostIdx", 0]]

params [["_ghostIdx", 0, [0]]];

if (!hasInterface) exitWith {};

private _ghosts = missionNamespace getVariable ["RMTFAR_ghosts", []];
if (count _ghosts <= _ghostIdx) exitWith {
    systemChat "RMTFAR: no hay ghost (F8 en modo DEBUG)";
};

private _g = _ghosts select _ghostIdx;
private _tx = (_g get "ptt_radio_sr") || {_g get "ptt_radio_lr"};
if (_tx) then {
    _g set ["ptt_radio_sr", false];
    _g set ["ptt_radio_lr", false];
    diag_log "RMTFAR DEBUG: ghost PTT OFF";
    systemChat "RMTFAR DEBUG: ghost PTT OFF";
} else {
    _g set ["ptt_radio_sr", true];
    _g set ["ptt_radio_lr", false];
    diag_log "RMTFAR DEBUG: ghost PTT SR ON";
    systemChat "RMTFAR DEBUG: ghost PTT SR ON";
};
