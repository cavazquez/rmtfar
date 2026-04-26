#!/usr/bin/env bash
# Sets up RMTFAR plugin in Mumble automatically:
#   1. Compiles and installs the plugin binary
#   2. Registers it in mumble_settings.json so Mumble enables it on next launch
#
# Usage: ./setup-mumble.sh
# Requirements: Rust toolchain, Python 3, Mumble 1.5+
# IMPORTANT: run with Mumble CLOSED — it overwrites settings.json on exit.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")" && pwd)"
MUMBLE_CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}/Mumble/Mumble/mumble_settings.json"
PLUGIN_PATH="$HOME/.local/share/Mumble/Mumble/Plugins/librmtfar_plugin.so"

# ---------------------------------------------------------------------------
# 1. Build and install plugin binary
# ---------------------------------------------------------------------------
echo "==> Compilando e instalando plugin..."
"$REPO_ROOT/install-plugin.sh"

# ---------------------------------------------------------------------------
# 2. Sanity checks before touching mumble_settings.json
# ---------------------------------------------------------------------------
if [ ! -f "$MUMBLE_CONFIG" ]; then
    echo ""
    echo "ERROR: No se encontró la configuración de Mumble en:"
    echo "  $MUMBLE_CONFIG"
    echo ""
    echo "Abrí Mumble al menos una vez para que cree el archivo, luego re-ejecutá este script."
    exit 1
fi

if pgrep -x mumble > /dev/null 2>&1; then
    echo ""
    echo "ERROR: Mumble está corriendo."
    echo "Cerralo antes de ejecutar este script — Mumble sobreescribe mumble_settings.json al salir."
    exit 1
fi

# ---------------------------------------------------------------------------
# 3. Register plugin in mumble_settings.json
# ---------------------------------------------------------------------------
echo ""
echo "==> Registrando plugin en Mumble..."

python3 - "$MUMBLE_CONFIG" "$PLUGIN_PATH" <<'PYEOF'
import sys, json, hashlib, pathlib

config_path = pathlib.Path(sys.argv[1])
plugin_path = sys.argv[2]

# Mumble keys plugins by SHA1 of their absolute path.
key = hashlib.sha1(plugin_path.encode()).hexdigest()

data = json.loads(config_path.read_text())
plugins = data.setdefault("plugins", {})

# Search for any existing rmtfar entry (path may differ from a previous install).
old_key = next(
    (k for k, v in plugins.items() if "rmtfar" in v.get("path", "")),
    None,
)
if old_key and old_key != key:
    print(f"    Eliminando entrada anterior: {old_key[:12]}…")
    del plugins[old_key]

plugins[key] = {
    "enabled": True,
    "path": plugin_path,
    "positional_data_enabled": False,
}

config_path.write_text(json.dumps(data, indent=4))
print(f"    Clave:  {key}")
print(f"    Path:   {plugin_path}")
print(f"    Estado: enabled=true")
PYEOF

# ---------------------------------------------------------------------------
# Done
# ---------------------------------------------------------------------------
echo ""
echo "Setup completo. Abrí Mumble — el plugin RMTFAR ya estará habilitado."
echo ""
echo "Tip: para actualizar el plugin después de un cambio de código:"
echo "  ./install-plugin.sh   (solo actualiza el .so, no toca la config)"
echo "  ./setup-mumble.sh     (recompila + actualiza config si el path cambió)"
