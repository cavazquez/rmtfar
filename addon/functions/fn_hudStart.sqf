// SPDX-License-Identifier: GPL-3.0
// fn_hudStart.sqf — HUD in-game (frecuencias, canales, PTT). Llama postInit vía spawn.
// Desactivar: missionNamespace setVariable ["RMTFAR_showRadioHud", false];

if (!hasInterface) exitWith {};

#define RMTFAR_HUD_LAYER 23
#define RMTFAR_HUD_IDC 84001

[] spawn {
    waitUntil { !isNull player && hasInterface };

    while { true } do {
        waitUntil {
            RMTFAR_enabled
            && {missionNamespace getVariable ["RMTFAR_showRadioHud", true]}
        };

        RMTFAR_HUD_LAYER cutRsc ["Default", "PLAIN", -1, false];
        uiSleep 0.05;
        uiNamespace setVariable ["RMTFAR_hudDisp", displayNull];
        RMTFAR_HUD_LAYER cutRsc ["RMTFAR_RadioHud", "PLAIN", -1, false];

        private _t0 = diag_tickTime + 5;
        waitUntil {
            !isNull (uiNamespace getVariable ["RMTFAR_hudDisp", displayNull])
            || {!RMTFAR_enabled}
            || {!missionNamespace getVariable ["RMTFAR_showRadioHud", true]}
            || {diag_tickTime > _t0}
        };

        private _dsp = uiNamespace getVariable ["RMTFAR_hudDisp", displayNull];
        private _go =
            RMTFAR_enabled
            && {missionNamespace getVariable ["RMTFAR_showRadioHud", true]}
            && {!isNull _dsp};

        if (!_go) then {
            RMTFAR_HUD_LAYER cutRsc ["Default", "PLAIN", -1, false];
            uiNamespace setVariable ["RMTFAR_hudDisp", displayNull];
            if (RMTFAR_enabled && {missionNamespace getVariable ["RMTFAR_showRadioHud", true]} && {isNull _dsp}) then {
                diag_log "RMTFAR HUD: timeout creando display";
            };
        } else {
            while {
                RMTFAR_enabled
                && {missionNamespace getVariable ["RMTFAR_showRadioHud", true]}
            } do {
                private _ctrl = _dsp displayCtrl RMTFAR_HUD_IDC;
                if (isNull _ctrl) exitWith {};

                private _lr = if (RMTFAR_radioFreqLR != "") then {
                    private _lrSt = ["B", "L", "R"] select ((RMTFAR_radioStereoLR max 0) min 2);
                    format [
                        "<br/><t size='0.85' color='#aaccee'>LR %1 MHz · C%2 · %3</t>",
                        RMTFAR_radioFreqLR,
                        RMTFAR_radioChannelLR,
                        _lrSt
                    ]
                } else {
                    "<br/><t size='0.85' color='#667788'>LR sin sintonizar</t>"
                };

                private _txSr = if (RMTFAR_pttRadioSR) then {
                    "<br/><t size='0.9' color='#ff7744'>● TX radio SR</t>"
                } else {
                    ""
                };

                private _txLr = if (RMTFAR_pttRadioLR) then {
                    "<br/><t size='0.9' color='#ffaa44'>● TX radio LR</t>"
                } else {
                    ""
                };

                private _txLoc = if (RMTFAR_pttLocal) then {
                    "<br/><t size='0.85' color='#eedd66'>● TX voz directa</t>"
                } else {
                    ""
                };

                private _active = toUpper (missionNamespace getVariable ["RMTFAR_activeRadio", "SR"]);
                private _activeTxt = format ["<br/><t size='0.8' color='#99ccaa'>Activa: %1</t>", _active];
                private _icTxt = format [
                    "<br/><t size='0.8' color='#cceea0'>IC %1 · C%2</t>",
                    if (RMTFAR_intercomEnabled) then {"ON"} else {"OFF"},
                    RMTFAR_intercomChannel
                ];
                private _icVehTxt = if ((missionNamespace getVariable ["RMTFAR_showIntercomDebug", false]) && {vehicle player != player}) then {
                    format ["<br/><t size='0.75' color='#99aa88'>IC-Veh %1</t>", netId (vehicle player)]
                } else {
                    ""
                };

                private _srSt = ["B", "L", "R"] select ((RMTFAR_radioStereo max 0) min 2);
                private _html = format [
                    "<t size='0.75' color='#8899aa'>RMTFAR</t><br/><t size='0.9' color='#ddeeff'>SR %1 MHz · C%2 · %3</t>%4%5%6%7%8%9%10",
                    RMTFAR_radioFreq,
                    RMTFAR_radioChannel,
                    _srSt,
                    _lr,
                    _txSr,
                    _txLr,
                    _txLoc,
                    _activeTxt,
                    _icTxt,
                    _icVehTxt
                ];

                _ctrl ctrlSetStructuredText parseText _html;
                uiSleep 0.25;
            };

            RMTFAR_HUD_LAYER cutRsc ["Default", "PLAIN", -1, false];
            uiNamespace setVariable ["RMTFAR_hudDisp", displayNull];
        };

        uiSleep 0.2;
    };
};
