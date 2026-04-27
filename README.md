# 📻 RMTFAR — Radio Mumble Task Force Arma Radio

> Open-source TFAR-style radio mod for Arma 3, powered by Mumble/Murmur instead of TeamSpeak.

<div align="center">

[![CI](https://github.com/cavazquez/rmtfar/actions/workflows/ci.yml/badge.svg)](https://github.com/cavazquez/rmtfar/actions/workflows/ci.yml)
[![Coverage](https://codecov.io/gh/cavazquez/rmtfar/graph/badge.svg)](https://codecov.io/gh/cavazquez/rmtfar)
[![Rust](https://img.shields.io/badge/Rust-1.75+-f74c00?logo=rust&logoColor=white)](https://www.rust-lang.org/)
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
v1|player_id|server_id|tick|x|y|z|dir|alive|conscious|vehicle|ptt_local|ptt_sr|ptt_lr|sr_freq|sr_ch|lr_freq|lr_ch|radio_los|sr_range_m|lr_range_m
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
├── 📄 Cargo.toml                  # Workspace de Rust
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
    ├── fn_init.sqf             # CBA keybinds (PTT voz directa, radio SR)
    ├── fn_cbaKeybindHasUserKeys.sqf  # ¿Hay teclas reales en el perfil de CBA? (no pisar defaults)
    ├── fn_hudStart.sqf         # HUD RscTitles: frecuencias y estado PTT
    ├── fn_loop.sqf             # Loop: recolecta estado, broadcast, envía a extension
    ├── fn_getPlayerState.sqf   # Lee pos/dir/alive/radio de un jugador
    └── fn_sendState.sqf        # Serializa payload v1 y llama callExtension
```

El **plugin de Mumble para Windows** (`rmtfar_plugin.dll`) no va dentro de `@rmtfar/` en el juego: copiarlo desde **`dist/windows-x64/mumble/`** a `%APPDATA%\Mumble\Plugins\`.

### Keybinds (CBA)

Se registran dos acciones en *Configuración → Controles → Configurar addons → RMTFAR*:

| Acción | Variable SQF | Comportamiento del default |
|---|---|---|
| **PTT - Voz directa** | `RMTFAR_pttLocal` | Si en el perfil de CBA **no** hay ninguna tecla “real” para esa acción, el mod aplica **sin tecla** (podés asignar una después). Si ya configuraste teclas, **no se tocan**. |
| **PTT - Radio (corto alcance)** | `RMTFAR_pttRadioSR` | Si no hay teclas guardadas, el mod propone **Bloq Mayús (Caps Lock)**. Si ya hay teclas, **no se tocan**. |

La comprobación usa el registro `cba_keybinding_registry_v3` del perfil (misma regla que CBA: tecla con código `> 1`). Ver `fn_cbaKeybindHasUserKeys.sqf`.

**Radio LR:** el protocolo y el estado incluyen `RMTFAR_pttRadioLR`, pero **no hay keybind CBA** para LR en la versión actual; se puede activar por misión/script (p. ej. `RMTFAR_fnc_radioTransmit` solo cubre SR; para LR habría que asignar `RMTFAR_pttRadioLR` desde la misión si hace falta).

### HUD en pantalla

Con la extensión cargada y `RMTFAR_enabled`, el mod muestra un panel discreto (abajo a la derecha) con **SR** (frecuencia y canal), **LR** si hay frecuencia LR, e indicadores de **TX** (radio SR/LR y voz directa). No sustituye ni integra la UI de **TFAR**; si cargás ambos mods podés tener overlays distintos.

Para **ocultar** el panel:

```sqf
missionNamespace setVariable ["RMTFAR_showRadioHud", false];
```

Para volver a mostrarlo: `true` (valor por defecto en `XEH_preInit.sqf`).

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
