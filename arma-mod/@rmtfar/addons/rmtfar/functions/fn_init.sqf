// SPDX-License-Identifier: GPL-3.0
// fn_init.sqf - Inicialización del mod (llamada en preInit via CBA)
// Registra CBA settings para teclas y configuración de radio.

if (!hasInterface) exitWith {};

// --- CBA Settings ---

[
    "RMTFAR_keyPttLocal",
    "KEY",
    "PTT - Voz Directa",
    "RMTFAR",
    0x3A,  // Caps Lock
    true,
    {}
] call CBA_settings_fnc_init;

[
    "RMTFAR_keyPttRadioSR",
    "KEY",
    "PTT - Radio (Corto Alcance)",
    "RMTFAR",
    0x14,  // T
    true,
    {}
] call CBA_settings_fnc_init;

[
    "RMTFAR_radioFreq",
    "STRING",
    "Frecuencia de Radio SR",
    "RMTFAR",
    "152.000",
    true,
    {}
] call CBA_settings_fnc_init;

diag_log "RMTFAR: Settings registered";
