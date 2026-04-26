// SPDX-License-Identifier: GPL-3.0
// fn_loop.sqf - Loop principal del mod a 20Hz.
// Broadcast local state via publicVariable, gather all players, send to extension.

while {RMTFAR_enabled} do {
    if (!isNull player && {alive player}) then {
        // Broadcast local player's radio state so other clients can read it
        player setVariable ["rmtfar_pttLocal",   RMTFAR_pttLocal,      true];
        player setVariable ["rmtfar_pttRadioSR", RMTFAR_pttRadioSR,    true];
        player setVariable ["rmtfar_pttRadioLR", RMTFAR_pttRadioLR,    true];
        player setVariable ["rmtfar_freq",       RMTFAR_radioFreq,     true];
        player setVariable ["rmtfar_ch",         RMTFAR_radioChannel,  true];
        player setVariable ["rmtfar_freqLR",     RMTFAR_radioFreqLR,   true];
        player setVariable ["rmtfar_chLR",       RMTFAR_radioChannelLR, true];

        // Send state for every player to the extension
        {
            private _state = [_x] call RMTFAR_fnc_getPlayerState;
            [_state] call RMTFAR_fnc_sendState;
        } forEach allPlayers;
    };
    sleep 0.05;
};
