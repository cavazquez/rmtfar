// SPDX-License-Identifier: GPL-3.0
// XEH_preInit.sqf - Ejecutado antes de que la misión cargue (máquina local)

RMTFAR_enabled = false;

// Modo test Windows: ghosts sintéticos → misma ruta que jugadores reales (ver docs/windows-ghost-test.md)
missionNamespace setVariable ["RMTFAR_debugMode", false];
missionNamespace setVariable ["RMTFAR_ghosts", []];

// RPT cada ~2 s con PTT local y retorno de callExtension. Desactivar:
//   missionNamespace setVariable ["RMTFAR_logSends", false];
missionNamespace setVariable ["RMTFAR_logSends", false];
missionNamespace setVariable ["RMTFAR_showRadioHud", true];
missionNamespace setVariable ["RMTFAR_showStartupHints", true];
missionNamespace setVariable ["RMTFAR_showIntercomDebug", false];
RMTFAR_radioFreq = "152.000";
RMTFAR_radioChannel = 1;
RMTFAR_radioStereo = 0; // 0=both, 1=left, 2=right
RMTFAR_radioCode = "";
RMTFAR_radioFreqLR = "";
RMTFAR_radioChannelLR = 1;
RMTFAR_radioStereoLR = 0; // 0=both, 1=left, 2=right
RMTFAR_radioCodeLR = "";
RMTFAR_pttLocal = false;
RMTFAR_pttRadioSR = false;
RMTFAR_pttRadioLR = false;
missionNamespace setVariable ["RMTFAR_activeRadio", "SR"];
RMTFAR_intercomEnabled = true;
RMTFAR_intercomChannel = 1;
// Squelch local al activar/desactivar PTT SR (sin depender de Mumble).
missionNamespace setVariable ["RMTFAR_radioSquelchEnabled", true];
// Alcances efectivos (m); los rellena resolveRadioModel desde CfgRMTFAR.
RMTFAR_radioSrRangeM = 5000;
RMTFAR_radioLrRangeM = 20000;

// Referencia: defaults que fn_init solo aplica si CBA no tiene teclas reales para esa acción.
RMTFAR_keyPttLocal   = 0;     // sin tecla
RMTFAR_keyPttRadioSR = 0;     // sin tecla (asignable por CBA)
RMTFAR_keyPttRadioLR = 0;     // sin tecla (asignable por CBA)
RMTFAR_keyPttRadioActive = 0x3A; // Caps Lock (PTT sobre radio activa SR/LR)
RMTFAR_keyPttRadioAdditional = 0; // sin tecla (PTT radio opuesta a la activa)
RMTFAR_keyToggleActiveRadio = 0; // sin tecla (alternar SR/LR activa)
RMTFAR_keyStereoBoth = 200; // Shift+Arrow Up
RMTFAR_keyStereoLeft = 203; // Shift+Arrow Left
RMTFAR_keyStereoRight = 205; // Shift+Arrow Right
RMTFAR_keyIntercomToggle = 0; // sin tecla (ON/OFF)
RMTFAR_keyIntercomNext = 201; // Alt+PageUp (canal siguiente)
RMTFAR_keyIntercomPrev = 209; // Alt+PageDown (canal anterior)
