#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0
# build-plugin.sh — Compila el plugin de Mumble para Linux (.so) y/o Windows (.dll).
#
# Uso:
#   ./scripts/build-plugin.sh              # Linux debug
#   RELEASE=1 ./scripts/build-plugin.sh   # Linux release
#   TARGET=windows ./scripts/build-plugin.sh              # Windows debug
#   TARGET=windows RELEASE=1 ./scripts/build-plugin.sh   # Windows release

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

PROFILE="${RELEASE:+release}"
PROFILE="${PROFILE:-debug}"
CARGO_FLAGS="${RELEASE:+--release}"

case "${TARGET:-linux}" in
  windows)
    RUST_TARGET="x86_64-pc-windows-gnu"
    OUT_FILE="rmtfar_plugin.dll"
    DEST_DIR="arma-mod/@rmtfar"
    INSTALL_PATH="%APPDATA%\\Mumble\\Plugins\\rmtfar_plugin.dll"

    if ! command -v x86_64-w64-mingw32-gcc &>/dev/null; then
      echo "ERROR: x86_64-w64-mingw32-gcc no encontrado."
      echo "  Instalar con: sudo apt install mingw-w64"
      exit 1
    fi
    if ! rustup target list --installed | grep -q "$RUST_TARGET"; then
      echo "Instalando Rust target $RUST_TARGET..."
      rustup target add "$RUST_TARGET"
    fi

    echo "=== Build Plugin Mumble para Windows ($RUST_TARGET, $PROFILE) ==="
    cargo build -p rmtfar-plugin --target "$RUST_TARGET" ${CARGO_FLAGS:-}

    SRC="target/$RUST_TARGET/$PROFILE/$OUT_FILE"
    cp "$SRC" "$DEST_DIR/$OUT_FILE"
    echo ""
    echo "=== Plugin compilado ==="
    ls -lh "$SRC"
    echo ""
    echo "Instalado en: $DEST_DIR/$OUT_FILE"
    echo "Copiar en Windows a: $INSTALL_PATH"
    echo ""
    echo "Exports Mumble presentes:"
    x86_64-w64-mingw32-objdump -p "$SRC" | grep "mumble_" | awk '{print "  " $NF}'
    ;;

  linux|*)
    echo "=== Build Plugin Mumble para Linux ($PROFILE) ==="
    cargo build -p rmtfar-plugin ${CARGO_FLAGS:-}

    SRC="target/$PROFILE/librmtfar_plugin.so"
    echo ""
    echo "=== Plugin compilado ==="
    ls -lh "$SRC"
    echo ""
    echo "Para instalar en Mumble:"
    echo "  ./install-plugin.sh"
    ;;
esac
