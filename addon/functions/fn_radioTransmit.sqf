// SPDX-License-Identifier: GPL-3.0
// fn_radioTransmit.sqf - Activa/desactiva transmisión por radio SR.
// params: [_transmitting (Boolean)]
//
// Normalmente el estado PTT se lee directamente desde GetKeyState en el loop.
// Esta función permite overrides programáticos (p.ej. desde Zeus o misiones).

params [["_transmitting", false, [false]]];

RMTFAR_pttRadioSR = _transmitting;
