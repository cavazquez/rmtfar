// SPDX-License-Identifier: GPL-3.0
#include "CfgFunctions.hpp"

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
