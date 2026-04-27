// SPDX-License-Identifier: GPL-3.0
#include "CfgFunctions.hpp"
#include "CfgRscTitles.hpp"

class CfgPatches {
    class rmtfar_main {
        name = "RMTFAR Main";
        units[] = {};
        weapons[] = {};
        requiredVersion = 0.1;
        requiredAddons[] = {"cba_main"};
        author = "RMTFAR Team";
        version = "0.1.0";
    };
};

class Extended_PreInit_EventHandlers {
    class rmtfar {
        init = "call compile preprocessFileLineNumbers '\rmtfar\addons\rmtfar\XEH_preInit.sqf'";
    };
};

class Extended_PostInit_EventHandlers {
    class rmtfar {
        init = "call compile preprocessFileLineNumbers '\rmtfar\addons\rmtfar\XEH_postInit.sqf'";
    };
};

// Modelos de alcance por classname de ítem y multiplicador por facción (ver fn_resolveRadioModel.sqf).
class CfgRMTFAR {
    class RadioItems {
        class Default {
            rangeSR = 5000;
            rangeLR = 20000;
        };
        // Radio personal Arma 3: SR típico, sin LR de mochila en este ítem.
        class ItemRadio {
            rangeSR = 5000;
            rangeLR = 0;
        };
    };
    class RadioFactions {
        class BLU_F {
            rangeMult = 1;
        };
        class OPF_F {
            rangeMult = 1;
        };
        class IND_F {
            rangeMult = 1;
        };
    };
};
