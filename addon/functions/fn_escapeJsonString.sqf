// SPDX-License-Identifier: GPL-3.0
// Legacy name: ahora escapa campos para payload delimitado por "|".
// Reglas:
//   "\" -> "\\"
//   "|" -> "\|"
params [["_s", "", [""]]];
private _bs = toString [92];
private _pipe = "|";
_s = (_s splitString _bs) joinString (_bs + _bs);
_s = (_s splitString _pipe) joinString (_bs + _pipe);
_s
