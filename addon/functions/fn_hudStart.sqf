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

                private _lr = "";
                if (RMTFAR_radioFreqLR != "") then {
                    _lr = format [
                        "<br/><t size='0.85' color='#aaccee'>LR %1 MHz · C%2</t>",
                        RMTFAR_radioFreqLR,
                        RMTFAR_radioChannelLR
                    ];
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

                private _html = format [
                    "<t size='0.75' color='#8899aa'>RMTFAR</t><br/><t size='0.9' color='#ddeeff'>SR %1 MHz · C%2</t>%3%4%5%6",
                    RMTFAR_radioFreq,
                    RMTFAR_radioChannel,
                    _lr,
                    _txSr,
                    _txLr,
                    _txLoc
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
