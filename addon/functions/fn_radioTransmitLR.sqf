// SPDX-License-Identifier: GPL-3.0
// fn_radioTransmitLR.sqf - Activa/desactiva transmisión por radio LR.
// params: [_transmitting (Boolean)]
//
// Normalmente el estado PTT se lee desde CBA keybinds.
// Esta función permite overrides programáticos (p.ej. desde Zeus o misiones).

params [["_transmitting", false, [false]]];

RMTFAR_pttRadioLR = _transmitting;
