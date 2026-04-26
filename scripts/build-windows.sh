#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0
# build-windows.sh — Cross-compila todos los binarios Windows desde Linux (mingw-w64).
#
# Qué produce:
#   debug   → target/x86_64-pc-windows-gnu/debug/   (ignorado por git)
#   release → arma-mod/@rmtfar/                      (commiteado, listo para distribuir)
#
# Requisitos (una sola vez):
#   sudo apt install mingw-w64
#   rustup target add x86_64-pc-windows-gnu
#
# Uso:
#   ./scripts/build-windows.sh           # debug  (solo compila, no toca arma-mod/)
#   RELEASE=1 ./scripts/build-windows.sh # release (copia a arma-mod/@rmtfar/)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

TARGET="x86_64-pc-windows-gnu"
PROFILE="${RELEASE:+release}"
PROFILE="${PROFILE:-debug}"
CARGO_FLAGS="${RELEASE:+--release}"
OUT_DIR="target/$TARGET/$PROFILE"

if [[ "$PROFILE" == "debug" ]]; then
    echo "⚠  Modo debug — los binarios quedan en $OUT_DIR (ignorado por git)."
    echo "   Usa RELEASE=1 ./scripts/build-windows.sh para generar binarios de distribución."
fi

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

# Cargo genera rmtfar.dll; Arma 3 espera rmtfar_x64.dll en sistemas 64-bit.
# En release, copiamos a arma-mod/; en debug, solo renombramos dentro de target/.
if [[ "$PROFILE" == "release" ]]; then
    cp "$OUT_DIR/rmtfar.dll" "arma-mod/@rmtfar/rmtfar_x64.dll"
    echo "  → arma-mod/@rmtfar/rmtfar_x64.dll ($(du -sh "arma-mod/@rmtfar/rmtfar_x64.dll" | cut -f1))"
else
    cp "$OUT_DIR/rmtfar.dll" "$OUT_DIR/rmtfar_x64.dll"
    echo "  → $OUT_DIR/rmtfar_x64.dll ($(du -sh "$OUT_DIR/rmtfar_x64.dll" | cut -f1))"
fi

# ── 2. Plugin DLL (Mumble) ────────────────────────────────────────────────────
echo ""
echo "--- Plugin DLL (Mumble) ---"
cargo build -p rmtfar-plugin --target "$TARGET" ${CARGO_FLAGS:-}

if [[ "$PROFILE" == "release" ]]; then
    cp "$OUT_DIR/rmtfar_plugin.dll" "arma-mod/@rmtfar/rmtfar_plugin.dll"
    echo "  → arma-mod/@rmtfar/rmtfar_plugin.dll ($(du -sh "arma-mod/@rmtfar/rmtfar_plugin.dll" | cut -f1))"
else
    echo "  → $OUT_DIR/rmtfar_plugin.dll ($(du -sh "$OUT_DIR/rmtfar_plugin.dll" | cut -f1))"
fi

# ── Verificar exports ─────────────────────────────────────────────────────────
EXT_DLL="$( [[ "$PROFILE" == "release" ]] && echo "arma-mod/@rmtfar/rmtfar_x64.dll"    || echo "$OUT_DIR/rmtfar_x64.dll" )"
PLG_DLL="$( [[ "$PROFILE" == "release" ]] && echo "arma-mod/@rmtfar/rmtfar_plugin.dll" || echo "$OUT_DIR/rmtfar_plugin.dll" )"

echo ""
echo "=== Verificando exports ==="
echo "Extension (RVExtension*):"
x86_64-w64-mingw32-objdump -p "$EXT_DLL" \
    | grep -E "RVExtension" || echo "  (sin exports visibles con objdump)"

echo "Plugin (mumble_*):"
x86_64-w64-mingw32-objdump -p "$PLG_DLL" \
    | grep -E "mumble_" | head -5 || echo "  (sin exports visibles con objdump)"

echo ""
echo "=== Build Windows completado ==="
if [[ "$PROFILE" == "release" ]]; then
    echo "Binarios listos para distribuir en arma-mod/@rmtfar/:"
    ls -lh "arma-mod/@rmtfar/rmtfar_x64.dll" "arma-mod/@rmtfar/rmtfar_plugin.dll"
    echo ""
    echo "Instalar en Windows:"
    echo "  rmtfar_x64.dll + addons/ → carpeta @rmtfar de Arma 3"
    echo "  rmtfar_plugin.dll        → %APPDATA%\\Mumble\\Plugins\\"
else
    echo "Binarios de desarrollo en $OUT_DIR/:"
    ls -lh "$OUT_DIR/rmtfar_x64.dll" "$OUT_DIR/rmtfar_plugin.dll"
fi
