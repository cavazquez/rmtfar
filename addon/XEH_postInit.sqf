// SPDX-License-Identifier: GPL-3.0
// XEH_postInit.sqf - Ejecutado después de que la misión cargue

if (!hasInterface) exitWith {};

private _version = "rmtfar" callExtension "version";
if (_version isEqualTo "") exitWith {
    diag_log "RMTFAR: Extension not loaded or not found. Disabling.";
    systemChat "RMTFAR: extension no cargada";
};

diag_log format ["RMTFAR: Extension v%1 found", _version];

// Register local player with the extension (no bridge needed)
"rmtfar" callExtension ["init", [name player]];

RMTFAR_enabled = true;

[] call RMTFAR_fnc_resolveRadioModel;
player addEventHandler ["InventoryClosed", { [] spawn { sleep 0.05; [] call RMTFAR_fnc_resolveRadioModel }; }];
player addEventHandler ["Respawn", { [] spawn { sleep 0.1; missionNamespace setVariable ["rmtfar_radio_fingerprint", ""]; [] call RMTFAR_fnc_resolveRadioModel }; }];

[] spawn RMTFAR_fnc_loop;

diag_log format ["RMTFAR: Initialized v%1", _version];
systemChat format ["RMTFAR activo (v%1)", _version];
