// SPDX-License-Identifier: GPL-3.0
// fn_init.sqf - Inicialización del mod (llamada en preInit via CBA)
// Registra keybinds CBA para PTT.

if (!hasInterface) exitWith {};

// --- CBA Keybinds ---
// Si el jugador ya tiene teclas guardadas para la acción, no se pisa el perfil (_overwrite false).
// Si no hay teclas reales (nueva instalación o solo KEYBIND_NULL), se aplican defaults del mod.
private _owLocal = !(["RMTFAR", "PTTLocal"] call RMTFAR_fnc_cbaKeybindHasUserKeys);
private _owSR = !(["RMTFAR", "PTTRadioSR"] call RMTFAR_fnc_cbaKeybindHasUserKeys);
private _owLR = !(["RMTFAR", "PTTRadioLR"] call RMTFAR_fnc_cbaKeybindHasUserKeys);
private _owActive = !(["RMTFAR", "PTTRadioActive"] call RMTFAR_fnc_cbaKeybindHasUserKeys);
private _owAdditional = !(["RMTFAR", "PTTRadioAdditional"] call RMTFAR_fnc_cbaKeybindHasUserKeys);
private _owToggle = !(["RMTFAR", "ToggleActiveRadio"] call RMTFAR_fnc_cbaKeybindHasUserKeys);
private _owCh1 = !(["RMTFAR", "SetActiveChannel1"] call RMTFAR_fnc_cbaKeybindHasUserKeys);
private _owCh2 = !(["RMTFAR", "SetActiveChannel2"] call RMTFAR_fnc_cbaKeybindHasUserKeys);
private _owCh3 = !(["RMTFAR", "SetActiveChannel3"] call RMTFAR_fnc_cbaKeybindHasUserKeys);
private _owCh4 = !(["RMTFAR", "SetActiveChannel4"] call RMTFAR_fnc_cbaKeybindHasUserKeys);
private _owCh5 = !(["RMTFAR", "SetActiveChannel5"] call RMTFAR_fnc_cbaKeybindHasUserKeys);
private _owCh6 = !(["RMTFAR", "SetActiveChannel6"] call RMTFAR_fnc_cbaKeybindHasUserKeys);
private _owCh7 = !(["RMTFAR", "SetActiveChannel7"] call RMTFAR_fnc_cbaKeybindHasUserKeys);
private _owCh8 = !(["RMTFAR", "SetActiveChannel8"] call RMTFAR_fnc_cbaKeybindHasUserKeys);
private _owCh9 = !(["RMTFAR", "SetActiveChannel9"] call RMTFAR_fnc_cbaKeybindHasUserKeys);
private _owChNext = !(["RMTFAR", "CycleActiveChannelNext"] call RMTFAR_fnc_cbaKeybindHasUserKeys);
private _owChPrev = !(["RMTFAR", "CycleActiveChannelPrev"] call RMTFAR_fnc_cbaKeybindHasUserKeys);
private _owStBoth = !(["RMTFAR", "SetActiveStereoBoth"] call RMTFAR_fnc_cbaKeybindHasUserKeys);
private _owStLeft = !(["RMTFAR", "SetActiveStereoLeft"] call RMTFAR_fnc_cbaKeybindHasUserKeys);
private _owStRight = !(["RMTFAR", "SetActiveStereoRight"] call RMTFAR_fnc_cbaKeybindHasUserKeys);
private _owStNext = !(["RMTFAR", "CycleActiveStereoNext"] call RMTFAR_fnc_cbaKeybindHasUserKeys);
private _owStPrev = !(["RMTFAR", "CycleActiveStereoPrev"] call RMTFAR_fnc_cbaKeybindHasUserKeys);
private _owIcToggle = !(["RMTFAR", "ToggleIntercom"] call RMTFAR_fnc_cbaKeybindHasUserKeys);
private _owIcNext = !(["RMTFAR", "CycleIntercomChannelNext"] call RMTFAR_fnc_cbaKeybindHasUserKeys);
private _owIcPrev = !(["RMTFAR", "CycleIntercomChannelPrev"] call RMTFAR_fnc_cbaKeybindHasUserKeys);
private _owIcDbg = !(["RMTFAR", "ToggleIntercomDebug"] call RMTFAR_fnc_cbaKeybindHasUserKeys);

["RMTFAR", "PTTLocal", "PTT - Voz Directa", {
    RMTFAR_pttLocal = true;
}, {
    RMTFAR_pttLocal = false;
}, [0, [false, false, false]], false, 0, _owLocal] call CBA_fnc_addKeybind;

["RMTFAR", "PTTRadioSR", "PTT - Radio (Corto Alcance)", {
    RMTFAR_pttRadioSR = true;
    ["on"] call RMTFAR_fnc_playRadioSquelch;
}, {
    RMTFAR_pttRadioSR = false;
    ["off"] call RMTFAR_fnc_playRadioSquelch;
}, [0, [false, false, false]], false, 0, _owSR] call CBA_fnc_addKeybind;

["RMTFAR", "PTTRadioLR", "PTT - Radio (Largo Alcance)", {
    RMTFAR_pttRadioLR = true;
    ["on"] call RMTFAR_fnc_playRadioSquelch;
}, {
    RMTFAR_pttRadioLR = false;
    ["off"] call RMTFAR_fnc_playRadioSquelch;
}, [0, [false, false, false]], false, 0, _owLR] call CBA_fnc_addKeybind;

["RMTFAR", "PTTRadioActive", "PTT - Radio activa (SR/LR)", {
    [true] call RMTFAR_fnc_radioTransmitActive;
    ["on"] call RMTFAR_fnc_playRadioSquelch;
}, {
    [false] call RMTFAR_fnc_radioTransmitActive;
    ["off"] call RMTFAR_fnc_playRadioSquelch;
}, [0x3A, [false, false, false]], false, 0, _owActive] call CBA_fnc_addKeybind;

["RMTFAR", "PTTRadioAdditional", "PTT - Radio adicional (opuesta a activa)", {
    [true] call RMTFAR_fnc_radioTransmitAdditional;
    ["on"] call RMTFAR_fnc_playRadioSquelch;
}, {
    [false] call RMTFAR_fnc_radioTransmitAdditional;
    ["off"] call RMTFAR_fnc_playRadioSquelch;
}, [0, [false, false, false]], false, 0, _owAdditional] call CBA_fnc_addKeybind;

["RMTFAR", "ToggleActiveRadio", "Alternar radio activa (SR/LR)", {
    [] call RMTFAR_fnc_toggleActiveRadio;
}, {}, [0, [false, false, false]], false, 0, _owToggle] call CBA_fnc_addKeybind;

["RMTFAR", "SetActiveChannel1", "Canal 1 (radio activa)", {
    [1] call RMTFAR_fnc_setChannelActive;
}, {}, [79, [false, false, false]], false, 0, _owCh1] call CBA_fnc_addKeybind;
["RMTFAR", "SetActiveChannel2", "Canal 2 (radio activa)", {
    [2] call RMTFAR_fnc_setChannelActive;
}, {}, [80, [false, false, false]], false, 0, _owCh2] call CBA_fnc_addKeybind;
["RMTFAR", "SetActiveChannel3", "Canal 3 (radio activa)", {
    [3] call RMTFAR_fnc_setChannelActive;
}, {}, [81, [false, false, false]], false, 0, _owCh3] call CBA_fnc_addKeybind;
["RMTFAR", "SetActiveChannel4", "Canal 4 (radio activa)", {
    [4] call RMTFAR_fnc_setChannelActive;
}, {}, [75, [false, false, false]], false, 0, _owCh4] call CBA_fnc_addKeybind;
["RMTFAR", "SetActiveChannel5", "Canal 5 (radio activa)", {
    [5] call RMTFAR_fnc_setChannelActive;
}, {}, [76, [false, false, false]], false, 0, _owCh5] call CBA_fnc_addKeybind;
["RMTFAR", "SetActiveChannel6", "Canal 6 (radio activa)", {
    [6] call RMTFAR_fnc_setChannelActive;
}, {}, [77, [false, false, false]], false, 0, _owCh6] call CBA_fnc_addKeybind;
["RMTFAR", "SetActiveChannel7", "Canal 7 (radio activa)", {
    [7] call RMTFAR_fnc_setChannelActive;
}, {}, [71, [false, false, false]], false, 0, _owCh7] call CBA_fnc_addKeybind;
["RMTFAR", "SetActiveChannel8", "Canal 8 (radio activa)", {
    [8] call RMTFAR_fnc_setChannelActive;
}, {}, [72, [false, false, false]], false, 0, _owCh8] call CBA_fnc_addKeybind;
["RMTFAR", "SetActiveChannel9", "Canal 9 (radio activa)", {
    [9] call RMTFAR_fnc_setChannelActive;
}, {}, [73, [false, false, false]], false, 0, _owCh9] call CBA_fnc_addKeybind;
["RMTFAR", "CycleActiveChannelNext", "Canal siguiente (radio activa)", {
    ["next"] call RMTFAR_fnc_cycleChannelActive;
}, {}, [201, [false, true, false]], false, 0, _owChNext] call CBA_fnc_addKeybind;
["RMTFAR", "CycleActiveChannelPrev", "Canal anterior (radio activa)", {
    ["prev"] call RMTFAR_fnc_cycleChannelActive;
}, {}, [209, [false, true, false]], false, 0, _owChPrev] call CBA_fnc_addKeybind;
["RMTFAR", "SetActiveStereoBoth", "Estereo Both (radio activa)", {
    [0] call RMTFAR_fnc_setStereoActive;
}, {}, [200, [false, true, false]], false, 0, _owStBoth] call CBA_fnc_addKeybind;
["RMTFAR", "SetActiveStereoLeft", "Estereo Left (radio activa)", {
    [1] call RMTFAR_fnc_setStereoActive;
}, {}, [203, [false, true, false]], false, 0, _owStLeft] call CBA_fnc_addKeybind;
["RMTFAR", "SetActiveStereoRight", "Estereo Right (radio activa)", {
    [2] call RMTFAR_fnc_setStereoActive;
}, {}, [205, [false, true, false]], false, 0, _owStRight] call CBA_fnc_addKeybind;
["RMTFAR", "CycleActiveStereoNext", "Estereo siguiente (radio activa)", {
    ["next"] call RMTFAR_fnc_cycleStereoActive;
}, {}, [0, [false, false, false]], false, 0, _owStNext] call CBA_fnc_addKeybind;
["RMTFAR", "CycleActiveStereoPrev", "Estereo anterior (radio activa)", {
    ["prev"] call RMTFAR_fnc_cycleStereoActive;
}, {}, [0, [false, false, false]], false, 0, _owStPrev] call CBA_fnc_addKeybind;
["RMTFAR", "ToggleIntercom", "Intercom ON/OFF", {
    [] call RMTFAR_fnc_toggleIntercom;
}, {}, [0, [false, false, false]], false, 0, _owIcToggle] call CBA_fnc_addKeybind;
["RMTFAR", "CycleIntercomChannelNext", "Intercom canal siguiente", {
    ["next"] call RMTFAR_fnc_cycleIntercomChannel;
}, {}, [201, [false, false, true]], false, 0, _owIcNext] call CBA_fnc_addKeybind;
["RMTFAR", "CycleIntercomChannelPrev", "Intercom canal anterior", {
    ["prev"] call RMTFAR_fnc_cycleIntercomChannel;
}, {}, [209, [false, false, true]], false, 0, _owIcPrev] call CBA_fnc_addKeybind;
["RMTFAR", "ToggleIntercomDebug", "Intercom debug HUD ON/OFF", {
    [] call RMTFAR_fnc_toggleIntercomDebug;
}, {}, [0, [false, false, false]], false, 0, _owIcDbg] call CBA_fnc_addKeybind;

// --- DEBUG / test en Windows (Ctrl + Shift + F7–F10) — ver docs/windows-ghost-test.md
["RMTFAR", "DebugToggle", "DEBUG: modo test on/off", {
    [] call RMTFAR_fnc_debugToggle;
}, {}, [0x41, [true, true, false]]] call CBA_fnc_addKeybind;

["RMTFAR", "DebugSpawnGhost", "DEBUG: spawn ghost SR", {
    [] call RMTFAR_fnc_debugGhostSpawn;
}, {}, [0x42, [true, true, false]]] call CBA_fnc_addKeybind;

["RMTFAR", "DebugGhostPTT", "DEBUG: ghost #0 PTT SR toggle", {
    [] call RMTFAR_fnc_debugGhostToggleTransmit;
}, {}, [0x43, [true, true, false]]] call CBA_fnc_addKeybind;

["RMTFAR", "DebugCycleFreq", "DEBUG: rotar SR local (152/43/50)", {
    [] call RMTFAR_fnc_debugCycleFreq;
}, {}, [0x44, [true, true, false]]] call CBA_fnc_addKeybind;

diag_log "RMTFAR: Keybinds registered";
