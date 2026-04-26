// SPDX-License-Identifier: GPL-3.0
// XEH_postInit.sqf - Ejecutado después de que la misión cargue

if (!hasInterface) exitWith {};

private _version = "rmtfar" callExtension "version";
if (_version isEqualTo "") exitWith {
    diag_log "RMTFAR: Extension not loaded or not found. Disabling.";
};

diag_log format ["RMTFAR: Extension v%1 found", _version];

RMTFAR_enabled = true;

[] spawn RMTFAR_fnc_loop;

diag_log format ["RMTFAR: Initialized v%1", _version];
