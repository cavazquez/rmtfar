#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0
# build-linux.sh — Compila todo el workspace para Linux e instala el plugin en Mumble.
#
# Qué hace:
#   1. Compila rmtfar-plugin, rmtfar-bridge y rmtfar-test-client
#   2. Instala librmtfar_plugin.so en los paths que Mumble busca
#   3. Genera rmtfar_plugin.mumble_plugin (instalable desde la UI de Mumble)
#
# Uso:
#   ./scripts/build-linux.sh           # release (recomendado)
#   ./scripts/build-linux.sh --debug   # debug (más logs de tracing)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

# ── Opciones ──────────────────────────────────────────────────────────────────
PROFILE="release"
CARGO_FLAGS=(--release)
if [[ "${1:-}" == "--debug" ]]; then
    PROFILE="debug"
    CARGO_FLAGS=()
fi

# ── 1. Compilar ───────────────────────────────────────────────────────────────
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

# ── 2. Instalar en Mumble ─────────────────────────────────────────────────────
echo ""
echo "=== Instalando plugin en Mumble ==="

# Path real leído desde mumble_settings.json (Mumble 1.5+)
TARGET_REAL="$HOME/.local/share/Mumble/Mumble/Plugins/librmtfar_plugin.so"
mkdir -p "$(dirname "$TARGET_REAL")"
cp "$SO" "$TARGET_REAL"
echo "  $TARGET_REAL"

# Path estándar XDG
TARGET_XDG="$HOME/.local/share/mumble/Plugins/rmtfar.mumble_plugin"
mkdir -p "$(dirname "$TARGET_XDG")"
cp "$SO" "$TARGET_XDG"
echo "  $TARGET_XDG"

# ── 3. Generar .mumble_plugin (ZIP con manifest.xml) ──────────────────────────
# Instalable desde Mumble → Configuración → Complementos → "Instalar un plugin..."
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
rm -f "$PKG"   # puede existir como .so raw si se usó install-plugin.sh antes
(cd "$TMPDIR_PKG" && zip -j "$REPO_ROOT/$PKG" manifest.xml librmtfar_plugin.so)

echo "  $(realpath "$PKG")"
echo ""
echo "Para instalar desde la UI:"
echo "  Mumble → Configuración → Complementos → 'Instalar un plugin...' → seleccionar el .mumble_plugin"
echo ""
echo "Reiniciá Mumble para que tome el nuevo binario."
