# 📻 RMTFAR — Radio Mumble Task Force Arma Radio

> Open-source TFAR-style radio mod for Arma 3, powered by Mumble/Murmur instead of TeamSpeak.

<div align="center">

[![CI](https://github.com/cavazquez/rmtfar/actions/workflows/ci.yml/badge.svg)](https://github.com/cavazquez/rmtfar/actions/workflows/ci.yml)
[![Coverage](https://codecov.io/gh/cavazquez/rmtfar/graph/badge.svg)](https://codecov.io/gh/cavazquez/rmtfar)
[![Rust](https://img.shields.io/badge/Rust-1.91+-f74c00?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE)
[![Mumble](https://img.shields.io/badge/Mumble-1.5%2B-darkgreen?logo=mumble&logoColor=white)](https://www.mumble.info/)
[![Arma 3](https://img.shields.io/badge/Arma_3-SQF-8B5C14)](https://store.steampowered.com/app/107410/Arma_3/)
[![GitHub Downloads](https://img.shields.io/github/downloads/cavazquez/rmtfar/total?label=descargas&logo=github)](https://github.com/cavazquez/rmtfar/releases)
[![GitHub Stars](https://img.shields.io/github/stars/cavazquez/rmtfar?style=flat&logo=github)](https://github.com/cavazquez/rmtfar/stargazers)

</div>

---

## 🛠️ Tech Stack

| Tecnología | Rol |
|---|---|
| 🦀 **Rust** | Extension DLL, bridge (testing) y plugin de Mumble |
| 🎮 **SQF** | Scripts dentro de Arma 3 |
| 🎙️ **Mumble 1.5+** | Transporte de voz (cliente, verificado en 1.5.735) |
| 🖥️ **Murmur** | Servidor de voz |
| 📦 **serde / serde_json** | Serialización del protocolo |
| 🔊 **DSP propio** | Biquad bandpass, AGC, bitcrusher y ruido de radio (implementado en `dsp.rs`) |
| 🧵 **UDP** | Comunicación local entre componentes |
| 🧠 **MumbleLink** | Shared memory para audio posicional |
| ⚙️ **C FFI** | Bindings al API de plugin de Mumble |
| 🔒 **GPLv3** | Licencia compatible con Mumble y ACRE2 |

---

## 🗺️ Arquitectura

```
┌──────────────────────────────────────────┐
│  🎮 Arma 3 Client                        │
│  ┌──────────────────────┐                │
│  │ SQF Scripts (@rmtfar)│                │
│  │  getPos, getDir, PTT, HUD │           │
│  │  allPlayers broadcast│                │
│  └──────────┬───────────┘                │
│             │ callExtension              │
│  ┌──────────▼───────────────────────┐    │
│  │ 🦀 Extension DLL (rmtfar_x64)    │    │
│  │  - Acumula estado de jugadores   │    │
│  │  - Escribe MumbleLink (shm)      │    │
│  │  - Envía RadioState → plugin     │    │
│  └──────┬──────────────┬────────────┘    │
└─────────┼──────────────┼────────────────┘
          │              │
          │ SharedMem    │ UDP :9501 (localhost)
          │ "MumbleLink" │
┌─────────▼──────────────▼────────────────────┐
│  🎙️ Mumble Client                            │
│  ┌────────────────────────────────────────┐  │
│  │ 🦀 RMTFAR Plugin (Rust + C FFI)        │  │
│  │  - Lee MumbleLink (audio posicional)   │  │
│  │  - Recibe radio state de la extension  │  │
│  │  - Audio callbacks: mute/unmute        │  │
│  │  - 🔊 DSP: biquad + AGC + bitcrusher   │  │
│  └────────────────────────────────────────┘  │
└──────────────────────────────────────────────┘
```

> **Sin bridge**: la extension DLL se comunica directamente con el plugin
> de Mumble (igual que TFAR con TeamSpeak). No hay proceso intermedio.

| Componente | Dónde vive | Lenguaje | Rol |
|---|---|---|---|
| `@rmtfar` (mod Arma) | Arma 3 | SQF + DLL Rust | Captura estado de todos los jugadores, escribe MumbleLink, envía radio state |
| Plugin Mumble | Cliente Mumble | 🦀 Rust + C FFI | Procesa audio por usuario |
| Bridge (testing) | Máquina local | 🦀 Rust | Solo para testing en Linux sin Arma 3 |

---

## 🚀 Estado de desarrollo

### ✅ Milestones verificados en Linux (Mumble 1.5.735)

| Funcionalidad | Estado | Notas |
|---|---|---|
| Pipeline end-to-end sin Arma 3 | ✅ | bridge + plugin + test-client |
| Plugin carga en Mumble 1.5+ | ✅ | API v1.0.x, `MUMBLE_FEATURE_AUDIO` (probado en 1.5.735) |
| Mapeo de identidad session → username | ✅ | lazy registration en talking callback |
| **Voz de proximidad** — atenuación por distancia | ✅ | rango 50m, lineal |
| **Filtro de frecuencia SR** — mute si no coincide | ✅ | string match exacto |
| **Filtro de canal** — mute si distinto canal en misma freq | ✅ | u8 match |
| **Rango de radio** — mute si dist > radio_range_m | ✅ | override con `--radio-range` |
| **DSP de radio** — biquad bandpass + AGC + bitcrusher + ruido | ✅ | audible, varía con distancia |
| **Interferencia por distancia** — crackle y dropouts progresivos | ✅ | `signal_quality` 1.0→0.0, dropout > 50% rango |
| **Muerte** — `alive=false` bloquea todo PTT | ✅ | log: `dead — muted` |
| **Inconsciente** — `conscious=false` bloquea PTT | ✅ | log: `unconscious — muted` |
| **Radio LR** — frecuencia, canal y rango independientes del SR | ✅ | rango default 20 km |
| **Vehículo** — PTT local bloqueado, radio sigue funcionando | ✅ | log: `in vehicle, no radio PTT — muted` |
| Mute correcto (zerear buffer, return true) | ✅ | fix API Mumble |
| **Tests de integración en CI** — 60 tests automatizados | ✅ | unit + integración bridge (subprocess UDP) |
| **Extension DLL para Arma 3** — cross-compile Windows x64 | ✅ | mingw-w64 (`x86_64-pc-windows-gnu`), ~200 KB release |
| **Plugin Mumble para Windows** — `rmtfar_plugin.dll` | ✅ | mingw-w64, 18 exports Mumble, ~800 KB release |
| **PBO packing** — scripts SQF empaquetados para Arma 3 | ✅ | `armake2`, con `$PBOPREFIX$` |
| **CBA keybinds** — PTT voz directa + radio SR; defaults solo si el perfil no tiene teclas | ✅ | `fn_cbaKeybindHasUserKeys`, `CBA_fnc_addKeybind` |
| **HUD in-game** — SR/LR, canales, indicadores de PTT | ✅ | `CfgRscTitles` + `fn_hudStart` (capa 23) |

### 🗺️ Fases de desarrollo

| Fase | Descripción | Estado |
|---|---|---|
| **1** | Voz por proximidad (posición 3D, atenuación por distancia) | ✅ |
| **2** | Radio simple (frecuencia, canal, rango, PTT, efecto DSP, muerte) | ✅ |
| **3** | Lógica tipo TFAR (SR/LR, potencia, interferencia, vehículos) | ✅ |
| **4** | Extension DLL + Plugin Mumble Windows (cross-compile mingw-w64 desde Linux) | ✅ |
| **5** | Testing en Windows con Arma 3 (PBO, CBA keybinds, mod loading) | 🚧 |

---

## 📨 Protocolo de mensajes

### 🎮 Arma 3 → Extension DLL (callExtension)

Formato delimitado (estable, sin JSON en SQF):

```text
v1|player_id|server_id|tick|x|y|z|dir|alive|conscious|vehicle|ptt_local|ptt_sr|ptt_lr|sr_freq|sr_ch|lr_freq|lr_ch|radio_los|sr_range_m|lr_range_m|sr_stereo|lr_stereo|sr_code|lr_code|intercom_enabled|intercom_channel|intercom_vehicle_id
```

- `radio_los`: factor 0–1 (1 = sin obstáculos entre el cliente local y ese jugador), con `lineIntersectsSurfaces` y caché.
- `sr_range_m` / `lr_range_m`: alcance en metros enviado desde Arma (según inventario y `CfgRMTFAR`). `0` o ausencia (payload de 18–19 campos) = usar el valor por defecto del protocolo en Rust.

**Modelos por ítem y facción** (`config.cpp` del addon): `CfgRMTFAR >> RadioItems` define `rangeSR` / `rangeLR` por **classname** de ítem (orden de búsqueda: asignados, armas, chaleco, uniforme, mochila, `items`). Si ningún ítem coincide, se usa la clase `Default`. `CfgRMTFAR >> RadioFactions >> <faction player>>` puede definir `rangeMult` (multiplicador positivo sobre ambos alcances).

Ejemplo (21 campos, LR sin frecuencia en el ejemplo):

```text
v1|Cristian|Servidor Test|123456|1234.5|567.8|12.3|145.0|1|1||0|1|0|43.0|1||1|1|5000|0
```

### 🦀 Extension → Plugin (UDP :9501)

```json
{
  "v": 1,
  "type": "radio_state",
  "server_id": "192.168.1.100:2302",
  "tick": 123456,
  "local_player": "Jugador2",
  "players": [
    {
      "player_id": "Jugador1",
      "pos": [200.0, 0.0, 0.0],
      "dir": 0.0,
      "alive": true,
      "conscious": true,
      "in_vehicle": false,
      "transmitting_local": false,
      "transmitting_radio": true,
      "radio_type": "sr",
      "radio_freq": "43.0",
      "radio_channel": 1,
      "radio_range_m": 500.0,
      "tuned_sr_freq": "43.0",
      "tuned_sr_channel": 1,
      "tuned_lr_freq": "",
      "tuned_lr_channel": 1,
      "radio_los_quality": 1.0
    }
  ]
}
```

---

## 📁 Estructura del repositorio

```
rmtfar/
├── 📄 Cargo.toml                  # Workspace de Rust (incl. rust-version 1.91)
├── 📄 rust-toolchain.toml         # Pin 1.91.0 + rustfmt + clippy (alineado con CI)
├── 📄 LICENSE                     # GPLv3
├── 📄 README.md
├── 📄 mod.cpp                     # Metadatos del mod (@rmtfar) — se copia al paquete de release
├── 🔍 check.sh                    # Quality gate: fmt + clippy + tests + SQF lint
├── 🔧 install-plugin.sh           # Compila e instala el plugin en todos los paths de Mumble
├── 📂 addon/                      # Fuentes SQF del addon (se empaquetan en rmtfar.pbo)
│   ├── $PBOPREFIX$
│   ├── config.cpp
│   ├── CfgRscTitles.hpp           # HUD in-game (RscTitles)
│   ├── functions/
│   └── …
├── 📂 .github/
│   └── workflows/
│       ├── ci.yml                 # CI: fmt + clippy + tests + build (GNU/mingw)
│       ├── release.yml            # Release: tag v* → build → GitHub Release
│       └── dep-audit.yml          # Auditoría anual de dependencias (diciembre)
├── 📦 crates/
│   ├── rmtfar-protocol/           # Tipos compartidos (PlayerState, RadioStateMessage…)
│   ├── rmtfar-extension/          # DLL para Arma 3 (cdylib, C ABI) — envía directo al plugin
│   ├── rmtfar-bridge/             # Proceso bridge (solo testing Linux sin Arma)
│   │   └── tests/integration.rs  # Tests de integración (bridge subprocess + UDP)
│   ├── rmtfar-plugin/             # Plugin de Mumble (cdylib, C FFI)
│   └── rmtfar-test-client/        # Simulador sin necesidad de Arma 3
├── 📂 dist/                       # Generado por los scripts (ignorado por git)
│   ├── windows-x64/
│   │   ├── arma3/@rmtfar/         # Listo para copiar al launcher de Arma 3
│   │   └── mumble/                # rmtfar_plugin.dll para Windows
│   └── linux/mumble/              # librmtfar_plugin.so para Linux
└── 🔧 scripts/
    ├── build-linux.sh             # Plugin .so (+ opcional --release → dist/ e instala Mumble)
    ├── build-windows.sh           # Cross-compile mingw-w64 (+ opcional --release → dist/)
    ├── pack-pbo.sh                # Empaqueta addon/ → dist/windows-x64/arma3/@rmtfar/addons/
    └── package-release.sh         # build-windows + build-linux en release + zip en dist/
```

---

## 📦 Dependencias

### 🦀 Rust

| Crate | Uso |
|---|---|
| `serde` + `serde_json` | Serialización del protocolo UDP |
| `anyhow` | Manejo de errores ergonómico (bridge, plugin) |
| `thiserror` | Tipos de error del protocolo |
| `libc` | Shared memory en Linux (bridge, extension) |
| `clap` | CLI del bridge y test-client |
| `tracing` | Logging estructurado |

### 🎮 Arma 3

| Mod | Requerido | Uso |
|---|---|---|
| [CBA_A3](https://github.com/CBATeam/CBA_A3) | Sí (`cba_main`) | Keybinds (PTT) |
| [ACE3](https://github.com/acemod/ACE3) | Opcional | Estado inconsciente: lee `ACE_isUnconscious` via `getVariable` (default `false` sin ACE3, sin errores) |

### 🎙️ Voz

| Software | Versión mínima | Versión verificada |
|---|---|---|
| [Mumble](https://www.mumble.info/) | 1.5+ | probado en **1.5.735** ✅ |
| [Murmur](https://www.mumble.info/documentation/mumble-server/) | Cualquier reciente | — |

---

## 🪟 Cross-compilación para Windows (mingw-w64 desde Linux)

Todos los binarios de Windows se cross-compilan desde Linux usando
`x86_64-pc-windows-gnu` (mingw-w64). Es el mismo toolchain que usa el CI.

### Requisitos (una sola vez)

```bash
sudo apt install mingw-w64
rustup target add x86_64-pc-windows-gnu
cargo install armake2
```

### Compilar

```bash
# Debug: binarios solo en target/ (ignorado por git)
./scripts/build-linux.sh
./scripts/build-windows.sh

# Release: además genera dist/ listo para copiar
./scripts/build-linux.sh --release    # dist/linux/mumble/librmtfar_plugin.so (+ instala en Mumble local)
./scripts/build-windows.sh --release  # dist/windows-x64/arma3/@rmtfar/ + dist/windows-x64/mumble/

# PBO (normalmente ya incluido en build-windows.sh --release; manual si hace falta)
./scripts/pack-pbo.sh

# Zip completo (Windows + Linux) en dist/rmtfar-v*.zip
./scripts/package-release.sh
```

En **Linux**, Arma 3 corre vía Proton/Steam Play y sigue usando la **extension DLL de Windows**; no hay binario nativo de la extensión para Linux. Para el mod en el juego, usá `dist/windows-x64/arma3/@rmtfar/` generado por `build-windows.sh --release`.

### Archivos generados (`--release`)

| Archivo | Tamaño aprox. | Dónde queda | Destino en el sistema |
|---|---|---|---|
| `rmtfar_x64.dll` | ~200 KB | `dist/windows-x64/arma3/@rmtfar/` | Arma 3 lo carga vía `callExtension` |
| `mod.cpp` | — | `dist/windows-x64/arma3/@rmtfar/` | Metadatos del mod en el launcher |
| `rmtfar.pbo` | ~17 KB | `dist/windows-x64/arma3/@rmtfar/addons/` | Scripts SQF empaquetados |
| `rmtfar_plugin.dll` | ~800 KB | `dist/windows-x64/mumble/` | `%APPDATA%\Mumble\Plugins\` |
| `librmtfar_plugin.so` | — | `dist/linux/mumble/` | Plugins de Mumble en Linux |

### Instalar en Windows

1. Copiar la carpeta `dist/windows-x64/arma3/@rmtfar/` (o el contenido equivalente del zip de release) a una ubicación accesible (ej. `C:\Users\...\Documents\Arma 3 - Other Profiles\...\mods\@rmtfar\`)
2. Copiar `dist/windows-x64/mumble/rmtfar_plugin.dll` a `%APPDATA%\Mumble\Plugins\`
3. En el **Launcher de Arma 3** → **MODS** → **Add local mod** → seleccionar la carpeta `@rmtfar`
4. **Desactivar BattlEye** en el Launcher (la DLL no está en la whitelist de BE y será bloqueada)
5. Mumble detecta el plugin automáticamente al iniciar. Verificar en *Mumble → Configuración → Plugins*

> ⚠️ **BattlEye**: Si BattlEye está habilitado, bloqueará la carga de `rmtfar_x64.dll`
> con error "Recursos insuficientes en el sistema" (error 1450). Esto ocurre porque
> BE intercepta `LoadLibrary` y rechaza DLLs no registradas. Para desarrollo y testing,
> lanzar Arma 3 sin BattlEye. Para servidores con BE, la DLL necesitaría ser whitelisteada.

### Verificar exports

```bash
# Extension (3 exports para Arma 3) — tras build-windows.sh --release
x86_64-w64-mingw32-objdump -p dist/windows-x64/arma3/@rmtfar/rmtfar_x64.dll | grep -A5 "Ordinal/Name"
# RVExtension  RVExtensionArgs  RVExtensionVersion

# Plugin (18 exports para Mumble)
x86_64-w64-mingw32-objdump -p dist/windows-x64/mumble/rmtfar_plugin.dll | grep mumble_
```

### Uso desde SQF

```sqf
// Verificar versión de la extension
private _ver = "rmtfar" callExtension "version";
systemChat format ["RMTFAR version: %1", _ver];

// Enviar estado del jugador a la extension (payload v1 delimitado)
private _result = "rmtfar" callExtension ["send", [_payloadV1]];
```

> **CI y releases**: los binarios de Windows se compilan con `x86_64-pc-windows-gnu` (mingw-w64). Las DLLs funcionan correctamente en Arma 3 **con BattlEye desactivado** (ver advertencia abajo).

---

## 🪖 Estructura del mod de Arma 3

En el repositorio, las fuentes del addon están en **`addon/`** (SQF + `config.cpp`); al hacer release se empaquetan en **`addons/rmtfar.pbo`**. El **`mod.cpp`** vive en la raíz del repo y se copia junto al DLL al armar `dist/`.

**Distribución** (`dist/windows-x64/arma3/@rmtfar/` tras `./scripts/build-windows.sh --release`):

```
@rmtfar/
├── mod.cpp                     # Metadatos del mod (launcher)
├── rmtfar_x64.dll              # Extension DLL (Rust, mingw-w64)
└── addons/
    └── rmtfar.pbo              # Scripts SQF empaquetados (desde addon/)
```

**Fuentes en el repo** (`addon/`):

```
addon/
├── $PBOPREFIX$                 # Prefijo interno: rmtfar\addons\rmtfar
├── config.cpp                  # CfgPatches + CBA XEH + CfgRMTFAR + include CfgRscTitles
├── CfgRscTitles.hpp            # Recurso RMTFAR_RadioHud (HUD de radio en pantalla)
├── XEH_preInit.sqf             # Variables globales + flags (p.ej. RMTFAR_showRadioHud)
├── XEH_postInit.sqf            # Extension + HUD + loop principal
└── functions/
    ├── fn_init.sqf             # CBA keybinds (PTT voz directa, radio SR y radio LR)
    ├── fn_cbaKeybindHasUserKeys.sqf  # ¿Hay teclas reales en el perfil de CBA? (no pisar defaults)
    ├── fn_hudStart.sqf         # HUD RscTitles: frecuencias y estado PTT
    ├── fn_loop.sqf             # Loop: recolecta estado, broadcast, envía a extension
    ├── fn_getPlayerState.sqf   # Lee pos/dir/alive/radio de un jugador
    └── fn_sendState.sqf        # Serializa payload v1 y llama callExtension
```

El **plugin de Mumble para Windows** (`rmtfar_plugin.dll`) no va dentro de `@rmtfar/` en el juego: copiarlo desde **`dist/windows-x64/mumble/`** a `%APPDATA%\Mumble\Plugins\`.

### Keybinds (CBA)

Se registran acciones en *Configuración → Controles → Configurar addons → RMTFAR*:

| Acción | Variable SQF | Comportamiento del default |
|---|---|---|
| **PTT - Voz directa** | `RMTFAR_pttLocal` | Si en el perfil de CBA **no** hay ninguna tecla “real” para esa acción, el mod aplica **sin tecla** (podés asignar una después). Si ya configuraste teclas, **no se tocan**. |
| **PTT - Radio (corto alcance)** | `RMTFAR_pttRadioSR` | Default **sin tecla** para evitar duplicar PTT cuando se usa radio activa. Si ya hay teclas, **no se tocan**. |
| **PTT - Radio (largo alcance)** | `RMTFAR_pttRadioLR` | Default **sin tecla** para no pisar perfiles existentes. Si ya hay teclas, **no se tocan**. |
| **PTT - Radio activa (SR/LR)** | `RMTFAR_pttRadioSR` / `RMTFAR_pttRadioLR` | Transmite con una sola tecla sobre la radio seleccionada en `RMTFAR_activeRadio` (default SR). En instalaciones nuevas, default **Bloq Mayús (Caps Lock)**. |
| **PTT - Radio adicional** | `RMTFAR_pttRadioSR` / `RMTFAR_pttRadioLR` | Transmite por la radio opuesta a `RMTFAR_activeRadio`. Default sin tecla. |
| **Alternar radio activa (SR/LR)** | `RMTFAR_activeRadio` | Cambia la radio activa entre SR y LR. Default sin tecla. |
| **Canal rápido 1..9 (radio activa)** | `RMTFAR_radioChannel` / `RMTFAR_radioChannelLR` | Cambia canal de la radio activa. Default NumPad 1..9 (editable en CBA). |
| **Canal siguiente/anterior (radio activa)** | `RMTFAR_radioChannel` / `RMTFAR_radioChannelLR` | Rota canal de la radio activa (loop 1..9). Default Shift+PageUp / Shift+PageDown. |
| **Estereo Both/Left/Right (radio activa)** | `RMTFAR_radioStereo` / `RMTFAR_radioStereoLR` | Ajusta el paneo de recepción de la radio activa. Default Shift+Flechas (Up/Left/Right). |
| **Estereo siguiente/anterior (radio activa)** | `RMTFAR_radioStereo` / `RMTFAR_radioStereoLR` | Rota paneo de la radio activa (loop B/L/R). Default sin tecla (asignable en CBA). |
| **Intercom ON/OFF** | `RMTFAR_intercomEnabled` | Habilita/deshabilita intercom local cuando estás en vehículo. Default sin tecla. |
| **Intercom canal siguiente/anterior** | `RMTFAR_intercomChannel` | Rota canal intercom (1..3). Default Alt+PageUp / Alt+PageDown. |
| **Intercom debug HUD ON/OFF** | `RMTFAR_showIntercomDebug` | Muestra/oculta `IC-Veh <netId>` en HUD. Default sin tecla (persistente). |

La comprobación usa el registro `cba_keybinding_registry_v3` del perfil (misma regla que CBA: tecla con código `> 1`). Ver `fn_cbaKeybindHasUserKeys.sqf`.

Configuración por defecto recomendada del mod:
- **Caps Lock**: `PTT - Radio activa (SR/LR)`
- **Sin tecla**: `PTT - Radio (corto alcance)` y `PTT - Radio (largo alcance)` (opcionales para quien prefiera PTT separadas)

También hay funciones de misión/script para forzar TX:
- `RMTFAR_fnc_radioTransmit` (SR)
- `RMTFAR_fnc_radioTransmitLR` (LR)
- `RMTFAR_fnc_radioTransmitActive` (sobre SR/LR activa)
- `RMTFAR_fnc_radioTransmitAdditional` (sobre radio opuesta a la activa)

Y para alternar la radio activa:
- `RMTFAR_fnc_toggleActiveRadio`

Y para canal rápido sobre radio activa:
- `RMTFAR_fnc_setChannelActive`
- `RMTFAR_fnc_cycleChannelActive`
- `RMTFAR_fnc_setStereoActive`
- `RMTFAR_fnc_cycleStereoActive`
- `RMTFAR_fnc_setRadioCodeActive`
- `RMTFAR_fnc_toggleIntercom`
- `RMTFAR_fnc_cycleIntercomChannel`

Y funciones para sintonizar frecuencias:
- `RMTFAR_fnc_setFrequency` (SR)
- `RMTFAR_fnc_setFrequencyLR` (LR)

Persistencia por perfil (`profileNamespace`):
- `RMTFAR_activeRadio`
- `RMTFAR_radioStereo` / `RMTFAR_radioStereoLR`
- `RMTFAR_intercomEnabled` / `RMTFAR_intercomChannel`
- `RMTFAR_showIntercomDebug`

### HUD en pantalla

Con la extensión cargada y `RMTFAR_enabled`, el mod muestra un panel discreto (abajo a la derecha) con **SR** (frecuencia y canal), **LR** (si está sintonizada muestra frecuencia/canal; si no, indica “LR sin sintonizar”), radio **Activa (SR/LR)**, e indicadores de **TX** (radio SR/LR y voz directa). No sustituye ni integra la UI de **TFAR**; si cargás ambos mods podés tener overlays distintos.

Al iniciar, el mod también muestra mensajes breves en `systemChat` con:
- radio activa inicial (`SR`/`LR`) y su estéreo (`B`/`L`/`R`);
- recordatorio de bind para `PTT - Radio activa (SR/LR)` si falta;
- recordatorio opcional para `Alternar radio activa (SR/LR)` si falta.

Para desactivar estos mensajes de inicio:

```sqf
missionNamespace setVariable ["RMTFAR_showStartupHints", false];
```

Ejemplo en misión (`initPlayerLocal.sqf`):

```sqf
// Oculta hints de arranque de RMTFAR para este jugador.
missionNamespace setVariable ["RMTFAR_showStartupHints", false];
```

Ejemplo para forzarlos en una misión de entrenamiento:

```sqf
// Fuerza hints de arranque de RMTFAR para este jugador.
missionNamespace setVariable ["RMTFAR_showStartupHints", true];
```

### Test rápido de SR/LR en Arma 3

1. Abrir **Controles → Configurar addons → RMTFAR** y verificar que exista `PTT - Radio (Largo Alcance)`.
2. Asignar una tecla para LR (por default viene sin tecla), y entrar a misión con el mod cargado.
3. Sintonizar LR por consola/script:

```sqf
["30.000", 1] call RMTFAR_fnc_setFrequencyLR;
```

4. Presionar la tecla de LR y confirmar en HUD el indicador `TX radio LR`.
5. Validar en RPT que cambie la frecuencia LR (`RMTFAR: LR frequency changed...`) y que el estado se siga enviando al plugin.

### Test rápido de radio activa (una sola PTT)

1. Asignar teclas a `PTT - Radio activa (SR/LR)` y `Alternar radio activa (SR/LR)`.
2. Entrar a misión y confirmar en HUD la línea `Activa: SR`.
3. Presionar la tecla de alternar y verificar que cambie a `Activa: LR`.
4. Mantener `PTT - Radio activa (SR/LR)` y confirmar que se encienda `TX radio LR` (y no SR).
5. Alternar de nuevo a SR y repetir: ahora debe encender `TX radio SR` (y no LR).
6. Reingresar a misión y verificar los `systemChat` de inicio (radio activa y hints de bind).
7. (Opcional) Desactivar hints con `RMTFAR_showStartupHints = false` y reingresar para confirmar que no aparecen.

### Test rápido de PTT adicional + canales rápidos

1. Asignar tecla a `PTT - Radio adicional` (si querés usarla) y verificar que existan `Canal 1..9 (radio activa)`.
2. Con `Activa: SR`, mantener `PTT - Radio adicional`: debe encender `TX radio LR`.
3. Alternar a `Activa: LR` y repetir: ahora debe encender `TX radio SR`.
4. Presionar `Canal 3 (radio activa)` y verificar que cambie el canal de la activa (SR o LR según corresponda).
5. Probar `Canal siguiente/anterior (radio activa)` y validar ciclo con loop 1..9.
6. Validar en RPT logs de cambio de canal activo (`active SR/LR channel changed to ...`).

### Test rápido de estéreo SR/LR

1. Con dos instancias de Mumble, dejar un emisor transmitiendo por SR constante.
2. En el oyente, con radio activa en SR, usar `Estereo Left` y verificar audio solo por canal izquierdo.
3. Repetir con `Estereo Right` y luego `Estereo Both`.
4. Alternar radio activa a LR y repetir la prueba con transmisión LR.
5. Verificar en HUD que SR/LR muestran `B`, `L` o `R` según el modo elegido.
6. (Opcional) Asignar `Estereo siguiente/anterior (radio activa)` y validar rotación B→L→R→B.

### Test rápido de código de radio (encryption lógico)

1. En dos jugadores, dejar misma frecuencia/canal SR.
2. En jugador A: `["ALFA"] call RMTFAR_fnc_setRadioCodeActive;` (con activa en SR).
3. En jugador B: `["BRAVO"] call RMTFAR_fnc_setRadioCodeActive;` (SR).
4. Transmitir A→B: debe quedar mute por `radio code mismatch`.
5. Poner mismo código en ambos (`ALFA`) y repetir: debe volver a oírse.

### Test rápido de intercom MVP

1. Entrar con dos jugadores dentro de vehículos y activar `PTT - Voz directa`.
2. Con `Intercom ON`, canal igual (p. ej. C1) y dentro del **mismo vehículo** (mismo `intercom_vehicle_id`), se deben oír por intercom.
3. Cambiar canal en uno (C2): debe mutear.
4. Volver a canal igual: debe volver audio.
5. Probar `Intercom OFF` en uno: debe mutear.
6. (Opcional) Activar debug y verificar en HUD línea `IC-Veh <netId>` cuando estás en vehículo.

> Nota MVP actual: ya distingue por vehículo específico vía `intercom_vehicle_id` (netId del vehículo).

Para mostrar debug de intercom en HUD:

```sqf
missionNamespace setVariable ["RMTFAR_showIntercomDebug", true];
```

Default: `false` (oculto en sesiones normales).

### Test rápido de persistencia por perfil

1. Cambiar radio activa a `LR`, estéreo a `Right`, intercom a `OFF` y canal `3`.
2. (Opcional) Activar `Intercom debug HUD`.
3. Salir de misión/Arma y volver a entrar con el mod.
4. Verificar que los estados anteriores se carguen automáticamente.

Para **ocultar** el panel:

```sqf
missionNamespace setVariable ["RMTFAR_showRadioHud", false];
```

Para volver a mostrarlo: `true` (valor por defecto en `XEH_preInit.sqf`).

---

## 🧩 Trabajo realizado en esta sesión

Esta sesión extendió `rmtfar` para acercarlo al flujo de TFAR clásico, manteniendo compatibilidad con lo ya implementado.

### Qué se implementó

- **Radio LR operable en gameplay**
  - Keybind CBA `PTT - Radio (Largo Alcance)`.
  - Funciones de script:
    - `RMTFAR_fnc_radioTransmitLR`
    - `RMTFAR_fnc_setFrequencyLR`
  - HUD actualizado para mostrar estado LR incluso sin sintonía (`LR sin sintonizar`).

- **Radio activa SR/LR + PTT unificada**
  - Estado `RMTFAR_activeRadio` (`SR`/`LR`).
  - Keybinds:
    - `PTT - Radio activa (SR/LR)`
    - `Alternar radio activa (SR/LR)`
  - Funciones:
    - `RMTFAR_fnc_radioTransmitActive`
    - `RMTFAR_fnc_toggleActiveRadio`
  - Default “compat”:
    - Caps Lock en `PTT - Radio activa`
    - SR/LR dedicadas sin tecla por defecto.

- **PTT adicional + gestión avanzada de canales**
  - Keybind `PTT - Radio adicional` (transmite por la radio opuesta a la activa).
  - Canales rápidos de la radio activa:
    - `Canal 1..9`
    - `Canal siguiente/anterior` (loop 1..9)
  - Funciones:
    - `RMTFAR_fnc_radioTransmitAdditional`
    - `RMTFAR_fnc_setChannelActive`
    - `RMTFAR_fnc_cycleChannelActive`

- **Estéreo SR/LR (Both/Left/Right) end-to-end**
  - Estado estéreo por radio:
    - `RMTFAR_radioStereo` (SR)
    - `RMTFAR_radioStereoLR` (LR)
  - Keybinds:
    - `Estereo Both/Left/Right (radio activa)`
    - `Estereo siguiente/anterior (radio activa)`
  - Funciones:
    - `RMTFAR_fnc_setStereoActive`
    - `RMTFAR_fnc_cycleStereoActive`
  - Protocolo/extensión/plugin:
    - Se agregaron campos `sr_stereo` / `lr_stereo`.
    - El plugin aplica paneo en audio según estéreo local del tipo de radio activo.

- **Código de radio (encryption lógico)**
  - Estado por radio:
    - `RMTFAR_radioCode`
    - `RMTFAR_radioCodeLR`
  - Función:
    - `RMTFAR_fnc_setRadioCodeActive`
  - Protocolo/extensión/plugin:
    - Se agregaron `sr_code` / `lr_code`.
    - El plugin mutea por `radio code mismatch` si no coincide el código.

- **Intercom MVP**
  - Estado local:
    - `RMTFAR_intercomEnabled`
    - `RMTFAR_intercomChannel` (1..3)
  - Keybinds y funciones:
    - `Intercom ON/OFF` → `RMTFAR_fnc_toggleIntercom`
    - `Intercom canal siguiente/anterior` → `RMTFAR_fnc_cycleIntercomChannel`
  - Protocolo/extensión/plugin:
    - Campos `intercom_enabled`, `intercom_channel`.
    - Lógica de intercom en plugin usando PTT local en vehículo.

- **Intercom por vehículo específico**
  - Se agregó `intercom_vehicle_id` (netId de vehículo).
  - El plugin ahora exige coincidencia de `intercom_vehicle_id` para abrir audio intercom.

- **HUD y UX**
  - Indicadores ampliados: SR/LR + estéreo, radio activa, estado de intercom.
  - Debug opcional en HUD:
    - `IC-Veh <netId>` controlado por `RMTFAR_showIntercomDebug`.

- **Hints de inicio configurables**
  - Mensajes de arranque (radio activa, sugerencias de keybind) protegidos por:
    - `RMTFAR_showStartupHints` (`true/false`).

- **Persistencia por perfil (`profileNamespace`)**
  - Guardado/carga automática de:
    - `RMTFAR_activeRadio`
    - `RMTFAR_radioStereo` / `RMTFAR_radioStereoLR`
    - `RMTFAR_intercomEnabled` / `RMTFAR_intercomChannel`
    - `RMTFAR_showIntercomDebug`
  - Funciones:
    - `RMTFAR_fnc_saveProfileSettings`
    - `RMTFAR_fnc_loadProfileSettings`
    - `RMTFAR_fnc_toggleIntercomDebug`

### Cómo testear (plan sugerido)

1. **Inicialización y keybinds**
   - Abrir `Controles -> Configurar addons -> RMTFAR`.
   - Verificar presencia de acciones SR/LR/activa/adicional, canales, estéreo, intercom.

2. **Radio activa + PTT**
   - Alternar `SR/LR` y validar en HUD `Activa: ...`.
   - Probar `PTT activa`: debe encender TX del tipo activo.
   - Probar `PTT adicional`: debe encender TX del tipo opuesto.

3. **Canales**
   - Probar `Canal 1..9` y `siguiente/anterior`.
   - Validar loop 1..9 y logs en RPT (`active SR/LR channel changed ...`).

4. **Estéreo**
   - Con dos clientes, emisor en radio continua.
   - En oyente, probar Both/Left/Right sobre SR y LR.
   - Validar paneo audible + indicador `B/L/R` en HUD.

5. **Código de radio**
   - Misma freq/canal con códigos distintos (`ALFA` vs `BRAVO`) -> mute.
   - Igualar códigos -> audio vuelve.

6. **Intercom**
   - Dos jugadores dentro del mismo vehículo.
   - Intercom ON + mismo canal -> se oyen por intercom.
   - Cambiar canal o apagar intercom -> mute.
   - Vehículos distintos (netId distinto) -> no debe abrir intercom.

7. **Debug intercom HUD**
   - Activar:
     - `missionNamespace setVariable ["RMTFAR_showIntercomDebug", true];`
   - Verificar línea `IC-Veh <netId>` en vehículo.

8. **Persistencia**
   - Cambiar radio activa/estéreo/intercom/debug.
   - Reingresar a misión.
   - Verificar que el perfil mantenga esos valores.

9. **Regresión básica**
   - Revisar que voz local en pie siga por proximidad.
   - Revisar que en vehículo sin radio no se abra local normal.
   - Revisar que SR/LR mantengan filtros por frecuencia/canal/rango/LOS.

### Checklist QA manual (por release)

Copiá esta lista en el issue/release y marcá cada item:

- [ ] **Inicialización**: mod cargado, extensión detectada, sin errores críticos en RPT.
- [ ] **Keybinds CBA**: acciones SR/LR/activa/adicional/canales/estéreo/intercom visibles y configurables.
- [ ] **PTT activa/adicional**: TX en tipo correcto (activa u opuesta) según HUD.
- [ ] **Canales**: cambios 1..9 + ciclo next/prev funcionando con loop correcto.
- [ ] **Estéreo**: Both/Left/Right y ciclo aplican paneo audible en SR y LR.
- [ ] **Códigos radio**: mismatch mutea, match restablece audio.
- [ ] **Intercom (mismo vehículo)**: ON + mismo canal permite audio con PTT local.
- [ ] **Intercom (vehículo distinto)**: con netId distinto no abre intercom.
- [ ] **Intercom OFF / canal distinto**: mutear correctamente.
- [ ] **HUD normal**: muestra SR/LR, activa, TX, intercom; sin ruido visual extra.
- [ ] **HUD debug intercom**: `RMTFAR_showIntercomDebug=true` muestra `IC-Veh <netId>`.
- [ ] **Persistencia perfil**: radio activa, estéreo, intercom y debug sobreviven reingreso.
- [ ] **Compatibilidad**: payloads previos siguen parseando (sin romper sesiones existentes).
- [ ] **Regresión de proximidad**: voz local en pie conserva atenuación por distancia.
- [ ] **Regresión en vehículo**: local normal bloqueado; radio/intercom según reglas.
- [ ] **Build**: `cargo check`/build release sin errores.

---

## 🪟 Prueba en Windows (Arma + Mumble, sin test-client Linux)

Para validar **SQF → `rmtfar_x64.dll` → UDP :9501 → plugin Mumble** con un “jugador remoto” sintético, usá el modo DEBUG del mod (ghosts) y seguí [docs/windows-ghost-test.md](docs/windows-ghost-test.md).

---

## 🐧 Cómo probar en Linux (sin Arma 3)

> **Nota:** El bridge solo se usa para testing en Linux donde no hay Arma 3.
> En producción (Windows + Arma 3), la extension DLL se comunica directamente
> con el plugin de Mumble — no se necesita bridge.

Guía paso a paso para verificar el sistema completo en Linux con dos instancias de Mumble.

### Requisitos

```bash
sudo apt install mumble murmur    # Mumble 1.5.735 en Ubuntu 24.04+
sudo systemctl start mumbled      # Arrancar el servidor local
```

### Paso 1 — Compilar e instalar el plugin

```bash
./install-plugin.sh
```

El script compila el plugin y lo copia a todos los paths que Mumble puede usar:
- `~/.local/share/Mumble/Mumble/Plugins/librmtfar_plugin.so` (path real, detectado desde `mumble_settings.json`)
- `~/.local/share/mumble/Plugins/rmtfar.mumble_plugin` (path estándar XDG)

> **Nota:** después de instalar, cerrá y reabrí Mumble para que tome el nuevo binario.

### Paso 2 — Abrir dos instancias de Mumble y conectarlas al servidor local

```bash
mumble &             # Instancia A — Jugador2 (tiene el plugin activo)
mumble --multiple &  # Instancia B — Jugador1
```

Conectá ambas a `localhost`. El nombre de usuario de cada instancia debe coincidir con el `--id` del test-client.

### Paso 3 — Arrancar el bridge (solo testing Linux)

```bash
cargo run --release -p rmtfar-bridge -- --local-id "Jugador2"
```

`--local-id` fija qué jugador es el oyente local (el que tiene Mumble con el plugin).

> En producción con Arma 3 en Windows, este paso no es necesario — la
> extension DLL reemplaza al bridge.

### Paso 4 — Simular estado de jugadores

#### 🔊 Prueba de frecuencia de radio

```bash
# Jugador2 sintonizado en 43.0 (sin PTT — solo escucha)
cargo run --release -p rmtfar-test-client -- --id "Jugador2" --freq 43.0

# Jugador1 transmite en la misma frecuencia → se escucha con efecto radio
cargo run --release -p rmtfar-test-client -- \
  --id "Jugador1" --freq 43.0 --ptt-radio --pos 200,0,0 --radio-range 500

# Jugador1 en frecuencia diferente → silencio
cargo run --release -p rmtfar-test-client -- \
  --id "Jugador1" --freq 50.0 --ptt-radio --pos 200,0,0 --radio-range 500
```

#### 📡 Prueba de rango de radio

```bash
# Dentro del rango (200m < 500m) → DSP aplicado, audible
cargo run --release -p rmtfar-test-client -- \
  --id "Jugador1" --freq 43.0 --ptt-radio --pos 200,0,0 --radio-range 500

# Fuera del rango (800m > 500m) → silencio
cargo run --release -p rmtfar-test-client -- \
  --id "Jugador1" --freq 43.0 --ptt-radio --pos 800,0,0 --radio-range 500
```

#### 🔢 Prueba de canal

```bash
# Jugador2 en canal 1, Jugador1 en canal 2 → silencio (misma freq, distinto canal)
cargo run --release -p rmtfar-test-client -- --id "Jugador2" --freq 43.0 --channel 1
cargo run --release -p rmtfar-test-client -- \
  --id "Jugador1" --freq 43.0 --channel 2 --ptt-radio --pos 200,0,0 --radio-range 500

# Ambos en canal 1 → audible
cargo run --release -p rmtfar-test-client -- \
  --id "Jugador1" --freq 43.0 --channel 1 --ptt-radio --pos 200,0,0 --radio-range 500
```

#### ☠️ Prueba de muerte / inconsciente

```bash
# Jugador1 muerto → PTT bloqueado, log: "dead — muted"
cargo run --release -p rmtfar-test-client -- \
  --id "Jugador1" --freq 43.0 --ptt-radio --pos 200,0,0 --radio-range 500 --dead

# Jugador1 inconsciente (ACE) → PTT bloqueado, log: "unconscious — muted"
cargo run --release -p rmtfar-test-client -- \
  --id "Jugador1" --freq 43.0 --ptt-radio --pos 200,0,0 --radio-range 500 --unconscious
```

#### 🚗 Prueba de vehículo

```bash
# Jugador1 en vehículo + PTT local → bloqueado, log: "in vehicle, no radio PTT — muted"
cargo run --release -p rmtfar-test-client -- \
  --id "Jugador1" --ptt-local --vehicle B_MRAP_01_F

# Jugador1 en vehículo + PTT radio → audible con DSP
cargo run --release -p rmtfar-test-client -- \
  --id "Jugador1" --freq 43.0 --ptt-radio --pos 200,0,0 --vehicle B_MRAP_01_F --radio-range 500
```

#### 📻 Prueba de radio LR (largo rango)

```bash
# Jugador2 escucha LR en 30.0
cargo run --release -p rmtfar-test-client -- --id "Jugador2" --freq-lr 30.0

# Jugador1 transmite por LR en la misma freq (rango default 20 km)
cargo run --release -p rmtfar-test-client -- \
  --id "Jugador1" --freq-lr 30.0 --ptt-radio-lr --pos 5000,0,0

# Jugador1 fuera de rango LR (> 20 km) → silencio
cargo run --release -p rmtfar-test-client -- \
  --id "Jugador1" --freq-lr 30.0 --ptt-radio-lr --pos 25000,0,0
```

#### 📶 Prueba de interferencia por distancia (DSP progresivo)

```bash
# Jugador2 escuchando en 43.0, rango 500m
cargo run --release -p rmtfar-test-client -- --id "Jugador2" --freq 43.0

# Señal fuerte (20% del rango) — q=0.80 — sin ruido, voz clara
cargo run --release -p rmtfar-test-client -- \
  --id "Jugador1" --freq 43.0 --ptt-radio --pos 100,0,0 --radio-range 500

# Señal media (60% del rango) — q=0.40 — ruido audible, sin dropouts
cargo run --release -p rmtfar-test-client -- \
  --id "Jugador1" --freq 43.0 --ptt-radio --pos 300,0,0 --radio-range 500

# Señal débil (90% del rango) — q=0.10 — estática fuerte + crackle/dropouts
cargo run --release -p rmtfar-test-client -- \
  --id "Jugador1" --freq 43.0 --ptt-radio --pos 450,0,0 --radio-range 500
```

En los logs verás el valor de `signal_quality` disminuir a medida que la distancia aumenta:
```
DEBUG rmtfar_plugin: radio — applying DSP uid=Jugador1 dist=100.0 signal_quality=0.80
DEBUG rmtfar_plugin: radio — applying DSP uid=Jugador1 dist=300.0 signal_quality=0.40
DEBUG rmtfar_plugin: radio — applying DSP uid=Jugador1 dist=450.0 signal_quality=0.10
```

#### 👂 Prueba de proximidad local

```bash
# Jugador1 a 20m → se escucha fuerte
cargo run --release -p rmtfar-test-client -- \
  --id "Jugador1" --ptt-local --pos 20,0,0

# Jugador1 a 45m → se escucha suave (volumen ~10%)
cargo run --release -p rmtfar-test-client -- \
  --id "Jugador1" --ptt-local --pos 45,0,0

# Jugador1 a 60m → silencio (> 50m de rango local)
cargo run --release -p rmtfar-test-client -- \
  --id "Jugador1" --ptt-local --pos 60,0,0
```

### Paso 5 — Verificar logs del plugin

Los logs aparecen en el terminal donde corrés Mumble:

```
INFO  RMTFAR: registering identity user_id=37 name=Jugador1
DEBUG rmtfar_plugin: radio — applying DSP uid=Jugador1 dist=200.0 signal_quality=0.60
DEBUG rmtfar_plugin: radio — applying DSP uid=Jugador1 dist=430.0 signal_quality=0.14
DEBUG rmtfar_plugin: out of radio range — muted uid=Jugador1 dist=800.0
DEBUG rmtfar_plugin: radio freq mismatch — muted uid=Jugador1 sender_freq=50.0 local_freq=43.0
DEBUG rmtfar_plugin: radio channel mismatch — muted uid=Jugador1 sender_ch=2 local_ch=1
DEBUG rmtfar_plugin: dead — muted uid=Jugador1
DEBUG rmtfar_plugin: unconscious — muted uid=Jugador1
DEBUG rmtfar_plugin: in vehicle, no radio PTT — muted uid=Jugador1
DEBUG rmtfar_plugin: local voice uid=Jugador1 dist=20.0 volume=0.60
DEBUG rmtfar_plugin: out of local range — muted uid=Jugador1 dist=60.0
```

---

## 🔍 Calidad de código

```bash
./check.sh        # fmt + clippy + tests + doc + SQF lint
cargo fmt --all   # Formateo automático
cargo test --workspace
```

**MSRV:** Rust **1.91+** (`rust-toolchain.toml` fija `1.91.0` con rustfmt y clippy; `workspace.package.rust-version` en `Cargo.toml`). Hace falta para APIs como `Duration::from_mins` y para que el mismo Clippy que en CI no falle por `duration_suboptimal_units`.

El CI corre en cada push: formato, clippy, **60 tests automatizados** (unitarios + integración) y build del plugin/bridge para Linux.

### 🧪 Cobertura de tests

| Crate | Tests | Qué cubren |
|---|---|---|
| `rmtfar-protocol` | 19 | Serialización, campos de `PlayerSummary`, vehicle, tuned_freq, LR, muerte/inconsciente |
| `rmtfar-plugin` | 22 | `process_audio`: freq, canal, rango, muerte, inconsciente, vehículo, proximidad, atenuación, DSP |
| `rmtfar-bridge` (unit) | 14 | Matching SR/LR, rango, muerte, canal, signal quality, vehicle |
| `rmtfar-bridge` (integración) | 5 | Bridge subprocess real + UDP: SR, multi-jugador, muerte, vehículo, LR |

### 🔊 Pipeline DSP de radio

El efecto de radio se aplica en `crates/rmtfar-plugin/src/dsp.rs`. Los parámetros son ajustables:

| Paso | Parámetro | Default | Efecto de modificarlo |
|---|---|---|---|
| High-pass biquad | `cutoff_hz = 300.0` | 300 Hz | Subir a 450 Hz → más "fino" y telefónico |
| Low-pass biquad | `cutoff_hz = 3_400.0` | 3 400 Hz | Bajar a 2 500 Hz → más apagado / lejano |
| AGC compressor | `threshold = 0.35, ratio = 4.0` | 4:1 | Ratio mayor → más comprimido / uniforme |
| Bitcrusher | `target_rate = 8_000` | 8 kHz | 12 000 Hz = menos artefactos; 6 000 Hz = más degradado |
| Soft-clip | `gain = 2.5` | 2.5 | Subir → más saturación / grit |
| Noise floor | `0.018 + (1-q)² × 0.25` | — | Subir el `0.018` → siempre hay estática de fondo |
| Dropout | activado cuando `q < 0.5` | — | Empieza al 50% del rango, máximo al límite |

#### Modelo `signal_quality`

```
signal_quality = 1.0 - (dist / range_m)   ∈ [0.0, 1.0]
```

| Calidad | Distancia | Comportamiento |
|---|---|---|
| 1.0 → 0.5 | 0–50% del rango | Señal clara, ruido mínimo |
| 0.5 → 0.1 | 50–90% del rango | Ruido creciente, empieza crackle |
| < 0.1 | > 90% del rango | Estática fuerte, dropouts frecuentes |

Una [auditoría de dependencias](.github/workflows/dep-audit.yml) se ejecuta automáticamente cada **1 de diciembre**.

---

## 🔗 Proyectos similares

| Proyecto | Backend de voz | Lenguaje |
|---|---|---|
| [TFAR](https://github.com/michail-nikolaev/task-force-arma-3-radio) | TeamSpeak 3 | C++ / SQF |
| [ACRE2](https://github.com/IDI-Systems/acre2) | TeamSpeak 3 | C++ / SQF |
| [FGCom-mumble](https://github.com/hbeni/fgcom-mumble) | Mumble | C++ / Lua |
| **RMTFAR** | **Mumble** | **🦀 Rust / SQF** |

---

## 📜 Licencia

**GPLv3** — ver [LICENSE](LICENSE).

Compatible con Mumble (GPLv3) y ACRE2 (GPLv3).  
No utiliza assets de Bohemia Interactive, por lo que la APL-SA no aplica.
