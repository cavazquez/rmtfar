// SPDX-License-Identifier: GPL-3.0
// fn_saveProfileSettings.sqf - Persiste ajustes locales de RMTFAR en profileNamespace.

private _payload = [
    toUpper (missionNamespace getVariable ["RMTFAR_activeRadio", "SR"]),
    (RMTFAR_radioStereo max 0) min 2,
    (RMTFAR_radioStereoLR max 0) min 2,
    RMTFAR_intercomEnabled,
    (RMTFAR_intercomChannel max 1) min 3,
    missionNamespace getVariable ["RMTFAR_showIntercomDebug", false]
];

profileNamespace setVariable ["RMTFAR_profileSettings_v1", _payload];
saveProfileNamespace;
