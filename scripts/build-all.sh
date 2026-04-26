#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0
# build-all.sh - Compila todo el workspace (Linux, debug por defecto)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$REPO_ROOT"

PROFILE="${RELEASE:+--release}"

echo "=== RMTFAR Build All ==="
echo "Profile: ${RELEASE:+release}${RELEASE:-debug}"
echo ""

echo "--- Compilando workspace ---"
cargo build $PROFILE

echo ""
echo "--- Tests ---"
cargo test $PROFILE

echo ""
echo "=== Build completado ==="
ls -lh target/${RELEASE:+release}${RELEASE:-debug}/lib/librmtfar_plugin.so \
        target/${RELEASE:+release}${RELEASE:-debug}/librmtfar_x64.so \
        target/${RELEASE:+release}${RELEASE:-debug}/rmtfar-bridge \
        target/${RELEASE:+release}${RELEASE:-debug}/rmtfar-test-client \
    2>/dev/null || true
