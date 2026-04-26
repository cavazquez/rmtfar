// SPDX-License-Identifier: GPL-3.0
// fn_getPlayerState.sqf - Recolecta estado del jugador local.
// Retorna un HashMap con todos los campos necesarios para el protocolo.

private _pos = getPosASL player;
private _dir = getDir player;
private _alive = alive player;

// Soporte para ACE3 (inconsciente) — si no está instalado, usa false
private _unconscious = player getVariable ["ACE_isUnconscious", false];
private _conscious = !_unconscious;

private _inVehicle = vehicle player != player;
private _vehicleType = if (_inVehicle) then { typeOf (vehicle player) } else { "" };

// Leer teclas PTT
private _pttLocal   = (GetKeyState RMTFAR_keyPttLocal)   > 0;
private _pttRadioSR = (GetKeyState RMTFAR_keyPttRadioSR) > 0;

// Guardar estado PTT para otros sistemas
RMTFAR_pttLocal   = _pttLocal;
RMTFAR_pttRadioSR = _pttRadioSR;

createHashMapFromArray [
    ["uid",           getPlayerUID player],
    ["pos",           _pos],
    ["dir",           _dir],
    ["alive",         _alive],
    ["conscious",     _conscious],
    ["vehicle",       _vehicleType],
    ["ptt_local",     _pttLocal],
    ["ptt_radio_sr",  _pttRadioSR],
    ["radio_freq",    RMTFAR_radioFreq],
    ["radio_channel", RMTFAR_radioChannel]
]
