// SPDX-License-Identifier: GPL-3.0
// fn_toggleIntercom.sqf - Activa/desactiva intercom local.

RMTFAR_intercomEnabled = !RMTFAR_intercomEnabled;
[format ["Intercom: %1", if (RMTFAR_intercomEnabled) then {"ON"} else {"OFF"}], true] call CBA_fnc_notify;
diag_log format ["RMTFAR: intercom enabled=%1", RMTFAR_intercomEnabled];
[] call RMTFAR_fnc_saveProfileSettings;
