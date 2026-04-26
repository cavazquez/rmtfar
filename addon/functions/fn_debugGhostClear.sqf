// SPDX-License-Identifier: GPL-3.0
// fn_debugGhostClear.sqf — Quita todos los ghosts del store Rust y de SQF.

private _ghosts = missionNamespace getVariable ["RMTFAR_ghosts", []];
{
    private _uid = _x get "uid";
    "rmtfar" callExtension ["forget", [_uid]];
} forEach _ghosts;

missionNamespace setVariable ["RMTFAR_ghosts", []];
diag_log "RMTFAR DEBUG: ghosts cleared";
