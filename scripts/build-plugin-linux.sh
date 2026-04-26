#!/usr/bin/env bash
# Compila el plugin de Mumble para Linux (x86_64) y lo instala en
# ~/.local/share/mumble/Plugins/ listo para cargar desde Mumble.
#
# Uso:
#   ./scripts/build-plugin-linux.sh          # release
#   ./scripts/build-plugin-linux.sh --debug  # debug (más info de tracing)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(dirname "$SCRIPT_DIR")"
cd "$ROOT"

# ── Opciones ─────────────────────────────────────────────────────────────────
PROFILE="release"
CARGO_FLAGS=(--release)
if [[ "${1:-}" == "--debug" ]]; then
    PROFILE="debug"
    CARGO_FLAGS=()
fi

# ── Build ─────────────────────────────────────────────────────────────────────
echo "==> Compilando rmtfar-plugin ($PROFILE)..."
cargo build "${CARGO_FLAGS[@]}" -p rmtfar-plugin

SO="target/$PROFILE/librmtfar_plugin.so"

if [[ ! -f "$SO" ]]; then
    echo "ERROR: $SO no encontrado. ¿Falló el build?"
    exit 1
fi

echo "==> Compilado: $SO ($(du -sh "$SO" | cut -f1))"

# ── Instalar en Mumble ────────────────────────────────────────────────────────
MUMBLE_PLUGIN_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/mumble/Plugins"
mkdir -p "$MUMBLE_PLUGIN_DIR"

DEST="$MUMBLE_PLUGIN_DIR/librmtfar_plugin.so"
cp "$SO" "$DEST"
echo "==> Instalado en: $DEST"

# ── Instrucciones ─────────────────────────────────────────────────────────────
echo ""
echo "┌─────────────────────────────────────────────────────────────┐"
echo "│  Plugin instalado. Pasos siguientes:                        │"
echo "│                                                             │"
echo "│  1. Abrí Mumble                                             │"
echo "│  2. Configuración → Plugins                                 │"
echo "│  3. Activá 'RMTFAR' en la lista                             │"
echo "│  4. En otra terminal, arrancá el bridge:                    │"
echo "│       cargo run -p rmtfar-bridge                            │"
echo "│  5. Simulá jugadores con el test-client:                    │"
echo "│       cargo run -p rmtfar-test-client -- --orbit --ptt-local│"
echo "└─────────────────────────────────────────────────────────────┘"
