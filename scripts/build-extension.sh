#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0
# build-extension.sh - Compila rmtfar-extension para Windows x86_64
# Requiere: rustup target add x86_64-pc-windows-gnu
#           apt install mingw-w64  (o equivalent)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$REPO_ROOT"

TARGET="x86_64-pc-windows-gnu"
PROFILE="${RELEASE:+release}${RELEASE:-debug}"
CARGO_FLAGS="${RELEASE:+--release}"

echo "=== Build Extension para Windows ($TARGET) ==="

# Verificar que el target está instalado
if ! rustup target list --installed | grep -q "$TARGET"; then
    echo "Instalando target $TARGET..."
    rustup target add "$TARGET"
fi

cargo build -p rmtfar-extension --target "$TARGET" $CARGO_FLAGS

OUTPUT="target/$TARGET/$PROFILE/rmtfar_x64.dll"
if [ -f "$OUTPUT" ]; then
    echo ""
    echo "=== DLL compilada: $OUTPUT ==="
    ls -lh "$OUTPUT"
    echo ""
    echo "Copiar a arma-mod/@rmtfar/rmtfar_x64.dll para instalar."
else
    echo "ERROR: No se encontró $OUTPUT"
    exit 1
fi
