// SPDX-License-Identifier: GPL-3.0
// fn_setFrequencyLR.sqf - Cambia la frecuencia de radio LR del jugador local.
// params: [_freq (String), _channel (Number, opcional)]
//
// Uso:
//   ["30.000"] call RMTFAR_fnc_setFrequencyLR;
//   ["30.000", 2] call RMTFAR_fnc_setFrequencyLR;

params [
    ["_freq",    RMTFAR_radioFreqLR,    [""]],
    ["_channel", RMTFAR_radioChannelLR, [0]]
];

RMTFAR_radioFreqLR    = _freq;
RMTFAR_radioChannelLR = _channel;

[format ["Frecuencia LR: %1 (canal %2)", _freq, _channel], true] call CBA_fnc_notify;

diag_log format ["RMTFAR: LR frequency changed to %1 ch%2", _freq, _channel];
