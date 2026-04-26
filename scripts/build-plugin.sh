#!/usr/bin/env bash
# Cross-compile the Mumble plugin DLL for Windows x86_64.
# Requires `cross` (cargo install cross) and Docker.
#
# Output: target/x86_64-pc-windows-gnu/release/rmtfar_plugin.dll
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(dirname "$SCRIPT_DIR")"

cd "$ROOT"

TARGET="x86_64-pc-windows-gnu"

if command -v cross &>/dev/null; then
    echo "==> Cross-compiling Mumble plugin for $TARGET (via cross)..."
    cross build --release --target "$TARGET" -p rmtfar-plugin
else
    cargo build --release --target "$TARGET" -p rmtfar-plugin
fi

SRC="target/$TARGET/release/rmtfar_plugin.dll"
echo ""
if [[ -f "$SRC" ]]; then
    echo "==> Plugin built: $SRC"
    echo "    Copy to: %APPDATA%\\Mumble\\Plugins\\rmtfar_plugin.dll"
else
    echo "ERROR: $SRC not found."
    exit 1
fi
