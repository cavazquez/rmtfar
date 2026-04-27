// SPDX-License-Identifier: GPL-3.0
// fn_radioTransmitActive.sqf - Activa/desactiva TX en la radio activa (SR/LR).
// params: [_transmitting (Boolean)]
//
// Uso:
//   [true] call RMTFAR_fnc_radioTransmitActive;
//   [false] call RMTFAR_fnc_radioTransmitActive;

params [["_transmitting", false, [false]]];

private _active = toUpper (missionNamespace getVariable ["RMTFAR_activeRadio", "SR"]);

if (_active isEqualTo "LR") then {
    RMTFAR_pttRadioLR = _transmitting;
    RMTFAR_pttRadioSR = false;
} else {
    RMTFAR_pttRadioSR = _transmitting;
    RMTFAR_pttRadioLR = false;
};
