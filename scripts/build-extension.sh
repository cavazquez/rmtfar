#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0
# build-extension.sh — Cross-compila rmtfar-extension para Windows x86_64.
#
# Usa cargo-xwin para producir binarios MSVC nativos desde Linux,
# evitando problemas de inicialización de mingw.
#
# Requisitos (instalar una vez):
#   cargo install cargo-xwin
#   rustup target add x86_64-pc-windows-msvc
#
# Uso:
#   ./scripts/build-extension.sh           # debug
#   RELEASE=1 ./scripts/build-extension.sh # release

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$REPO_ROOT"

TARGET="x86_64-pc-windows-msvc"
PROFILE="${RELEASE:+release}"
PROFILE="${PROFILE:-debug}"
CARGO_FLAGS="${RELEASE:+--release}"

echo "=== Build Extension para Windows ($TARGET, $PROFILE) ==="

# Verificar cargo-xwin
if ! command -v cargo-xwin &>/dev/null; then
    echo "ERROR: cargo-xwin no encontrado."
    echo "  Instalar con: cargo install cargo-xwin"
    exit 1
fi

# Verificar target de Rust
if ! rustup target list --installed | grep -q "$TARGET"; then
    echo "Instalando Rust target $TARGET..."
    rustup target add "$TARGET"
fi

# Compilar
cargo xwin build -p rmtfar-extension --target "$TARGET" ${CARGO_FLAGS:-}

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
echo "Para verificar los exports PE (tabla que Arma 3 usa):"
echo "  x86_64-w64-mingw32-objdump -p $DEST_MOD | grep -A3 'Ordinal/Name'"
