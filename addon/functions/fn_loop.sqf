// SPDX-License-Identifier: GPL-3.0
// fn_loop.sqf - Loop principal del mod a 20Hz.
// Broadcast local state via publicVariable, gather all players, send to extension.

private _lastModelCheck = -99;
while {RMTFAR_enabled} do {
    if (!isNull player) then {
        // resolveRadioModel builds a fingerprint of the player's inventory;
        // throttled to 2 s to avoid str() calls on every 20 Hz tick.
        if (diag_tickTime - _lastModelCheck >= 2) then {
            [] call RMTFAR_fnc_resolveRadioModel;
            _lastModelCheck = diag_tickTime;
        };
        // Broadcast local player's radio state so other clients can read it
        player setVariable ["rmtfar_pttLocal",   RMTFAR_pttLocal,      true];
        player setVariable ["rmtfar_pttRadioSR", RMTFAR_pttRadioSR,    true];
        player setVariable ["rmtfar_pttRadioLR", RMTFAR_pttRadioLR,    true];
        player setVariable ["rmtfar_freq",       RMTFAR_radioFreq,     true];
        player setVariable ["rmtfar_ch",         RMTFAR_radioChannel,  true];
        player setVariable ["rmtfar_st",         RMTFAR_radioStereo,   true];
        player setVariable ["rmtfar_code",       RMTFAR_radioCode,     true];
        player setVariable ["rmtfar_freqLR",     RMTFAR_radioFreqLR,   true];
        player setVariable ["rmtfar_chLR",       RMTFAR_radioChannelLR, true];
        player setVariable ["rmtfar_stLR",       RMTFAR_radioStereoLR, true];
        player setVariable ["rmtfar_codeLR",     RMTFAR_radioCodeLR,   true];
        player setVariable ["rmtfar_icEnabled",  RMTFAR_intercomEnabled, true];
        player setVariable ["rmtfar_icChannel",  RMTFAR_intercomChannel, true];
        player setVariable ["rmtfar_icVehId",    if (vehicle player != player) then { netId (vehicle player) } else { "" }, true];
        player setVariable ["rmtfar_srRangeM",   RMTFAR_radioSrRangeM, true];
        player setVariable ["rmtfar_lrRangeM",   RMTFAR_radioLrRangeM, true];

        // Send state for every player to the extension.
        // Each client sends all players so the local bridge can compute
        // distances and audio decisions for this machine's perspective.
        {
            if (!isNull _x) then {
                private _state = [_x] call RMTFAR_fnc_getPlayerState;
                [_state] call RMTFAR_fnc_sendState;
            };
        } forEach allPlayers;

        // Modo DEBUG: jugadores sintéticos (mismo v1|... que jugadores reales)
        if (missionNamespace getVariable ["RMTFAR_debugMode", false]) then {
            {
                [_x] call RMTFAR_fnc_sendState;
            } forEach (missionNamespace getVariable ["RMTFAR_ghosts", []]);
        };
    };
    sleep 0.05;
};
