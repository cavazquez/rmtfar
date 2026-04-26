// SPDX-License-Identifier: GPL-3.0
// XEH_preInit.sqf - Ejecutado antes de que la misión cargue (máquina local)

RMTFAR_enabled = false;
RMTFAR_radioFreq = "152.000";
RMTFAR_radioChannel = 1;
RMTFAR_pttLocal = false;
RMTFAR_pttRadioSR = false;

// Teclas por defecto (configurables via CBA Settings)
// Caps Lock = 0x3A, T = 0x14
RMTFAR_keyPttLocal   = 0x3A;
RMTFAR_keyPttRadioSR = 0x14;
