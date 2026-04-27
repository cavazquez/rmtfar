// SPDX-License-Identifier: GPL-3.0
// XEH_preInit.sqf - Ejecutado antes de que la misión cargue (máquina local)

RMTFAR_enabled = false;

// Modo test Windows: ghosts sintéticos → misma ruta que jugadores reales (ver docs/windows-ghost-test.md)
missionNamespace setVariable ["RMTFAR_debugMode", false];
missionNamespace setVariable ["RMTFAR_ghosts", []];

// RPT cada ~2 s con PTT local y retorno de callExtension. Desactivar:
//   missionNamespace setVariable ["RMTFAR_logSends", false];
missionNamespace setVariable ["RMTFAR_logSends", true];
missionNamespace setVariable ["RMTFAR_showRadioHud", true];
RMTFAR_radioFreq = "152.000";
RMTFAR_radioChannel = 1;
RMTFAR_radioFreqLR = "";
RMTFAR_radioChannelLR = 1;
RMTFAR_pttLocal = false;
RMTFAR_pttRadioSR = false;
RMTFAR_pttRadioLR = false;
// Squelch local al activar/desactivar PTT SR (sin depender de Mumble).
missionNamespace setVariable ["RMTFAR_radioSquelchEnabled", true];
// Alcances efectivos (m); los rellena resolveRadioModel desde CfgRMTFAR.
RMTFAR_radioSrRangeM = 5000;
RMTFAR_radioLrRangeM = 20000;

// Referencia: defaults que fn_init solo aplica si CBA no tiene teclas reales para esa acción.
RMTFAR_keyPttLocal   = 0;     // sin tecla
RMTFAR_keyPttRadioSR = 0x3A;  // Caps Lock
