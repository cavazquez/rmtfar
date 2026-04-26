// SPDX-License-Identifier: GPL-3.0
// fn_setFrequency.sqf - Cambia la frecuencia de radio SR del jugador local.
// params: [_freq (String), _channel (Number, opcional)]
//
// Uso:
//   ["152.500"] call RMTFAR_fnc_setFrequency;
//   ["152.500", 2] call RMTFAR_fnc_setFrequency;

params [
    ["_freq",    RMTFAR_radioFreq,    [""]],
    ["_channel", RMTFAR_radioChannel, [0]]
];

RMTFAR_radioFreq    = _freq;
RMTFAR_radioChannel = _channel;

[format ["Frecuencia: %1 (canal %2)", _freq, _channel], true] call CBA_fnc_notify;

diag_log format ["RMTFAR: Frequency changed to %1 ch%2", _freq, _channel];
