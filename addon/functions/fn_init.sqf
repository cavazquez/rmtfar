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
}, {
    RMTFAR_pttRadioSR = false;
}, [0x14, [false, false, false]]] call CBA_fnc_addKeybind;

diag_log "RMTFAR: Keybinds registered";
