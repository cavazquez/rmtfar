#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0
# build-all.sh - Compila todo el workspace para Linux (debug por defecto).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$REPO_ROOT"

RELEASE_FLAG="${RELEASE:+--release}"
PROFILE="${RELEASE:+release}"
PROFILE="${PROFILE:-debug}"

echo "=== RMTFAR Build All (Linux, $PROFILE) ==="
echo ""

echo "--- Compilando workspace ---"
cargo build $RELEASE_FLAG

echo ""
echo "--- Tests ---"
cargo test $RELEASE_FLAG

echo ""
echo "=== Build completado ==="
echo "Binarios en target/$PROFILE/:"
ls -lh \
    "target/$PROFILE/librmtfar_plugin.so" \
    "target/$PROFILE/rmtfar-bridge" \
    "target/$PROFILE/rmtfar-test-client" \
    2>/dev/null || true
