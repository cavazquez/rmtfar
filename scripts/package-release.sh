#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0
# package-release.sh - Empaqueta el release completo localmente.
#
# Nota: en CI los releases se publican automáticamente en GitHub Actions
# al pushear un tag v*. Este script es para empaquetar manualmente.
#
# Requisitos:
#   sudo apt install mingw-w64 armake2
#   rustup target add x86_64-pc-windows-gnu
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

DIST="dist/rmtfar-v$VERSION"
mkdir -p "$DIST/@rmtfar/addons" "$DIST/bin"

echo "=== Empaquetando RMTFAR v$VERSION ==="

# ── Compilar binarios ────────────────────────────────────────────────────────

echo "--- Binarios Windows (extension + plugin + bridge) ---"
RELEASE=1 bash scripts/build-windows.sh

echo "--- Binarios Linux (plugin + bridge + test-client) ---"
bash scripts/build-linux.sh

echo "--- PBO ---"
bash scripts/pack-pbo.sh

# ── Copiar binarios ──────────────────────────────────────────────────────────

# Extension: Cargo genera rmtfar.dll; ya fue renombrada por build-extension.sh
cp arma-mod/@rmtfar/rmtfar_x64.dll     "$DIST/@rmtfar/"
cp arma-mod/@rmtfar/rmtfar_plugin.dll  "$DIST/@rmtfar/"
cp arma-mod/@rmtfar/addons/rmtfar.pbo  "$DIST/@rmtfar/addons/" 2>/dev/null || \
    cp -r arma-mod/@rmtfar/addons/rmtfar "$DIST/@rmtfar/addons/"

# Bridge Windows (build-bridge-windows.sh lo copia a @rmtfar/)
cp arma-mod/@rmtfar/rmtfar-bridge.exe  "$DIST/@rmtfar/" 2>/dev/null || true

# Linux
cp target/release/librmtfar_plugin.so  "$DIST/bin/" 2>/dev/null || true
cp target/release/rmtfar-bridge        "$DIST/bin/" 2>/dev/null || true
cp target/release/rmtfar-test-client   "$DIST/bin/" 2>/dev/null || true

cp LICENSE README.md "$DIST/"

# ── Crear zip ────────────────────────────────────────────────────────────────
mkdir -p dist
cd dist
zip -r "rmtfar-v$VERSION.zip" "rmtfar-v$VERSION/"
echo ""
echo "=== Release: dist/rmtfar-v$VERSION.zip ==="
ls -lh "rmtfar-v$VERSION.zip"
