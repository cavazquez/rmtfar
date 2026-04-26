# 📻 RMTFAR — Radio Mumble Task Force Arma Radio

> Open-source TFAR-style radio mod for Arma 3, powered by Mumble/Murmur instead of TeamSpeak.

<div align="center">

[![CI](https://github.com/cavazquez/rmtfar/actions/workflows/ci.yml/badge.svg)](https://github.com/cavazquez/rmtfar/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/Rust-1.75+-f74c00?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE)
[![Mumble](https://img.shields.io/badge/Mumble-1.4.0+-darkgreen?logo=mumble&logoColor=white)](https://www.mumble.info/)
[![Arma 3](https://img.shields.io/badge/Arma_3-SQF-8B5C14)](https://store.steampowered.com/app/107410/Arma_3/)

</div>

---

## 🛠️ Tech Stack

| | Tecnología | Rol |
|---|---|---|
| 🦀 | **Rust** | Extension DLL, bridge y plugin de Mumble |
| 🎮 | **SQF** | Scripts dentro de Arma 3 |
| 🎙️ | **Mumble 1.4.0+** | Transporte de voz (cliente) |
| 🖥️ | **Murmur** | Servidor de voz |
| 📦 | **serde / serde_json** | Serialización del protocolo |
| 🔊 | **dasp** | DSP: filtro bandpass, soft-clip y ruido de radio |
| 🧵 | **UDP** | Comunicación local entre componentes |
| 🧠 | **MumbleLink** | Shared memory para audio posicional |
| ⚙️ | **C FFI** | Bindings al API de plugin de Mumble (1.4.0) |
| 🔒 | **GPLv3** | Licencia compatible con Mumble y ACRE2 |

---

## 🗺️ Arquitectura

```
┌──────────────────────────────┐
│  🎮 Arma 3 Client            │
│  ┌──────────────────────┐    │
│  │ SQF Scripts (@rmtfar)│    │
│  │  getPos, getDir, PTT │    │
│  └──────────┬───────────┘    │
│             │ callExtension  │
│  ┌──────────▼───────────┐    │
│  │ 🦀 Extension DLL     │    │
│  │   rmtfar_x64.dll     │    │
│  └──────────┬───────────┘    │
└─────────────┼────────────────┘
              │ UDP :9500 (localhost)
┌─────────────▼────────────────┐
│  🦀 RMTFAR Bridge            │
│  - Recibe estado del jugador │
│  - Escribe MumbleLink (shm)  │
│  - Broadcast radio → :9501   │
└──────┬───────────────────────┘
       │
       ├─── 🧠 SharedMem "MumbleLink" ──────────┐
       │                                         │
       └─── 📡 UDP :9501 ───────────────────────┤
                                                 │
┌────────────────────────────────────────────────▼──┐
│  🎙️ Mumble Client                                  │
│  ┌──────────────────────────────────────────────┐  │
│  │ 🦀 RMTFAR Plugin (Rust + C FFI)              │  │
│  │  - Lee MumbleLink (audio posicional)         │  │
│  │  - Recibe radio state del bridge             │  │
│  │  - Audio callbacks: mute/unmute por usuario  │  │
│  │  - 🔊 DSP: bandpass + soft-clip + ruido      │  │
│  └──────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────┘
```

| Componente | Dónde vive | Lenguaje | Rol |
|---|---|---|---|
| `@rmtfar` (mod Arma) | Arma 3 | SQF + DLL Rust | Captura y envía estado del jugador |
| Bridge | Máquina local | 🦀 Rust | Traduce estado de juego → datos Mumble |
| Plugin Mumble | Cliente Mumble | 🦀 Rust + C FFI | Procesa audio por usuario |

---

## 🚀 Estado de desarrollo

### ✅ Milestone — Pipeline end-to-end verificado en Linux

El sistema completo fue probado en Linux sin Arma 3:

| Componente | Estado |
|---|---|
| `rmtfar-bridge` — recibe estado del jugador vía UDP :9500 | ✅ |
| Plugin Mumble — carga correctamente en Mumble 1.5.x | ✅ |
| `mumble_onAudioSourceFetched` — intercepta audio entrante | ✅ |
| Mapeo de identidad: session_id → username → PlayerState | ✅ |
| Filtro de radio por frecuencia (mute si no coincide) | ✅ |
| DSP de radio (bandpass + soft-clip + noise) — **audible** | ✅ |
| Silencio cuando no hay PTT activo | ✅ |
| Voz de proximidad con atenuación por distancia | ✅ |
| Extension DLL para Arma 3 | ⚠️ Solo Windows (cross-compile) |

### 🗺️ Fases de desarrollo

| Fase | Descripción | Estado |
|---|---|---|
| **1** | Voz por proximidad (posición 3D, atenuación por distancia) | ✅ |
| **2** | Radio simple (frecuencia, PTT radio, efecto DSP) | ✅ |
| **3** | Lógica tipo TFAR (SR/LR, canales, rango, interferencia, vehículos) | 🔜 |

---

## 📨 Protocolo de mensajes

### 🎮 Arma 3 → Bridge (UDP :9500)

```json
{
  "v": 1,
  "type": "player_state",
  "steam_id": "76561198000000000",
  "server_id": "192.168.1.100:2302",
  "tick": 123456,
  "pos": [1234.5, 567.8, 12.3],
  "dir": 145.0,
  "alive": true,
  "conscious": true,
  "vehicle": "",
  "ptt_local": false,
  "ptt_radio_sr": false,
  "ptt_radio_lr": false,
  "radio_sr": { "freq": "152.000", "channel": 1, "volume": 1.0, "enabled": true },
  "radio_lr": null
}
```

### 🦀 Bridge → Plugin (UDP :9501)

```json
{
  "v": 1,
  "type": "radio_state",
  "server_id": "192.168.1.100:2302",
  "tick": 123456,
  "local_player": "76561198000000000",
  "players": [
    {
      "steam_id": "76561198000000000",
      "pos": [1234.5, 567.8, 12.3],
      "dir": 145.0,
      "alive": true,
      "transmitting_local": false,
      "transmitting_radio": true,
      "radio_freq": "152.000"
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
├── 🔍 check.sh                    # Quality gate: fmt + clippy + tests + SQF lint
├── 📂 .github/
│   └── workflows/
│       └── dep-audit.yml          # Auditoría anual de dependencias (diciembre)
├── 📚 docs/
│   ├── architecture.md
│   ├── protocol.md
│   ├── building.md
│   └── setup-guide.md
├── 📦 crates/
│   ├── rmtfar-protocol/           # Tipos compartidos (PlayerState, RadioStateMessage…)
│   ├── rmtfar-extension/          # DLL para Arma 3 (cdylib, C ABI)
│   ├── rmtfar-bridge/             # Proceso bridge local
│   ├── rmtfar-plugin/             # Plugin de Mumble (cdylib, C FFI)
│   └── rmtfar-test-client/        # Simulador sin necesidad de Arma 3
├── 🪖 arma-mod/
│   └── @rmtfar/                   # Mod de Arma 3 + DLL precompilada
└── 🔧 scripts/
    ├── build-all.sh
    ├── build-extension.sh         # Cross-compile a Windows (x64)
    ├── build-plugin.sh
    └── package-release.sh
```

---

## 📦 Dependencias

### 🦀 Rust

| Crate | Uso |
|---|---|
| `serde` + `serde_json` | Serialización del protocolo UDP |
| `anyhow` | Manejo de errores ergonómico |
| `mumble-link` | Shared memory MumbleLink (audio posicional) |
| `rust-mumble-sys` | Bindings al API C de plugins de Mumble 1.4 |
| `dasp` | DSP: filtro bandpass, soft-clip, generación de ruido |
| `windows` | Shared memory en Windows (bridge) |
| `libc` | Shared memory en Linux/macOS (bridge) |
| `clap` | CLI del bridge y del test-client |
| `tracing` | Logging estructurado |

### 🎮 Arma 3

| Mod | Requerido | Uso |
|---|---|---|
| [CBA_A3](https://github.com/CBATeam/CBA_A3) | Recomendado | Keybinds y settings |
| [ACE3](https://github.com/acemod/ACE3) | Opcional | Estado inconsciente |

### 🎙️ Voz

| Software | Versión mínima |
|---|---|
| [Mumble](https://www.mumble.info/) | 1.4.0+ |
| [Murmur](https://www.mumble.info/documentation/mumble-server/) | Cualquier versión reciente |

---

## 🐧 Cómo probar en Linux (sin Arma 3)

Guía paso a paso para verificar el sistema completo en Linux con dos instancias de Mumble.

### Requisitos

```bash
sudo apt install mumble murmur    # Mumble 1.5.x en Ubuntu 24.04+
sudo systemctl start mumbled      # Arrancar el servidor local
```

### Paso 1 — Compilar e instalar el plugin

```bash
./scripts/build-plugin-linux.sh
```

Genera `target/release/rmtfar_plugin.mumble_plugin`. Instalalo en Mumble:
**Configurar → Complementos → Instalar un plugin…** → seleccioná el `.mumble_plugin`.

### Paso 2 — Abrir dos instancias de Mumble

```bash
mumble &                     # Instancia A (escucha, tiene el plugin)
mumble --multiple &          # Instancia B (habla, sin plugin necesario)
```

> **Importante:** el `--id` del test-client debe coincidir con el **username de Mumble** del jugador que habla. El plugin usa el nombre de Mumble para mapear la identidad.

Conectá ambas instancias al servidor local (`localhost`).

### Paso 3 — Arrancar el bridge

```bash
cargo run --release -p rmtfar-bridge
```

### Paso 4 — Simular estado de jugadores

Reemplazá `NombreMumble_A` y `NombreMumble_B` con los usernames reales de cada instancia.

#### 🔊 Prueba de radio

```bash
# Instancia A escucha en freq 43.0
cargo run --release -p rmtfar-test-client -- \
  --id "NombreMumble_A" --freq 43.0

# Instancia B transmite por radio en freq 43.0
cargo run --release -p rmtfar-test-client -- \
  --id "NombreMumble_B" --freq 43.0 --ptt-radio
```

Cuando la Instancia B hable, la Instancia A escucha el audio con **efecto de radio** (filtro bandpass + ruido). Si cambiás la frecuencia de B a `44.0`, el audio se silencia.

#### 👂 Prueba de proximidad

```bash
# Instancia A (oyente) quieta en 0,0,0
cargo run --release -p rmtfar-test-client -- \
  --id "NombreMumble_A"

# Instancia B (hablante) orbitando — se aleja y acerca
cargo run --release -p rmtfar-test-client -- \
  --id "NombreMumble_B" --orbit --orbit-radius 30 --ptt-local

# También: B estático a distancia fija para comparar
# --pos 10,0,0  → volumen ~80%
# --pos 45,0,0  → volumen ~10%
# --pos 60,0,0  → silencio (> 50m)
```

El volumen de la voz de B se atenúa linealmente con la distancia (rango máximo: **50 m**).

### Paso 5 — Verificar logs

En el terminal del proceso Mumble de la Instancia A:

```
INFO  RMTFAR: registering identity user_id=X name="NombreMumble_B"
DEBUG rmtfar_plugin: radio — applying DSP uid="NombreMumble_B" dist=0.0
DEBUG rmtfar_plugin: local voice uid="NombreMumble_B" dist=23.5 volume=0.53
DEBUG rmtfar_plugin: out of local range — muted uid="NombreMumble_B"
```

---

## 🔍 Calidad de código

```bash
./check.sh        # fmt + clippy + tests + doc + SQF lint
cargo fmt --all   # Formateo automático
cargo test --workspace
```

El CI corre en cada push: formato, clippy, tests y build del plugin/bridge para Linux.  
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
