# Testing RMTFAR en Windows (Mumble + Arma 3)

Guía paso a paso para probar el sistema completo en Windows.

> **No se necesita bridge.** La extension DLL se comunica directamente con el
> plugin de Mumble, igual que TFAR con TeamSpeak.

## Requisitos

| Software | Versión | Link |
|---|---|---|
| Arma 3 | Steam | [Steam](https://store.steampowered.com/app/107410/Arma_3/) |
| CBA_A3 | Última | [Steam Workshop](https://steamcommunity.com/sharedfile/filedetails/?id=450814997) |
| Mumble | 1.5+ | [mumble.info](https://www.mumble.info/downloads/) |
| Servidor Murmur | Cualquier reciente | Puede ser local o remoto |

---

## Paso 0 — Obtener los binarios

### Opción A: Descargar de CI (recomendado)

Ir a [GitHub Actions](https://github.com/cavazquez/rmtfar/actions) y descargar los artefactos del último build exitoso:

- `rmtfar-extension-windows-x64` → contiene `rmtfar_x64.dll`
- `rmtfar-plugin-windows-x64` → contiene `rmtfar_plugin.dll`

### Opción B: Compilar desde Linux (cross-compile)

```bash
sudo apt install mingw-w64
rustup target add x86_64-pc-windows-gnu
cargo install armake2

./scripts/build-windows.sh --release
```

Los artefactos quedan en `dist/windows-x64/`:

- `arma3/@rmtfar/rmtfar_x64.dll`, `mod.cpp`, `addons/rmtfar.pbo`
- `mumble/rmtfar_plugin.dll`

---

## Paso 1 — Instalar el mod de Arma 3

Copiar la carpeta `@rmtfar` completa al directorio de Arma 3:

```
C:\Program Files (x86)\Steam\steamapps\common\Arma 3\@rmtfar\
├── addons\
│   └── rmtfar\
│       ├── config.cpp
│       ├── CfgFunctions.hpp
│       ├── XEH_preInit.sqf
│       ├── XEH_postInit.sqf
│       └── functions\
│           ├── fn_init.sqf
│           ├── fn_loop.sqf
│           ├── fn_getPlayerState.sqf
│           ├── fn_sendState.sqf
│           ├── fn_setFrequency.sqf
│           └── fn_radioTransmit.sqf
├── mod.cpp
└── rmtfar_x64.dll        ← la extension DLL
```

**Activar el mod:** En el launcher de Arma 3, ir a *Mods* → activar `@rmtfar` y `@CBA_A3`.

> **BattlEye:** La extension no está whitelistada todavía. Para testing, lanzar Arma 3 sin BattlEye o en un servidor local sin BE.

---

## Paso 2 — Instalar el plugin de Mumble

Copiar `rmtfar_plugin.dll` a la carpeta de plugins de Mumble:

```
%APPDATA%\Mumble\Plugins\rmtfar_plugin.dll
```

La ruta completa suele ser:
```
C:\Users\TuUsuario\AppData\Roaming\Mumble\Plugins\rmtfar_plugin.dll
```

> Si la carpeta `Plugins` no existe, crearla.

Reiniciar Mumble. Verificar en **Configure → Plugins** que aparezca **RMTFAR** habilitado.

### Configurar audio posicional

En Mumble: **Configure → Settings → Audio Output**
- Marcar **Positional Audio**
- Setear distancia mínima/máxima según preferencia

---

## Paso 3 — Conectar Mumble al servidor

1. Abrir Mumble
2. Conectarse al servidor Murmur (puede ser `localhost` si corrés Murmur localmente)
3. Verificar que el plugin RMTFAR está cargado (Configure → Plugins)

### Servidor Murmur local (para testing)

Si querés testear en local, podés instalar Murmur:
- Descargar de [mumble.info](https://www.mumble.info/downloads/)
- Ejecutar `murmur.exe` (viene con el instalador de Mumble)

---

## Paso 4 — Lanzar Arma 3

1. Abrir Arma 3 con los mods `@rmtfar` y `@CBA_A3` activados
2. **Sin BattlEye** para testing
3. Entrar a una misión (editor o servidor local)

> No hay que levantar ningún proceso extra. La extension DLL dentro de Arma
> se comunica directamente con el plugin de Mumble.

### Verificar que la extension carga

En la consola debug de Arma 3 (Esc → Debug Console):

```sqf
"rmtfar" callExtension "version";
```

Debería devolver la versión (`0.1.0`). Si devuelve vacío, la DLL no se cargó.

### Configurar teclas

En Arma 3: **Settings → Controls → Configure Addons → RMTFAR**

| Acción | Tecla default |
|---|---|
| PTT Local (proximidad) | Caps Lock |
| PTT Radio SR | T |

### Cambiar frecuencia

Desde la consola debug:

```sqf
["43.0", 1] call RMTFAR_fnc_setFrequency;
```

---

## Paso 5 — Testing con otro jugador

Necesitás al menos **dos personas** (o dos PCs) conectadas al mismo servidor de Arma 3 y al mismo Murmur.

### Test de proximidad
1. Ambos jugadores cerca (< 50m), sin PTT
2. Uno habla por Mumble → el otro debería escuchar con atenuación por distancia
3. Alejarse a > 50m → silencio

### Test de radio SR
1. Ambos en la misma frecuencia: `["43.0", 1] call RMTFAR_fnc_setFrequency;`
2. Uno presiona PTT Radio (T) y habla → se escucha con efecto DSP de radio
3. Cambiar la frecuencia de uno → silencio

### Test de radio LR
1. Configurar frecuencia LR en ambos
2. PTT LR → debería escucharse a distancias mayores (default 20 km de rango)

---

## Testing en Linux (sin Arma 3)

Para probar el pipeline de audio/radio sin Arma 3, usá el **bridge** +
**test-client** en Linux. Ver la sección correspondiente en el README principal.

---

## Troubleshooting

### "Extension not loaded" en Arma 3
- Verificar que `rmtfar_x64.dll` está en la raíz de `@rmtfar/` (no dentro de `addons/`)
- Verificar que Arma 3 se lanzó con el mod `@rmtfar` activo
- Verificar que no hay error de BattlEye bloqueando la DLL

### Plugin no aparece en Mumble
- Verificar que `rmtfar_plugin.dll` está en `%APPDATA%\Mumble\Plugins\`
- Reiniciar Mumble después de copiar
- Revisar la versión de Mumble (necesita 1.5+)

### No hay efecto de radio / audio posicional
- En Mumble: Configure → Settings → Audio Output → Positional Audio debe estar habilitado
- Verificar que hay al menos 2 jugadores en el servidor de Arma y de Mumble
- Revisar los logs de Arma 3 (`.rpt`) por errores de RMTFAR

### Firewall
- La extension usa solo `127.0.0.1` (localhost) para comunicarse con el plugin
- No necesita reglas de firewall
