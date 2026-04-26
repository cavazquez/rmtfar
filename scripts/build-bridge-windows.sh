#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0
# build-bridge-windows.sh — Cross-compila rmtfar-bridge para Windows x86_64.
#
# Requisitos (instalar una vez):
#   sudo apt install mingw-w64
#   rustup target add x86_64-pc-windows-gnu
#
# Uso:
#   ./scripts/build-bridge-windows.sh           # debug
#   RELEASE=1 ./scripts/build-bridge-windows.sh # release

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$REPO_ROOT"

TARGET="x86_64-pc-windows-gnu"
PROFILE="${RELEASE:+release}"
PROFILE="${PROFILE:-debug}"
CARGO_FLAGS="${RELEASE:+--release}"

echo "=== Build Bridge para Windows ($TARGET, $PROFILE) ==="

if ! command -v x86_64-w64-mingw32-gcc &>/dev/null; then
    echo "ERROR: x86_64-w64-mingw32-gcc no encontrado."
    echo "  Instalar con: sudo apt install mingw-w64"
    exit 1
fi

if ! rustup target list --installed | grep -q "$TARGET"; then
    echo "Instalando Rust target $TARGET..."
    rustup target add "$TARGET"
fi

cargo build -p rmtfar-bridge --target "$TARGET" ${CARGO_FLAGS:-}

SRC="target/$TARGET/$PROFILE/rmtfar-bridge.exe"
DEST_DIR="arma-mod/@rmtfar"
DEST="$DEST_DIR/rmtfar-bridge.exe"

if [ ! -f "$SRC" ]; then
    echo "ERROR: No se encontró $SRC"
    exit 1
fi

cp "$SRC" "$DEST"

echo ""
echo "=== Bridge compilado exitosamente ==="
ls -lh "$SRC"
echo ""
echo "Instalado en: $DEST"
echo ""
echo "En Windows, ejecutar:"
echo "  rmtfar-bridge.exe --local-id TuNombreMumble"
