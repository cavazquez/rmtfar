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

# ── manifest.xml — formato v1.0.0 (Mumble 1.4.0+) ────────────────────────────
# Fuente: PluginManifest.cpp en mumble-voip/mumble v1.5.735
#   - <bundle version="1.0.0">  ← atributo "version", NO "format"
#   - os="linux"  arch="x86_64" ← verificado con strings(1) sobre el binario
#     (la doc oficial dice "x64" pero el Ubuntu build usa "x86_64")
cat > "$TMPDIR/manifest.xml" <<'XML'
<?xml version="1.0" encoding="UTF-8"?>
<bundle version="1.0.0">
  <assets>
    <plugin os="linux" arch="x86_64">librmtfar_plugin.so</plugin>
  </assets>
  <name>RMTFAR</name>
  <version>0.1.0</version>
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
