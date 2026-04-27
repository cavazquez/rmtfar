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

[] call RMTFAR_fnc_loadProfileSettings;
[] call RMTFAR_fnc_hudStart;

[] call RMTFAR_fnc_resolveRadioModel;
player addEventHandler ["InventoryClosed", { [] spawn { sleep 0.05; [] call RMTFAR_fnc_resolveRadioModel }; }];
player addEventHandler ["Respawn", { [] spawn { sleep 0.1; missionNamespace setVariable ["rmtfar_radio_fingerprint", ""]; [] call RMTFAR_fnc_resolveRadioModel }; }];

[] spawn RMTFAR_fnc_loop;

diag_log format ["RMTFAR: Initialized v%1", _version];
systemChat format ["RMTFAR activo (v%1)", _version];
if (missionNamespace getVariable ["RMTFAR_showStartupHints", true]) then {
    private _active = toUpper (missionNamespace getVariable ["RMTFAR_activeRadio", "SR"]);
    private _st = if (_active isEqualTo "LR") then { RMTFAR_radioStereoLR } else { RMTFAR_radioStereo };
    private _stLabel = ["B", "L", "R"] select ((_st max 0) min 2);
    systemChat format ["RMTFAR: radio activa inicial %1 (%2)", _active, _stLabel];

    private _hasPttActive = ["RMTFAR", "PTTRadioActive"] call RMTFAR_fnc_cbaKeybindHasUserKeys;
    if (!_hasPttActive) then {
        systemChat "RMTFAR: asigna una tecla a 'PTT - Radio activa (SR/LR)' (sugerida: Caps Lock)";
    };

    private _hasToggle = ["RMTFAR", "ToggleActiveRadio"] call RMTFAR_fnc_cbaKeybindHasUserKeys;
    if (!_hasToggle) then {
        systemChat "RMTFAR: opcional - asigna tecla a 'Alternar radio activa (SR/LR)'";
    };
};
