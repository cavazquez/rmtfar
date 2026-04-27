// SPDX-License-Identifier: GPL-3.0
// fn_init.sqf - Inicialización del mod (llamada en preInit via CBA)
// Registra keybinds CBA para PTT.

if (!hasInterface) exitWith {};

// --- CBA Keybinds ---

["RMTFAR", "PTTLocal", "PTT - Voz Directa", {
    RMTFAR_pttLocal = true;
}, {
    RMTFAR_pttLocal = false;
}, [0x3A, [false, false, false]]] call CBA_fnc_addKeybind;

["RMTFAR", "PTTRadioSR", "PTT - Radio (Corto Alcance)", {
    RMTFAR_pttRadioSR = true;
    ["on"] call RMTFAR_fnc_playRadioSquelch;
}, {
    RMTFAR_pttRadioSR = false;
    ["off"] call RMTFAR_fnc_playRadioSquelch;
}, [0x14, [false, false, false]]] call CBA_fnc_addKeybind;

// --- DEBUG / test en Windows (Ctrl + Shift + F7–F10) — ver docs/windows-ghost-test.md
["RMTFAR", "DebugToggle", "DEBUG: modo test on/off", {
    [] call RMTFAR_fnc_debugToggle;
}, {}, [0x41, [true, true, false]]] call CBA_fnc_addKeybind;

["RMTFAR", "DebugSpawnGhost", "DEBUG: spawn ghost SR", {
    [] call RMTFAR_fnc_debugGhostSpawn;
}, {}, [0x42, [true, true, false]]] call CBA_fnc_addKeybind;

["RMTFAR", "DebugGhostPTT", "DEBUG: ghost #0 PTT SR toggle", {
    [] call RMTFAR_fnc_debugGhostToggleTransmit;
}, {}, [0x43, [true, true, false]]] call CBA_fnc_addKeybind;

["RMTFAR", "DebugCycleFreq", "DEBUG: rotar SR local (152/43/50)", {
    [] call RMTFAR_fnc_debugCycleFreq;
}, {}, [0x44, [true, true, false]]] call CBA_fnc_addKeybind;

diag_log "RMTFAR: Keybinds registered";
