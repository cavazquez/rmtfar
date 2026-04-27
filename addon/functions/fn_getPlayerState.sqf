// SPDX-License-Identifier: GPL-3.0
// fn_getPlayerState.sqf - Recolecta estado de una unidad.
// Para el jugador local usa las variables globales; para remotos lee publicVariable.
// params: [_unit]

params ["_unit"];

private _pos = getPosASL _unit;
private _dir = getDir _unit;
private _alive = alive _unit;

private _unconscious = _unit getVariable ["ACE_isUnconscious", false];
private _conscious = !_unconscious;

private _inVehicle = vehicle _unit != _unit;
private _vehicleType = if (_inVehicle) then { typeOf (vehicle _unit) } else { "" };

private _isLocal = _unit == player;

private _pttLocal   = false;
private _pttRadioSR = false;
private _pttRadioLR = false;
private _freq       = "152.000";
private _channel    = 1;
private _stereo     = 0;
private _code       = "";
private _freqLR     = "";
private _channelLR  = 1;
private _stereoLR   = 0;
private _codeLR     = "";
private _icEnabled  = true;
private _icChannel  = 1;
private _icVehId    = "";

if (_isLocal) then {
    _pttLocal   = RMTFAR_pttLocal;
    _pttRadioSR = RMTFAR_pttRadioSR;
    _pttRadioLR = RMTFAR_pttRadioLR;
    _freq       = RMTFAR_radioFreq;
    _channel    = RMTFAR_radioChannel;
    _stereo     = RMTFAR_radioStereo;
    _code       = RMTFAR_radioCode;
    _freqLR     = RMTFAR_radioFreqLR;
    _channelLR  = RMTFAR_radioChannelLR;
    _stereoLR   = RMTFAR_radioStereoLR;
    _codeLR     = RMTFAR_radioCodeLR;
    _icEnabled  = RMTFAR_intercomEnabled;
    _icChannel  = RMTFAR_intercomChannel;
    _icVehId    = if (_inVehicle) then { netId (vehicle _unit) } else { "" };
} else {
    _pttLocal   = _unit getVariable ["rmtfar_pttLocal",   false];
    _pttRadioSR = _unit getVariable ["rmtfar_pttRadioSR", false];
    _pttRadioLR = _unit getVariable ["rmtfar_pttRadioLR", false];
    _freq       = _unit getVariable ["rmtfar_freq",       "152.000"];
    _channel    = _unit getVariable ["rmtfar_ch",         1];
    _stereo     = _unit getVariable ["rmtfar_st",         0];
    _code       = _unit getVariable ["rmtfar_code",       ""];
    _freqLR     = _unit getVariable ["rmtfar_freqLR",     ""];
    _channelLR  = _unit getVariable ["rmtfar_chLR",       1];
    _stereoLR   = _unit getVariable ["rmtfar_stLR",       0];
    _codeLR     = _unit getVariable ["rmtfar_codeLR",     ""];
    _icEnabled  = _unit getVariable ["rmtfar_icEnabled",  true];
    _icChannel  = _unit getVariable ["rmtfar_icChannel",  1];
    _icVehId    = _unit getVariable ["rmtfar_icVehId",    ""];
};

private _srRm = if (_isLocal) then { RMTFAR_radioSrRangeM } else { _unit getVariable ["rmtfar_srRangeM", 0] };
private _lrRm = if (_isLocal) then { RMTFAR_radioLrRangeM } else { _unit getVariable ["rmtfar_lrRangeM", 0] };

// Oclusión radio (LOS local → esta unidad), 1 = línea clara.
// Caché ~250 ms; el recálculo se reparte en 8 fases según UID para no hacer N−1 raycasts en un solo frame.
private _los = 1;
if (!_isLocal && {_alive} && {!isNull player} && {alive player}) then {
    private _ck = format ["rmtfar_los_%1", name _unit];
    private _cached = missionNamespace getVariable [_ck, [1, -1]];
    private _now = diag_tickTime;
    private _lastT = _cached select 1;
    private _fresh = (_lastT >= 0 && {(_now - _lastT) < 0.25});
    if (_fresh) then {
        _los = _cached select 0;
    } else {
        private _staggerN = 8;
        private _uid = name _unit;
        private _h = 0;
        { _h = _h + _x } forEach (toArray _uid);
        private _slot = _h mod _staggerN;
        private _frame = floor (_now * 20);
        private _tooStale = (_lastT >= 0 && {(_now - _lastT) > 1.0});
        private _firstEver = (_lastT < 0);
        private _mayRay = _firstEver || {_tooStale} || {(_frame mod _staggerN) isEqualTo _slot};
        if (_mayRay) then {
            _los = 1;
            private _from = eyePos player;
            private _to = (getPosASL _unit) vectorAdd [0, 0, 1];
            private _total = _from distance _to;
            if (_total > 1) then {
                private _hits = lineIntersectsSurfaces [_from, _to, player, vehicle player, true, 4, "VIEW", "NONE"];
                if (count _hits > 0) then {
                    private _first = _hits select 0;
                    private _hitPos = _first select 0;
                    private _hitObj = _first select 2;
                    private _dHit = _from distance _hitPos;
                    private _ratio = _dHit / _total;
                    private _tVeh = vehicle _unit;
                    if (!isNull _hitObj && {_hitObj isEqualTo _unit || {!isNull _tVeh && {_hitObj isEqualTo _tVeh}}}) then {
                        _los = 1;
                    } else {
                        if (_ratio < 0.98) then {
                            private _sq = _ratio * _ratio;
                            _los = ((0.15 + 0.85 * _sq) min 1) max 0.05;
                        };
                    };
                };
            };
            missionNamespace setVariable [_ck, [_los, _now]];
        } else {
            if (_lastT >= 0) then {
                _los = _cached select 0;
            };
        };
    };
};

createHashMapFromArray [
    ["uid",            name _unit],
    ["pos",            _pos],
    ["dir",            _dir],
    ["alive",          _alive],
    ["conscious",      _conscious],
    ["vehicle",        _vehicleType],
    ["ptt_local",      _pttLocal],
    ["ptt_radio_sr",   _pttRadioSR],
    ["ptt_radio_lr",   _pttRadioLR],
    ["radio_freq",     _freq],
    ["radio_channel",  _channel],
    ["radio_stereo",   _stereo],
    ["radio_code",     _code],
    ["radio_freq_lr",  _freqLR],
    ["radio_channel_lr", _channelLR],
    ["radio_stereo_lr", _stereoLR],
    ["radio_code_lr", _codeLR],
    ["intercom_enabled", _icEnabled],
    ["intercom_channel", _icChannel],
    ["intercom_vehicle_id", _icVehId],
    ["radio_los", _los],
    ["radio_sr_range_m", _srRm],
    ["radio_lr_range_m", _lrRm]
]
