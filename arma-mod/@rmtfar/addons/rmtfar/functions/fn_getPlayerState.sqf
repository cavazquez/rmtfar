// SPDX-License-Identifier: GPL-3.0
// fn_getPlayerState.sqf - Recolecta estado de una unidad.
// Para el jugador local usa las variables locales; para remotos lee publicVariable.
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
private _freqLR     = "";
private _channelLR  = 1;

if (_isLocal) then {
    _pttLocal   = (GetKeyState RMTFAR_keyPttLocal)   > 0;
    _pttRadioSR = (GetKeyState RMTFAR_keyPttRadioSR) > 0;
    _pttRadioLR = RMTFAR_pttRadioLR;
    _freq       = RMTFAR_radioFreq;
    _channel    = RMTFAR_radioChannel;
    _freqLR     = RMTFAR_radioFreqLR;
    _channelLR  = RMTFAR_radioChannelLR;
} else {
    _pttLocal   = _unit getVariable ["rmtfar_pttLocal",   false];
    _pttRadioSR = _unit getVariable ["rmtfar_pttRadioSR", false];
    _pttRadioLR = _unit getVariable ["rmtfar_pttRadioLR", false];
    _freq       = _unit getVariable ["rmtfar_freq",       "152.000"];
    _channel    = _unit getVariable ["rmtfar_ch",         1];
    _freqLR     = _unit getVariable ["rmtfar_freqLR",     ""];
    _channelLR  = _unit getVariable ["rmtfar_chLR",       1];
};

createHashMapFromArray [
    ["uid",            getPlayerUID _unit],
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
    ["radio_freq_lr",  _freqLR],
    ["radio_channel_lr", _channelLR]
]
