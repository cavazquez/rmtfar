#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0
# package-release.sh — Empaqueta el release completo en dist/ y genera el zip final.
#
# Nota: en CI los releases se publican automáticamente en GitHub Actions
# al pushear un tag v*. Este script es para empaquetar manualmente.
#
# Requisitos:
#   sudo apt install mingw-w64
#   rustup target add x86_64-pc-windows-gnu
#   cargo install armake2
#
# Uso:
#   ./scripts/package-release.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

VERSION=$(cargo metadata --no-deps --format-version 1 | \
    python3 -c "import sys,json; pkgs=json.load(sys.stdin)['packages']; \
    print(next(p['version'] for p in pkgs if p['name']=='rmtfar-protocol'))")

echo "=== Empaquetando RMTFAR v$VERSION ==="

# ── Compilar todo ────────────────────────────────────────────────────────────
echo ""
echo "--- Binarios Windows (Arma 3 + Mumble) ---"
bash "$SCRIPT_DIR/build-windows.sh" --release

echo ""
echo "--- Binarios Linux (Mumble) ---"
bash "$SCRIPT_DIR/build-linux.sh" --release

# ── Copiar archivos extra al paquete Arma 3 ───────────────────────────────────
cp LICENSE README.md "dist/windows-x64/arma3/@rmtfar/"

# ── Crear zip ────────────────────────────────────────────────────────────────
cd dist
ZIP="rmtfar-v$VERSION.zip"
rm -f "$ZIP"
zip -r "$ZIP" windows-x64/ linux/
echo ""
echo "=== Release: dist/$ZIP ==="
ls -lh "$ZIP"
