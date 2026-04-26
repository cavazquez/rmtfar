// SPDX-License-Identifier: GPL-3.0
// XEH_preInit.sqf - Ejecutado antes de que la misión cargue (máquina local)

RMTFAR_enabled = false;
RMTFAR_radioFreq = "152.000";
RMTFAR_radioChannel = 1;
RMTFAR_radioFreqLR = "";
RMTFAR_radioChannelLR = 1;
RMTFAR_pttLocal = false;
RMTFAR_pttRadioSR = false;
RMTFAR_pttRadioLR = false;

// Teclas por defecto (configurables via CBA Settings)
RMTFAR_keyPttLocal   = 0x3A;  // Caps Lock
RMTFAR_keyPttRadioSR = 0x14;  // T
