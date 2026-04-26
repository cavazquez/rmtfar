#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0
# pack-pbo.sh — Empaqueta los addons de @rmtfar en PBO para Arma 3.
#
# Requisitos:
#   cargo install armake2
#
# Uso:
#   ./scripts/pack-pbo.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$REPO_ROOT"

if ! command -v armake2 &>/dev/null; then
    echo "ERROR: armake2 no encontrado."
    echo "  Instalar con: cargo install armake2"
    exit 1
fi

SRC="arma-mod/@rmtfar/addons/rmtfar"
DEST="arma-mod/@rmtfar/addons/rmtfar.pbo"

if [ ! -d "$SRC" ]; then
    echo "ERROR: no se encontró $SRC"
    exit 1
fi

echo "=== Empaquetando PBO ==="
armake2 pack "$SRC" "$DEST"

echo ""
echo "=== PBO creado ==="
ls -lh "$DEST"
