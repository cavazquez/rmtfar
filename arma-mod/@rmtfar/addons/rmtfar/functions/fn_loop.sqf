// SPDX-License-Identifier: GPL-3.0
// fn_loop.sqf - Loop principal del mod a 20Hz (sleep 0.05)
// Recolecta estado del jugador y lo envía al bridge vía extension.

while {RMTFAR_enabled} do {
    if (!isNull player && {alive player}) then {
        private _state = call RMTFAR_fnc_getPlayerState;
        [_state] call RMTFAR_fnc_sendState;
    };
    sleep 0.05;
};
