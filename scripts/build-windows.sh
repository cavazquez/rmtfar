#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0
# build-windows.sh — Cross-compila todos los binarios Windows desde Linux (mingw-w64).
#
# Qué produce:
#   arma-mod/@rmtfar/rmtfar_x64.dll      ← extension para Arma 3
#   arma-mod/@rmtfar/rmtfar_plugin.dll   ← plugin para Mumble (Windows)
#   arma-mod/@rmtfar/rmtfar-bridge.exe   ← bridge para testing en Windows
#
# Requisitos (una sola vez):
#   sudo apt install mingw-w64
#   rustup target add x86_64-pc-windows-gnu
#
# Uso:
#   ./scripts/build-windows.sh           # debug
#   RELEASE=1 ./scripts/build-windows.sh # release

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

TARGET="x86_64-pc-windows-gnu"
PROFILE="${RELEASE:+release}"
PROFILE="${PROFILE:-debug}"
CARGO_FLAGS="${RELEASE:+--release}"

echo "=== Build Windows ($TARGET, $PROFILE) ==="

# ── Verificar toolchain ────────────────────────────────────────────────────────
if ! command -v x86_64-w64-mingw32-gcc &>/dev/null; then
    echo "ERROR: mingw-w64 no encontrado."
    echo "  Instalar con: sudo apt install mingw-w64"
    exit 1
fi

if ! rustup target list --installed | grep -q "$TARGET"; then
    echo "Instalando Rust target $TARGET..."
    rustup target add "$TARGET"
fi

# ── 1. Extension DLL (Arma 3) ─────────────────────────────────────────────────
echo ""
echo "--- Extension DLL (Arma 3) ---"
cargo build -p rmtfar-extension --target "$TARGET" ${CARGO_FLAGS:-}

# Cargo genera rmtfar.dll; Arma 3 espera rmtfar_x64.dll en sistemas 64-bit
cp "target/$TARGET/$PROFILE/rmtfar.dll" "arma-mod/@rmtfar/rmtfar_x64.dll"
echo "  → arma-mod/@rmtfar/rmtfar_x64.dll ($(du -sh "arma-mod/@rmtfar/rmtfar_x64.dll" | cut -f1))"

# ── 2. Plugin DLL (Mumble) ────────────────────────────────────────────────────
echo ""
echo "--- Plugin DLL (Mumble) ---"
cargo build -p rmtfar-plugin --target "$TARGET" ${CARGO_FLAGS:-}

cp "target/$TARGET/$PROFILE/rmtfar_plugin.dll" "arma-mod/@rmtfar/rmtfar_plugin.dll"
echo "  → arma-mod/@rmtfar/rmtfar_plugin.dll ($(du -sh "arma-mod/@rmtfar/rmtfar_plugin.dll" | cut -f1))"

# ── 3. Bridge (testing Windows) ───────────────────────────────────────────────
echo ""
echo "--- Bridge (testing Windows) ---"
cargo build -p rmtfar-bridge --target "$TARGET" ${CARGO_FLAGS:-}

cp "target/$TARGET/$PROFILE/rmtfar-bridge.exe" "arma-mod/@rmtfar/rmtfar-bridge.exe"
echo "  → arma-mod/@rmtfar/rmtfar-bridge.exe ($(du -sh "arma-mod/@rmtfar/rmtfar-bridge.exe" | cut -f1))"

# ── Verificar exports ─────────────────────────────────────────────────────────
echo ""
echo "=== Verificando exports ==="
echo "Extension (RVExtension*):"
x86_64-w64-mingw32-objdump -p "arma-mod/@rmtfar/rmtfar_x64.dll" \
    | grep -E "RVExtension" || echo "  (sin exports visibles con objdump)"

echo "Plugin (mumble_*):"
x86_64-w64-mingw32-objdump -p "arma-mod/@rmtfar/rmtfar_plugin.dll" \
    | grep -E "mumble_" | head -5 || echo "  (sin exports visibles con objdump)"

echo ""
echo "=== Build Windows completado ==="
echo "Binarios en arma-mod/@rmtfar/:"
ls -lh \
    arma-mod/@rmtfar/rmtfar_x64.dll \
    arma-mod/@rmtfar/rmtfar_plugin.dll \
    arma-mod/@rmtfar/rmtfar-bridge.exe \
    2>/dev/null || true
echo ""
echo "Instalar en Windows:"
echo "  rmtfar_x64.dll + addons/ → carpeta @rmtfar de Arma 3"
echo "  rmtfar_plugin.dll        → %APPDATA%\\Mumble\\Plugins\\"
