// SPDX-License-Identifier: GPL-3.0
// params: [_addon, _actionId]
// Devuelve true si en el perfil de CBA hay al menos una tecla "real" para la acción
// (misma regla que CBA: DIK > DIK_ESCAPE; 0 = sin tecla por defecto).

params [["_addon", "RMTFAR", [""]], ["_actionId", "", [""]]];

private _reg = profileNamespace getVariable ["cba_keybinding_registry_v3", nil];
if (isNil "_reg") exitWith { false };

private _action = toLower format ["%1$%2", _addon, _actionId];
private _list = [_reg, _action] call CBA_fnc_hashGet;

if (isNil "_list") exitWith { false };
if !(_list isEqualType []) exitWith { false };

count (_list select { (_x select 0) > 1 }) > 0
