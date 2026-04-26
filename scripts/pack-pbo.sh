#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0
# pack-pbo.sh — Empaqueta addon/ en un PBO para Arma 3.
#
# Fuente : addon/
# Destino: dist/windows-x64/arma3/@rmtfar/addons/rmtfar.pbo
#
# Requisitos:
#   cargo install armake2
#
# Uso (normalmente invocado desde build-windows.sh --release):
#   ./scripts/pack-pbo.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

if ! command -v armake2 &>/dev/null; then
    echo "ERROR: armake2 no encontrado. Instalar con: cargo install armake2"
    exit 1
fi

SRC="addon"
DEST_DIR="dist/windows-x64/arma3/@rmtfar/addons"
DEST="$DEST_DIR/rmtfar.pbo"

if [[ ! -d "$SRC" ]]; then
    echo "ERROR: no se encontró $SRC"
    exit 1
fi

mkdir -p "$DEST_DIR"

echo "=== Empaquetando PBO ==="
armake2 pack "$SRC" "$DEST"

echo ""
echo "=== PBO creado ==="
ls -lh "$DEST"
