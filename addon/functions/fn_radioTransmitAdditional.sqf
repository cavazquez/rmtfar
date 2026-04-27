// SPDX-License-Identifier: GPL-3.0
// fn_radioTransmitAdditional.sqf - Activa/desactiva TX en la radio no activa (SR/LR).
// params: [_transmitting (Boolean)]
//
// Uso:
//   [true] call RMTFAR_fnc_radioTransmitAdditional;
//   [false] call RMTFAR_fnc_radioTransmitAdditional;

params [["_transmitting", false, [false]]];

private _active = toUpper (missionNamespace getVariable ["RMTFAR_activeRadio", "SR"]);

if (_active isEqualTo "LR") then {
    RMTFAR_pttRadioSR = _transmitting;
    RMTFAR_pttRadioLR = false;
} else {
    RMTFAR_pttRadioLR = _transmitting;
    RMTFAR_pttRadioSR = false;
};
