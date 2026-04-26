#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0
# package-release.sh - Empaqueta el release completo

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$REPO_ROOT"

VERSION=$(cargo metadata --no-deps --format-version 1 | \
    python3 -c "import sys,json; pkgs=json.load(sys.stdin)['packages']; \
    print(next(p['version'] for p in pkgs if p['name']=='rmtfar-bridge'))")

DIST="dist/rmtfar-v$VERSION"
mkdir -p "$DIST"

echo "=== Empaquetando RMTFAR v$VERSION ==="

# Build release
RELEASE=1 bash scripts/build-all.sh
RELEASE=1 bash scripts/build-extension.sh
TARGET=windows RELEASE=1 bash scripts/build-plugin.sh
RELEASE=1 bash scripts/build-bridge-windows.sh
bash scripts/pack-pbo.sh

# Copiar binarios
cp target/x86_64-pc-windows-gnu/release/rmtfar_x64.dll \
    arma-mod/@rmtfar/rmtfar_x64.dll 2>/dev/null || true

cp -r arma-mod/@rmtfar "$DIST/"

# Bridge y plugin para Linux
mkdir -p "$DIST/bin"
cp target/release/rmtfar-bridge "$DIST/bin/" 2>/dev/null || true
cp target/release/librmtfar_plugin.so "$DIST/bin/" 2>/dev/null || true

# Bridge para Windows (ya copiado a @rmtfar por build-bridge-windows.sh)
echo "Bridge Windows incluido en $DIST/@rmtfar/rmtfar-bridge.exe"

cp LICENSE "$DIST/"
cp README.md "$DIST/"

# Crear zip
cd dist
zip -r "rmtfar-v$VERSION.zip" "rmtfar-v$VERSION/"
echo ""
echo "=== Release: dist/rmtfar-v$VERSION.zip ==="
ls -lh "rmtfar-v$VERSION.zip"
