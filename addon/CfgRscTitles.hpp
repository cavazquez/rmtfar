// SPDX-License-Identifier: GPL-3.0
// HUD de radio (capa RscTitles). Nombre de recurso = RMTFAR_RadioHud (cutRsc).

class RscText;
class RscStructuredText;

class RscTitles {
    class RMTFAR_RadioHud {
        idd = -1;
        movingEnable = 0;
        duration = 1e+011;
        fadeIn = 0;
        fadeOut = 0;
        name = "RMTFAR_RadioHud";

        class controls {
            class RMTFAR_hudBg: RscText {
                idc = -1;
                x = safeZoneX + safeZoneW * 0.72;
                y = safeZoneY + safeZoneH * 0.66;
                w = safeZoneW * 0.27;
                h = safeZoneH * 0.22;
                colorBackground[] = {0.05, 0.05, 0.08, 0.55};
            };
            class RMTFAR_hudText: RscStructuredText {
                idc = 84001;
                x = safeZoneX + safeZoneW * 0.725;
                y = safeZoneY + safeZoneH * 0.665;
                w = safeZoneW * 0.26;
                h = safeZoneH * 0.21;
                shadow = 0;
                colorBackground[] = {0, 0, 0, 0};
                onLoad = "uiNamespace setVariable ['RMTFAR_hudDisp', ctrlParent (_this select 0)];";
            };
        };
    };
};
