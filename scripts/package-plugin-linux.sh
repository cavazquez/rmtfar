#!/usr/bin/env bash
# Empaqueta el plugin de Mumble como .mumble_plugin (ZIP con manifest.xml)
# listo para instalar desde Configuración → Complementos → "Instalar un plugin..."
#
# Requiere: cargo build --release -p rmtfar-plugin (o build-plugin-linux.sh primero)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(dirname "$SCRIPT_DIR")"
cd "$ROOT"

SO="target/release/librmtfar_plugin.so"
if [[ ! -f "$SO" ]]; then
    echo "==> .so no encontrado, compilando primero..."
    cargo build --release -p rmtfar-plugin
fi

TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

# ── manifest.xml según el esquema de Mumble 1.4.0+ ───────────────────────────
cat > "$TMPDIR/manifest.xml" <<'XML'
<?xml version="1.0" encoding="utf-8"?>
<bundle version="1">
  <!-- Linux x86_64 -->
  <plugin os="linux" arch="x86_64" path="librmtfar_plugin.so" />
</bundle>
XML

cp "$SO" "$TMPDIR/librmtfar_plugin.so"

OUT="target/release/rmtfar_plugin.mumble_plugin"
(cd "$TMPDIR" && zip -j "$OLDPWD/$OUT" manifest.xml librmtfar_plugin.so)

echo ""
echo "==> Paquete creado: $OUT ($(du -sh "$OUT" | cut -f1))"
echo ""
echo "┌──────────────────────────────────────────────────────────────────┐"
echo "│  Para instalarlo en Mumble:                                      │"
echo "│                                                                  │"
echo "│  1. Abrí Mumble → Configuración → Complementos                  │"
echo "│  2. Click en 'Instalar un plugin...'                             │"
echo "│  3. Seleccioná: $(realpath "$OUT")"
echo "│  4. Aceptá la instalación                                        │"
echo "│  5. Activá 'RMTFAR' en la lista de complementos                 │"
echo "└──────────────────────────────────────────────────────────────────┘"
