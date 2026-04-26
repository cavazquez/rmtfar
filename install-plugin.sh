#!/usr/bin/env bash
# Compila e instala el plugin de Mumble en todos los paths conocidos.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")" && pwd)"
SO="$REPO_ROOT/target/release/librmtfar_plugin.so"

echo "==> Compilando rmtfar-plugin..."
cargo build --release -p rmtfar-plugin

echo "==> Instalando plugin..."

# Path real configurado en Mumble (detectado desde mumble_settings.json)
TARGET_MUMBLE="$HOME/.local/share/Mumble/Mumble/Plugins/librmtfar_plugin.so"
mkdir -p "$(dirname "$TARGET_MUMBLE")"
cp "$SO" "$TARGET_MUMBLE"
echo "    $TARGET_MUMBLE"

# Path alternativo (primer setup vía UI, target/release)
TARGET_RELEASE="$REPO_ROOT/target/release/rmtfar_plugin.mumble_plugin"
cp "$SO" "$TARGET_RELEASE"
echo "    $TARGET_RELEASE"

# Path estándar XDG (fallback)
TARGET_USER="$HOME/.local/share/mumble/Plugins/rmtfar.mumble_plugin"
mkdir -p "$(dirname "$TARGET_USER")"
cp "$SO" "$TARGET_USER"
echo "    $TARGET_USER"

echo ""
echo "Plugin instalado. Reiniciá Mumble para que tome los cambios."
