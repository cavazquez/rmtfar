// SPDX-License-Identifier: GPL-3.0
// fn_sendState.sqf - Envía un único estado via callExtension ["send", ...].
// Usado por el modo debug (ghosts). El loop principal usa send_batch via fn_loop.sqf.
// params: [_state (HashMap)]

params ["_state"];

private _payload = [_state] call RMTFAR_fnc_buildStatePayload;
private _ret = "rmtfar" callExtension ["send", [_payload]];

private _uid = _state get "uid";
if (
    missionNamespace getVariable ["RMTFAR_debugMode", false]
    && { _uid find "RMTFAR_ghost" == 0 }
) then {
    private _key = format ["RMTFAR_dbghost_%1", _uid];
    private _last = missionNamespace getVariable [_key, -100];
    private _t = diag_tickTime;
    if (_t - _last >= 1) then {
        missionNamespace setVariable [_key, _t];
        diag_log format [
            "RMTFAR DEBUG: ghost send uid=%1 ret=%2 sr_ptt=%3 lr_ptt=%4 freq=%5 ch=%6",
            _uid,
            _ret,
            _state get "ptt_radio_sr",
            _state get "ptt_radio_lr",
            _state get "radio_freq",
            _state get "radio_channel"
        ];
    };
};
