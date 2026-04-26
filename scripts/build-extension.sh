#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0
# build-extension.sh — Cross-compila rmtfar-extension para Windows x86_64.
#
# Requisitos (instalar una vez):
#   sudo apt install mingw-w64
#   rustup target add x86_64-pc-windows-gnu
#
# Uso:
#   ./scripts/build-extension.sh           # debug
#   RELEASE=1 ./scripts/build-extension.sh # release

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$REPO_ROOT"

TARGET="x86_64-pc-windows-gnu"
PROFILE="${RELEASE:+release}"
PROFILE="${PROFILE:-debug}"
CARGO_FLAGS="${RELEASE:+--release}"

echo "=== Build Extension para Windows ($TARGET, $PROFILE) ==="

# Verificar linker
if ! command -v x86_64-w64-mingw32-gcc &>/dev/null; then
    echo "ERROR: x86_64-w64-mingw32-gcc no encontrado."
    echo "  Instalar con: sudo apt install mingw-w64"
    exit 1
fi

# Verificar target de Rust
if ! rustup target list --installed | grep -q "$TARGET"; then
    echo "Instalando Rust target $TARGET..."
    rustup target add "$TARGET"
fi

# Compilar
cargo build -p rmtfar-extension --target "$TARGET" ${CARGO_FLAGS:-}

# Cargo genera "rmtfar.dll"; Arma 3 espera "rmtfar_x64.dll" en sistemas 64-bit.
SRC="target/$TARGET/$PROFILE/rmtfar.dll"
DEST_NAME="rmtfar_x64.dll"
DEST_MOD="arma-mod/@rmtfar/$DEST_NAME"

if [ ! -f "$SRC" ]; then
    echo "ERROR: No se encontró $SRC"
    exit 1
fi

cp "$SRC" "$DEST_MOD"

echo ""
echo "=== DLL compilada exitosamente ==="
ls -lh "$SRC"
echo ""
echo "Instalada en: $DEST_MOD"
echo ""
echo "Para verificar los símbolos exportados:"
echo "  x86_64-w64-mingw32-nm --demangle $DEST_MOD | grep -E 'RVExtension'"
