#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0
# build-windows.sh — Cross-compila binarios Windows desde Linux (mingw-w64).
#
# Qué produce:
#   debug   → target/x86_64-pc-windows-gnu/debug/   (ignorado por git)
#   release → dist/windows-x64/arma3/@rmtfar/
#             dist/windows-x64/mumble/
#
# Requisitos (una sola vez):
#   sudo apt install mingw-w64
#   rustup target add x86_64-pc-windows-gnu
#   cargo install armake2
#
# Uso:
#   ./scripts/build-windows.sh           # debug (solo compila, queda en target/)
#   ./scripts/build-windows.sh --release # release (genera dist/windows-x64/)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

RELEASE=0
for arg in "$@"; do
    case "$arg" in
        --release) RELEASE=1 ;;
        *) echo "Argumento desconocido: $arg"; exit 1 ;;
    esac
done

TARGET="x86_64-pc-windows-gnu"
PROFILE="$( [[ $RELEASE -eq 1 ]] && echo "release" || echo "debug" )"
CARGO_FLAGS="$( [[ $RELEASE -eq 1 ]] && echo "--release" || echo "" )"
OUT_DIR="target/$TARGET/$PROFILE"

DIST_ARMA="dist/windows-x64/arma3/@rmtfar"
DIST_MUMBLE="dist/windows-x64/mumble"

if [[ $RELEASE -eq 0 ]]; then
    echo "⚠  Modo debug — los binarios quedan en $OUT_DIR (ignorado por git)."
    echo "   Usá --release para generar en dist/windows-x64/."
fi

echo "=== Build Windows ($TARGET, $PROFILE) ==="

# ── Verificar toolchain ────────────────────────────────────────────────────────
if ! command -v x86_64-w64-mingw32-gcc &>/dev/null; then
    echo "ERROR: mingw-w64 no encontrado. Instalar con: sudo apt install mingw-w64"
    exit 1
fi

if ! rustup target list --installed | grep -q "$TARGET"; then
    echo "Instalando Rust target $TARGET..."
    rustup target add "$TARGET"
fi

# ── 1. Extension DLL (Arma 3) ─────────────────────────────────────────────────
echo ""
echo "--- Extension DLL (Arma 3) ---"
cargo build -p rmtfar-extension --target "$TARGET" ${CARGO_FLAGS}

if [[ $RELEASE -eq 1 ]]; then
    mkdir -p "$DIST_ARMA"
    cp "$OUT_DIR/rmtfar.dll" "$DIST_ARMA/rmtfar_x64.dll"
    echo "  → $DIST_ARMA/rmtfar_x64.dll ($(du -sh "$DIST_ARMA/rmtfar_x64.dll" | cut -f1))"
else
    cp "$OUT_DIR/rmtfar.dll" "$OUT_DIR/rmtfar_x64.dll"
    echo "  → $OUT_DIR/rmtfar_x64.dll ($(du -sh "$OUT_DIR/rmtfar_x64.dll" | cut -f1))"
fi

# ── 2. Plugin DLL (Mumble) ────────────────────────────────────────────────────
echo ""
echo "--- Plugin DLL (Mumble) ---"
cargo build -p rmtfar-plugin --target "$TARGET" ${CARGO_FLAGS}

if [[ $RELEASE -eq 1 ]]; then
    mkdir -p "$DIST_MUMBLE"
    cp "$OUT_DIR/rmtfar_plugin.dll" "$DIST_MUMBLE/rmtfar_plugin.dll"
    echo "  → $DIST_MUMBLE/rmtfar_plugin.dll ($(du -sh "$DIST_MUMBLE/rmtfar_plugin.dll" | cut -f1))"
else
    echo "  → $OUT_DIR/rmtfar_plugin.dll ($(du -sh "$OUT_DIR/rmtfar_plugin.dll" | cut -f1))"
fi

# ── 3. PBO + mod.cpp (solo release) ──────────────────────────────────────────
if [[ $RELEASE -eq 1 ]]; then
    echo ""
    echo "--- PBO ---"
    bash "$SCRIPT_DIR/pack-pbo.sh"

    cp "$REPO_ROOT/mod.cpp" "$DIST_ARMA/mod.cpp"
    echo "  → $DIST_ARMA/mod.cpp"
fi

# ── Verificar exports ─────────────────────────────────────────────────────────
EXT_DLL="$( [[ $RELEASE -eq 1 ]] && echo "$DIST_ARMA/rmtfar_x64.dll"    || echo "$OUT_DIR/rmtfar_x64.dll" )"
PLG_DLL="$( [[ $RELEASE -eq 1 ]] && echo "$DIST_MUMBLE/rmtfar_plugin.dll" || echo "$OUT_DIR/rmtfar_plugin.dll" )"

echo ""
echo "=== Verificando exports ==="
echo "Extension (RVExtension*):"
x86_64-w64-mingw32-objdump -p "$EXT_DLL" \
    | grep -E "RVExtension" || echo "  (sin exports visibles con objdump)"

echo "Plugin (mumble_*):"
x86_64-w64-mingw32-objdump -p "$PLG_DLL" \
    | grep -E "mumble_" | head -5 || echo "  (sin exports visibles con objdump)"

# ── Resumen ───────────────────────────────────────────────────────────────────
echo ""
echo "=== Build Windows completado ==="
if [[ $RELEASE -eq 1 ]]; then
    echo ""
    echo "  Windows / Arma 3  → $DIST_ARMA/"
    ls -lh "$DIST_ARMA/rmtfar_x64.dll" "$DIST_ARMA/addons/rmtfar.pbo" 2>/dev/null | awk '{print "    "$NF, $5}'
    echo ""
    echo "  Windows / Mumble  → $DIST_MUMBLE/"
    ls -lh "$DIST_MUMBLE/rmtfar_plugin.dll" 2>/dev/null | awk '{print "    "$NF, $5}'
    echo ""
    echo "  Instrucciones:"
    echo "    Arma 3 : copiar $DIST_ARMA/ a la carpeta de mods"
    echo "    Mumble : copiar $DIST_MUMBLE/rmtfar_plugin.dll a %APPDATA%\\Mumble\\Plugins\\"
else
    echo "Binarios de desarrollo en $OUT_DIR/:"
    ls -lh "$OUT_DIR/rmtfar_x64.dll" "$OUT_DIR/rmtfar_plugin.dll"
fi
