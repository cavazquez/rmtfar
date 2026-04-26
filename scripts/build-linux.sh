#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0
# build-linux.sh — Compila binarios Linux e instala el plugin en Mumble.
#
# Qué produce:
#   debug   → target/debug/             (ignorado por git)
#   release → dist/linux/mumble/        (ignorado por git)
#             + instala en Mumble local
#
# Nota: Arma 3 no tiene binario nativo Linux. Si corrés Arma 3 via Proton/Steam Play,
#       usá los binarios de dist/windows-x64/arma3/ generados por build-windows.sh --release.
#
# Uso:
#   ./scripts/build-linux.sh           # debug
#   ./scripts/build-linux.sh --release # release (genera dist/linux/ e instala en Mumble)

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

PROFILE="$( [[ $RELEASE -eq 1 ]] && echo "release" || echo "debug" )"
CARGO_FLAGS=( $( [[ $RELEASE -eq 1 ]] && echo "--release" || echo "" ) )

DIST_MUMBLE="dist/linux/mumble"

echo "=== Build Linux ($PROFILE) ==="
cargo build "${CARGO_FLAGS[@]}" \
    -p rmtfar-plugin \
    -p rmtfar-bridge \
    -p rmtfar-test-client

SO="target/$PROFILE/librmtfar_plugin.so"

if [[ ! -f "$SO" ]]; then
    echo "ERROR: $SO no encontrado."
    exit 1
fi

echo ""
echo "Binarios:"
ls -lh \
    "target/$PROFILE/librmtfar_plugin.so" \
    "target/$PROFILE/rmtfar-bridge" \
    "target/$PROFILE/rmtfar-test-client" \
    2>/dev/null || true

if [[ $RELEASE -eq 1 ]]; then
    # ── Copiar a dist/ ─────────────────────────────────────────────────────────
    mkdir -p "$DIST_MUMBLE"
    cp "$SO" "$DIST_MUMBLE/librmtfar_plugin.so"

    # ── Instalar en Mumble local ───────────────────────────────────────────────
    echo ""
    echo "=== Instalando plugin en Mumble ==="

    TARGET_REAL="$HOME/.local/share/Mumble/Mumble/Plugins/librmtfar_plugin.so"
    mkdir -p "$(dirname "$TARGET_REAL")"
    cp "$SO" "$TARGET_REAL"
    echo "  $TARGET_REAL"

    TARGET_XDG="$HOME/.local/share/mumble/Plugins/rmtfar.mumble_plugin"
    mkdir -p "$(dirname "$TARGET_XDG")"
    cp "$SO" "$TARGET_XDG"
    echo "  $TARGET_XDG"

    # ── Generar .mumble_plugin ─────────────────────────────────────────────────
    echo ""
    echo "=== Generando .mumble_plugin ==="

    TMPDIR_PKG="$(mktemp -d)"
    trap 'rm -rf "$TMPDIR_PKG"' EXIT

    cat > "$TMPDIR_PKG/manifest.xml" <<'XML'
<?xml version="1.0" encoding="UTF-8"?>
<bundle version="1.0.0">
  <assets>
    <plugin os="linux" arch="x64">librmtfar_plugin.so</plugin>
  </assets>
  <name>RMTFAR</name>
  <version>0.1.0</version>
</bundle>
XML

    cp "$SO" "$TMPDIR_PKG/librmtfar_plugin.so"

    PKG="target/$PROFILE/rmtfar_plugin.mumble_plugin"
    rm -f "$PKG"
    (cd "$TMPDIR_PKG" && zip -j "$REPO_ROOT/$PKG" manifest.xml librmtfar_plugin.so)

    echo "  $(realpath "$PKG")"
    echo ""
    echo "Para instalar desde la UI:"
    echo "  Mumble → Configuración → Complementos → 'Instalar un plugin...' → seleccionar el .mumble_plugin"

    # ── Resumen ────────────────────────────────────────────────────────────────
    echo ""
    echo "=== Build Linux completado ==="
    echo ""
    echo "  Linux / Mumble  → $DIST_MUMBLE/"
    ls -lh "$DIST_MUMBLE/librmtfar_plugin.so" | awk '{print "    "$NF, $5}'
    echo ""
    echo "  Nota: Arma 3 en Linux (Proton/Steam Play) usa los binarios Windows."
    echo "        Generarlos con: ./scripts/build-windows.sh --release"
    echo ""
    echo "Reiniciá Mumble para que tome el nuevo binario."
else
    echo ""
    echo "=== Build Linux completado (debug) ==="
    echo "Binarios de desarrollo en target/debug/."
fi
